pub mod deck;
pub mod history;
pub mod message;
pub mod transition;

use std::sync::Arc;

use rtrb::{Consumer, Producer};

use crate::RATE;
use crate::dsp::echo::Echo;
use crate::dsp::limiter::Limiter;
use crate::sound::Sound;
use crate::source::Source;
use deck::Deck;
use message::{Command, Event, Garbage, Load, Shared, Snapshot};
use transition::{Placement, Shape, TransitionKind};

const BLOCK: usize = 32;
const FADE_STEP: f32 = 1.0 / (0.04 * RATE as f32);
const STOP_FRAMES: u64 = (RATE / 30) as u64;
const VOLUME_SMOOTHING: f32 = 0.0008;

struct Cued {
    load: Load,
    placement: Placement,
}

struct Crossing {
    placement: Placement,
    elapsed: u64,
    outgoing: usize,
}

struct Audition {
    clip: Arc<[[f32; 2]]>,
    position: usize,
}

pub struct Channels {
    pub commands: Consumer<Command>,
    pub events: Producer<Event>,
    pub garbage: Producer<Garbage>,
    pub tap: Producer<[f32; 2]>,
}

/// Everything that runs on the audio thread. It never allocates, locks or waits.
pub struct Mixer {
    channels: Channels,
    decks: [Deck; 2],
    current: usize,
    cued: Option<Cued>,
    crossing: Option<Crossing>,
    global: Sound,
    echo: Echo,
    limiter: Limiter,
    running: bool,
    level: f32,
    stopping: Option<u64>,
    unload_when_silent: bool,
    audition: Option<Audition>,
    audition_level: f32,
    volume: f32,
    volume_target: f32,
    shared: Arc<Shared>,
    mix: [[f32; 2]; BLOCK],
    send: [[f32; 2]; BLOCK],
}

impl Mixer {
    pub fn new(channels: Channels, shared: Arc<Shared>) -> Self {
        Self {
            channels,
            decks: [Deck::default(), Deck::default()],
            current: 0,
            cued: None,
            crossing: None,
            global: Sound::default(),
            echo: Echo::default(),
            limiter: Limiter::default(),
            running: false,
            level: 0.0,
            stopping: None,
            unload_when_silent: false,
            audition: None,
            audition_level: 0.0,
            volume: 1.0,
            volume_target: 1.0,
            shared,
            mix: [[0.0; 2]; BLOCK],
            send: [[0.0; 2]; BLOCK],
        }
    }

    /// Fills an interleaved stereo buffer.
    pub fn render(&mut self, out: &mut [f32]) {
        while let Ok(command) = self.channels.commands.pop() {
            self.apply(command);
        }
        let total = out.len() / 2;
        let mut offset = 0;
        while offset < total {
            let mut frames = (total - offset).min(BLOCK);
            self.mix = [[0.0; 2]; BLOCK];
            self.send = [[0.0; 2]; BLOCK];
            let main_target = if self.running && self.audition.is_none() {
                1.0
            } else {
                0.0
            };
            if self.level > 0.0 || main_target > 0.0 {
                frames = self.render_decks(frames);
            }
            for i in 0..frames {
                self.level = approach(self.level, main_target);
                let echo = self.echo.process(self.send[i]);
                let mut frame = [
                    (self.mix[i][0] + echo[0]) * self.level,
                    (self.mix[i][1] + echo[1]) * self.level,
                ];
                self.add_audition(&mut frame);
                let frame = self.limiter.process(frame);
                let _ = self.channels.tap.push(frame);
                self.volume += (self.volume_target - self.volume) * VOLUME_SMOOTHING;
                let gain = self.volume * self.volume;
                out[(offset + i) * 2] = frame[0] * gain;
                out[(offset + i) * 2 + 1] = frame[1] * gain;
            }
            offset += frames;
        }
        if self.unload_when_silent && self.level == 0.0 {
            self.unload_all();
            self.unload_when_silent = false;
        }
        self.publish();
    }

    fn add_audition(&mut self, frame: &mut [f32; 2]) {
        let target = if self.audition.is_some() { 1.0 } else { 0.0 };
        self.audition_level = approach(self.audition_level, target);
        let Some(audition) = &mut self.audition else {
            return;
        };
        if audition.clip.is_empty() {
            return;
        }
        let sample = audition.clip[audition.position];
        audition.position = (audition.position + 1) % audition.clip.len();
        frame[0] += sample[0] * self.audition_level;
        frame[1] += sample[1] * self.audition_level;
    }

    fn render_decks(&mut self, mut frames: usize) -> usize {
        if self.crossing.is_none()
            && let Some(cued) = &self.cued
        {
            let deck = &self.decks[self.current];
            let shape = lead_shape(deck, &cued.placement);
            let rate = if shape.stretch != 1.0 {
                shape.stretch
            } else {
                1.0
            };
            let until = if deck.is_exhausted() || !deck.is_loaded() {
                0
            } else {
                deck.frames_until(cued.placement.cue, rate)
            };
            if until == 0 {
                self.begin_crossing();
            } else {
                frames = frames.min(until as usize);
            }
        }
        let (mix, send) = (&mut self.mix[..frames], &mut self.send[..frames]);
        if let Some(crossing) = &mut self.crossing {
            let middle = crossing.elapsed as f32 + frames as f32 * 0.5;
            let t = (middle / crossing.placement.overlap as f32).min(1.0);
            let (outgoing, incoming) = transition::overlap(&crossing.placement, t);
            let released = [
                self.decks[crossing.outgoing].render(mix, send, outgoing, &self.global),
                self.decks[self.current].render(mix, send, incoming, &self.global),
            ];
            crossing.elapsed += frames as u64;
            let outgoing_done = self.decks[crossing.outgoing].is_silent()
                || self.decks[crossing.outgoing].is_exhausted();
            let overtime = crossing.elapsed >= crossing.placement.overlap + u64::from(RATE / 10);
            let finished =
                crossing.elapsed >= crossing.placement.overlap && outgoing_done || overtime;
            released
                .into_iter()
                .flatten()
                .for_each(|source| self.discard(source));
            if finished {
                self.finish_crossing();
            }
            return frames;
        }
        let deck = &mut self.decks[self.current];
        if !deck.is_loaded() {
            return frames;
        }
        let shape = match (&self.cued, self.stopping) {
            (_, Some(_)) => Shape {
                gain: 0.0,
                ..Shape::NEUTRAL
            },
            (Some(cued), None) => lead_shape(deck, &cued.placement),
            (None, None) => Shape::NEUTRAL,
        };
        let released = deck.render(mix, send, shape, &self.global);
        let over = deck.end.is_some_and(|end| deck.position() >= end as f64) || deck.is_exhausted();
        if let Some(source) = released {
            self.discard(source);
        }
        match self.stopping {
            Some(left) if left <= frames as u64 => {
                let token = self.decks[self.current].token;
                self.stopping = None;
                self.running = false;
                self.unload_all();
                let _ = self.channels.events.push(Event::Ended { token });
            }
            Some(left) => self.stopping = Some(left - frames as u64),
            None if over && self.cued.is_none() => self.stopping = Some(STOP_FRAMES),
            None => {}
        }
        frames
    }

    fn begin_crossing(&mut self) {
        let Some(Cued { load, placement }) = self.cued.take() else {
            return;
        };
        let incoming = 1 - self.current;
        self.unload_deck(incoming);
        let from = self.decks[self.current].token;
        let to = load.token;
        if let Some(old) =
            self.decks[incoming].load(load.source, load.token, load.sound, load.end, &self.global)
        {
            self.discard(old);
        }
        if placement.kind == TransitionKind::None {
            let [first, second] = &mut self.decks;
            let (from, to) = if incoming == 0 {
                (&*second, first)
            } else {
                (&*first, second)
            };
            to.inherit(from, &self.global);
            to.warm();
        }
        if placement.kind == TransitionKind::Slam {
            self.echo.set_delay(placement.echo as usize);
        }
        let outgoing = self.current;
        self.current = incoming;
        self.stopping = None;
        let _ = self.channels.events.push(Event::Crossing { from, to });
        if placement.kind == TransitionKind::None || placement.overlap == 0 {
            self.unload_deck(outgoing);
            let _ = self.channels.events.push(Event::Advanced { token: to });
        } else {
            self.crossing = Some(Crossing {
                placement,
                elapsed: 0,
                outgoing,
            });
        }
    }

    fn finish_crossing(&mut self) {
        if let Some(crossing) = self.crossing.take() {
            self.unload_deck(crossing.outgoing);
            let token = self.decks[self.current].token;
            let _ = self.channels.events.push(Event::Advanced { token });
        }
    }

    fn apply(&mut self, command: Command) {
        match command {
            Command::Play { load, skip } => self.play(load, skip),
            Command::Cue { load, placement } => {
                if let Some(old) = self.cued.replace(Cued { load, placement }) {
                    self.discard(old.load.source);
                }
            }
            Command::Uncue => {
                if let Some(old) = self.cued.take() {
                    self.discard(old.load.source);
                }
            }
            Command::Seek { source, token } => {
                self.finish_crossing();
                let deck = &mut self.decks[self.current];
                if deck.is_loaded() && deck.token == token {
                    self.stopping = None;
                    if let Some(old) = deck.replace(source, token) {
                        self.discard(old);
                    }
                } else {
                    self.discard(source);
                }
            }
            Command::Pause => self.running = false,
            Command::Resume => {
                self.running = self.decks[self.current].is_loaded();
                self.unload_when_silent = false;
            }
            Command::Stop => {
                self.running = false;
                self.unload_when_silent = true;
            }
            Command::Sound { token, sound, end } => {
                for deck in &mut self.decks {
                    if deck.is_loaded() && deck.token == token {
                        deck.set_sound(sound, end, &self.global);
                    }
                }
                if let Some(cued) = &mut self.cued
                    && cued.load.token == token
                {
                    cued.load.sound = sound;
                    cued.load.end = end;
                }
            }
            Command::Global(sound) => {
                self.global = sound;
                for deck in &mut self.decks {
                    deck.configure(&sound);
                }
            }
            Command::Volume(volume) => self.volume_target = volume.clamp(0.0, 1.0),
            Command::Audition(clip) => {
                let next = clip.map(|clip| Audition { clip, position: 0 });
                if let Some(old) = std::mem::replace(&mut self.audition, next) {
                    let _ = self.channels.garbage.push(Garbage::Clip(old.clip));
                }
            }
        }
    }

    fn play(&mut self, load: Load, skip: bool) {
        if let Some(old) = self.cued.take() {
            self.discard(old.load.source);
        }
        let token = load.token;
        let audible = self.running && self.level > 0.0 && self.decks[self.current].is_loaded();
        if skip && audible {
            self.finish_crossing();
            let kind = TransitionKind::Skip;
            self.cued = Some(Cued {
                load,
                placement: Placement {
                    kind,
                    cue: 0,
                    lead_in: 0,
                    overlap: (kind.default_seconds() * RATE as f32) as u64,
                    stretch: 1.0,
                    echo: 0,
                    swap: 0.5,
                },
            });
            self.begin_crossing();
        } else {
            self.unload_all();
            if let Some(old) = self.decks[self.current].load(
                load.source,
                load.token,
                load.sound,
                load.end,
                &self.global,
            ) {
                self.discard(old);
            }
        }
        self.running = true;
        self.unload_when_silent = false;
        self.stopping = None;
        let _ = self.channels.events.push(Event::Began { token });
    }

    fn unload_deck(&mut self, index: usize) {
        for source in self.decks[index].unload().into_iter().flatten() {
            self.discard(source);
        }
    }

    fn unload_all(&mut self) {
        self.crossing = None;
        self.unload_deck(0);
        self.unload_deck(1);
        if let Some(old) = self.cued.take() {
            self.discard(old.load.source);
        }
    }

    fn discard(&mut self, source: Source) {
        let _ = self.channels.garbage.push(Garbage::Source(source));
    }

    fn publish(&self) {
        let deck = &self.decks[self.current];
        let crossing = self
            .crossing
            .as_ref()
            .map(|crossing| crossing.elapsed as f32 / crossing.placement.overlap.max(1) as f32);
        self.shared.publish(Snapshot {
            token: if deck.is_loaded() { deck.token } else { 0 },
            seconds: deck.seconds(),
            playing: self.running && deck.is_loaded(),
            crossing,
            limiting_db: self.limiter.reduction_db(),
        });
    }
}

fn approach(value: f32, target: f32) -> f32 {
    if value < target {
        (value + FADE_STEP).min(target)
    } else {
        (value - FADE_STEP).max(target)
    }
}

fn lead_shape(deck: &Deck, placement: &Placement) -> Shape {
    let position = deck.position();
    if placement.lead_in >= placement.cue || position < placement.lead_in as f64 {
        return Shape::NEUTRAL;
    }
    let u = (position - placement.lead_in as f64) / (placement.cue - placement.lead_in) as f64;
    transition::lead_in(placement, u.clamp(0.0, 1.0) as f32)
}
