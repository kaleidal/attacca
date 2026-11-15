use crate::RATE;
use crate::dsp::eq::Equalizer;
use crate::dsp::filters::{Crossover, SlopeFilter};
use crate::dsp::stretch::{LOOKAHEAD, Stretcher};
use crate::dsp::svf::Shape as FilterShape;
use crate::mixer::history::History;
use crate::mixer::transition::Shape;
use crate::sound::{Sound, TrackSound};
use crate::source::Source;

const SMOOTHING: f32 = 0.0139;
const SEND_SMOOTHING: f32 = 0.005;
/// Most a song's level moves in one block when its gain changes: 0.05 dB.
const LEVEL_GLIDE: f32 = 1.005_773;
const DECLICK_FRAMES: f32 = 96.0;

#[derive(Clone, Copy, PartialEq)]
enum Reading {
    Direct,
    Varispeed,
    Stretched,
}

/// One song on its way to the mix: where it is, how it is read, and its own sound.
pub struct Deck {
    pub token: u64,
    source: Option<Source>,
    replacement: Option<(Source, u64)>,
    history: History,
    position: f64,
    skew: f64,
    reading: Reading,
    stretcher: Stretcher,
    pub end: Option<u64>,
    sound: TrackSound,
    equalizer: Equalizer,
    highpass: SlopeFilter,
    lowpass: SlopeFilter,
    sweep_high: SlopeFilter,
    sweep_low: SlopeFilter,
    crossover: Crossover,
    gain: f32,
    level: f32,
    bass: f32,
    send: f32,
    declick: f32,
}

impl Default for Deck {
    fn default() -> Self {
        Self {
            token: 0,
            source: None,
            replacement: None,
            history: History::default(),
            position: 0.0,
            skew: 0.0,
            reading: Reading::Direct,
            stretcher: Stretcher::default(),
            end: None,
            sound: TrackSound::default(),
            equalizer: Equalizer::default(),
            highpass: SlopeFilter::new(FilterShape::Highpass),
            lowpass: SlopeFilter::new(FilterShape::Lowpass),
            sweep_high: SlopeFilter::new(FilterShape::Highpass),
            sweep_low: SlopeFilter::new(FilterShape::Lowpass),
            crossover: Crossover::default(),
            gain: 0.0,
            level: 1.0,
            bass: 1.0,
            send: 0.0,
            declick: 1.0,
        }
    }
}

impl Deck {
    pub fn is_loaded(&self) -> bool {
        self.source.is_some()
    }

    /// Song frame being played. A stream that opens with the previous song's last frames is
    /// skewed by them.
    pub fn position(&self) -> f64 {
        self.position - self.skew
    }

    pub fn seconds(&self) -> f64 {
        self.position().max(0.0) / f64::from(RATE)
    }

    /// Starts a song from silence. Returns the source it replaced, to be dropped off the audio
    /// thread.
    pub fn load(
        &mut self,
        source: Source,
        token: u64,
        sound: TrackSound,
        end: Option<u64>,
        global: &Sound,
    ) -> Option<Source> {
        self.token = token;
        self.end = end;
        self.sound = sound;
        self.history.reset(source.origin);
        self.position = source.origin as f64;
        self.skew = source.status.skew() as f64;
        self.reading = Reading::Direct;
        self.gain = 0.0;
        self.level = sound.gain;
        self.bass = 1.0;
        self.send = 0.0;
        self.declick = 1.0;
        self.configure(global);
        self.equalizer.reset();
        self.highpass.jump();
        self.lowpass.jump();
        self.sweep_high.set_hz(Shape::NEUTRAL.highpass);
        self.sweep_low.set_hz(Shape::NEUTRAL.lowpass);
        self.sweep_high.jump();
        self.sweep_low.jump();
        self.crossover.reset();
        self.source.replace(source)
    }

    /// Continues another deck's filters, so a gapless join sounds like one stream.
    pub fn inherit(&mut self, from: &Deck, global: &Sound) {
        self.equalizer = from.equalizer.clone();
        self.highpass = from.highpass.clone();
        self.lowpass = from.lowpass.clone();
        self.crossover = from.crossover.clone();
        self.configure(global);
    }

    /// Skips the fade-in, for a cut or a gapless join where the first sample must be at full
    /// level.
    pub fn warm(&mut self) {
        self.gain = self.level * self.equalizer.preamp();
    }

    /// Moves to another place in the same song, fading out and back in around the jump.
    pub fn replace(&mut self, source: Source, token: u64) -> Option<Source> {
        self.replacement
            .replace((source, token))
            .map(|(source, _)| source)
    }

    pub fn unload(&mut self) -> [Option<Source>; 2] {
        self.token = 0;
        [
            self.source.take(),
            self.replacement.take().map(|(source, _)| source),
        ]
    }

    pub fn set_sound(&mut self, sound: TrackSound, end: Option<u64>, global: &Sound) {
        self.sound = sound;
        self.end = end;
        self.configure(global);
    }

    pub fn configure(&mut self, global: &Sound) {
        self.equalizer.set(self.sound.eq.unwrap_or(global.eq));
        let filters = self.sound.filters.unwrap_or(global.filters);
        self.highpass.set(filters.highpass);
        self.lowpass.set(filters.lowpass);
    }

    pub fn is_silent(&self) -> bool {
        self.gain.abs() < 1e-5
    }

    pub fn is_exhausted(&self) -> bool {
        match &self.source {
            Some(source) => {
                source.status.finished()
                    && source.frames.is_empty()
                    && self.position >= self.history.end() as f64
            }
            None => true,
        }
    }

    pub fn is_starved(&self) -> bool {
        self.source
            .as_ref()
            .is_some_and(|source| !source.status.finished() && source.frames.is_empty())
            && self.position + 1.0 >= self.history.end() as f64
    }

    /// Output frames until the play position reaches `frame`, at the current reading speed.
    pub fn frames_until(&self, frame: u64, rate: f32) -> u64 {
        let remaining = frame as f64 - self.position();
        if remaining <= 0.0 {
            0
        } else {
            (remaining / f64::from(rate.max(0.01))).ceil() as u64
        }
    }

    /// Renders and adds this deck into `out`, and its echo send into `send`. Returns a source
    /// the deck let go of.
    pub fn render(
        &mut self,
        out: &mut [[f32; 2]],
        send: &mut [[f32; 2]],
        shape: Shape,
        global: &Sound,
    ) -> Option<Source> {
        let source = self.source.as_mut()?;
        let rate = if shape.stretch != 1.0 {
            shape.stretch
        } else {
            shape.speed
        };
        let reach = match self.reading {
            Reading::Stretched => self.stretcher.needed_until(),
            _ => {
                self.position.max(0.0) as u64
                    + (out.len() as f64 * f64::from(rate)) as u64
                    + LOOKAHEAD
            }
        };
        self.history.fill(&mut source.frames, reach);
        if self.is_starved() {
            return None;
        }
        let preamp = self.equalizer.update();
        self.highpass.update();
        self.lowpass.update();
        self.sweep_high.set_hz(shape.highpass);
        self.sweep_low.set_hz(shape.lowpass);
        self.sweep_high.update();
        self.sweep_low.update();
        let ratio =
            (self.sound.gain / self.level.max(1e-6)).clamp(LEVEL_GLIDE.recip(), LEVEL_GLIDE);
        self.level *= ratio;
        let level = self.level * preamp;
        let mut released = None;
        for (frame, send_frame) in out.iter_mut().zip(send.iter_mut()) {
            let dry = self.read(shape);
            let shaped = self.chain(dry);
            let (low, high) = self.crossover.split(shaped);
            self.gain += (shape.gain * level * self.declick - self.gain) * SMOOTHING;
            self.bass += (shape.bass - self.bass) * SMOOTHING;
            self.send += (shape.send - self.send) * SEND_SMOOTHING;
            let wet = [
                (low[0] * self.bass + high[0]) * self.gain,
                (low[1] * self.bass + high[1]) * self.gain,
            ];
            frame[0] += wet[0];
            frame[1] += wet[1];
            send_frame[0] += wet[0] * self.send;
            send_frame[1] += wet[1] * self.send;
            if released.is_none() {
                released = self.step_declick(global);
            }
        }
        released
    }

    fn step_declick(&mut self, global: &Sound) -> Option<Source> {
        if self.replacement.is_some() {
            self.declick = (self.declick - 1.0 / DECLICK_FRAMES).max(0.0);
            if self.declick == 0.0 {
                let (source, token) = self.replacement.take()?;
                let sound = self.sound;
                let end = self.end;
                let old = self.load(source, token, sound, end, global);
                self.gain = 0.0;
                self.declick = 0.0;
                return old;
            }
        } else if self.declick < 1.0 {
            self.declick = (self.declick + 1.0 / DECLICK_FRAMES).min(1.0);
        }
        None
    }

    #[inline]
    fn read(&mut self, shape: Shape) -> [f32; 2] {
        if shape.stretch != 1.0 || self.reading == Reading::Stretched {
            if self.reading != Reading::Stretched {
                self.stretcher
                    .start(&self.history, self.position.round() as i64);
                self.reading = Reading::Stretched;
            }
            let frame = self.stretcher.next(&self.history, shape.stretch);
            self.position = self.stretcher.position();
            return frame;
        }
        if shape.speed != 1.0 || self.reading == Reading::Varispeed {
            self.reading = Reading::Varispeed;
            let frame = self.history.interpolate(self.position);
            self.position += f64::from(shape.speed);
            return frame;
        }
        let frame = self.history.at(self.position as i64);
        self.position += 1.0;
        frame
    }

    #[inline]
    fn chain(&mut self, frame: [f32; 2]) -> [f32; 2] {
        let frame = self.equalizer.process(frame);
        let frame = self.lowpass.process(self.highpass.process(frame));
        self.sweep_low.process(self.sweep_high.process(frame))
    }
}
