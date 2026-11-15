mod plan;

pub use plan::place;

use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::{Receiver, RecvTimeoutError};
use rtrb::{Consumer, Producer};

use crate::mixer::message::{Command, Event, Garbage, Load};
use crate::mixer::transition::TransitionKind;
use crate::sound::{Sound, TrackSound};
use crate::source::{Source, SourceStatus, decode_clip};
use crate::{EngineEvent, RATE, TrackPlan, TransitionSpec};

const TICK: Duration = Duration::from_millis(20);
const CLIP_FADE: usize = (RATE / 200) as usize;

pub enum Request {
    Play {
        track: TrackPlan,
        skip: bool,
    },
    Cue(Option<(TrackPlan, TransitionSpec)>),
    Seek(f64),
    Pause,
    Resume,
    Stop,
    Sound {
        token: u64,
        sound: TrackSound,
        end: Option<f64>,
    },
    Global(Sound),
    Volume(f32),
    Audition(Option<Audition>),
}

pub struct Audition {
    pub track: TrackPlan,
    pub from: f64,
    pub to: f64,
}

/// Runs beside the audio thread: opens songs ahead of time, places transitions, frees what the
/// audio thread lets go of and passes its events on.
pub struct Conductor<F> {
    pub requests: Receiver<Request>,
    pub commands: Producer<Command>,
    pub events: Consumer<Event>,
    pub garbage: Consumer<Garbage>,
    pub on_event: F,
    pub current: Option<Playing>,
    pub next: Option<Playing>,
}

/// A song handed to the audio thread, and the status of its decoder.
pub struct Playing {
    track: TrackPlan,
    status: Arc<SourceStatus>,
}

fn frames(seconds: f64) -> u64 {
    (seconds.max(0.0) * f64::from(RATE)).round() as u64
}

impl<F: Fn(EngineEvent)> Conductor<F> {
    pub fn run(mut self) {
        loop {
            match self.requests.recv_timeout(TICK) {
                Ok(request) => self.handle(request),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return,
            }
            while let Ok(event) = self.events.pop() {
                self.relay(event);
            }
            while self.garbage.pop().is_ok() {}
        }
    }

    fn send(&mut self, command: Command) {
        if self.commands.push(command).is_err() {
            eprintln!("attacca: the audio thread is not taking commands");
        }
    }

    fn fail(&self, track: &TrackPlan, error: attacca_media::MediaError) {
        (self.on_event)(EngineEvent::Failed {
            token: track.token,
            message: error.to_string(),
        });
    }

    fn open(&self, track: &TrackPlan, from: u64) -> Option<Source> {
        Source::open(&track.path, from)
            .map_err(|error| self.fail(track, error))
            .ok()
    }

    /// Wraps a source for the audio thread and remembers it as playing now or next.
    fn load(&mut self, source: Source, track: TrackPlan, now: bool) -> Load {
        let load = Load {
            token: track.token,
            sound: track.sound,
            end: track.end.map(frames),
            source,
        };
        let playing = Some(Playing {
            status: Arc::clone(&load.source.status),
            track,
        });
        if now {
            self.current = playing;
        } else {
            self.next = playing;
        }
        load
    }

    fn handle(&mut self, request: Request) {
        match request {
            Request::Play { track, skip } => {
                let Some(source) = self.open(&track, frames(track.start)) else {
                    return;
                };
                self.next = None;
                let load = self.load(source, track, true);
                self.send(Command::Play { load, skip });
            }
            Request::Cue(None) => {
                self.next = None;
                self.send(Command::Uncue);
            }
            Request::Cue(Some((track, spec))) => {
                let Some(current) = &self.current else {
                    return;
                };
                let (placement, start) = plan::place(&current.track, &track, &spec);
                let source = if placement.kind == TransitionKind::None && start == 0 {
                    Source::open_following(&track.path, &current.status)
                        .map_err(|error| self.fail(&track, error))
                        .ok()
                } else {
                    self.open(&track, start)
                };
                let Some(source) = source else {
                    return;
                };
                let load = self.load(source, track, false);
                self.send(Command::Cue { load, placement });
            }
            Request::Seek(seconds) => {
                let Some(current) = &mut self.current else {
                    return;
                };
                let token = current.track.token;
                match Source::open(&current.track.path, frames(seconds)) {
                    Ok(source) => {
                        current.status = Arc::clone(&source.status);
                        self.send(Command::Seek { source, token });
                    }
                    Err(error) => {
                        let track = current.track.clone();
                        self.fail(&track, error);
                    }
                }
            }
            Request::Pause => self.send(Command::Pause),
            Request::Resume => self.send(Command::Resume),
            Request::Stop => {
                self.current = None;
                self.next = None;
                self.send(Command::Stop);
            }
            Request::Sound { token, sound, end } => {
                for playing in self.current.iter_mut().chain(self.next.iter_mut()) {
                    if playing.track.token == token {
                        playing.track.sound = sound;
                        playing.track.end = end;
                    }
                }
                self.send(Command::Sound {
                    token,
                    sound,
                    end: end.map(frames),
                });
            }
            Request::Global(sound) => self.send(Command::Global(sound)),
            Request::Volume(volume) => self.send(Command::Volume(volume)),
            Request::Audition(None) => self.send(Command::Audition(None)),
            Request::Audition(Some(audition)) => {
                if let Some(clip) = self.clip(&audition) {
                    self.send(Command::Audition(Some(clip)));
                }
            }
        }
    }

    fn clip(&self, audition: &Audition) -> Option<Arc<[[f32; 2]]>> {
        let from = frames(audition.from);
        let length = frames(audition.to).saturating_sub(from);
        let mut clip = match decode_clip(&audition.track.path, from, length) {
            Ok(clip) => clip,
            Err(error) => {
                (self.on_event)(EngineEvent::Failed {
                    token: audition.track.token,
                    message: error.to_string(),
                });
                return None;
            }
        };
        let gain = audition.track.sound.gain;
        let fade = CLIP_FADE.min(clip.len() / 2);
        let length = clip.len();
        for (i, frame) in clip.iter_mut().enumerate() {
            let edge = i.min(length - 1 - i);
            let ramp = if edge < fade {
                edge as f32 / fade as f32
            } else {
                1.0
            };
            frame[0] *= gain * ramp;
            frame[1] *= gain * ramp;
        }
        Some(clip.into())
    }

    fn relay(&mut self, event: Event) {
        let event = match event {
            Event::Began { token } => EngineEvent::Began(token),
            Event::Crossing { from, to } => {
                if self
                    .next
                    .as_ref()
                    .is_some_and(|next| next.track.token == to)
                {
                    self.current = self.next.take();
                }
                EngineEvent::Crossing { from, to }
            }
            Event::Advanced { token } => EngineEvent::Advanced(token),
            Event::Ended { token } => {
                self.current = None;
                EngineEvent::Ended(token)
            }
        };
        (self.on_event)(event);
    }
}
