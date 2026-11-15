mod conductor;
mod dsp;
mod mixer;
mod output;
mod sound;
mod source;
mod visual;

use std::path::PathBuf;
use std::sync::Arc;

use crossbeam_channel::Sender;
use rtrb::RingBuffer;

pub use conductor::Audition;
use conductor::{Conductor, Request};
pub use dsp::eq::{BAND_HZ, EqCurve};
pub use dsp::filters::{Cut, FilterSettings};
use mixer::message::Shared;
pub use mixer::message::Snapshot;
pub use mixer::transition::TransitionKind;
use mixer::{Channels, Mixer};
pub use output::Device;
use output::Output;
pub use sound::{Sound, TrackSound};
use visual::Visual;
pub use visual::{BANDS as VISUAL_BANDS, WAVE as VISUAL_WAVE};

/// Every song is mixed at this rate.
pub const RATE: u32 = 48_000;

/// Beat positions of a song in seconds, with every fourth beat from `downbeat` starting a bar.
#[derive(Clone, Debug)]
pub struct Rhythm {
    pub tempo: f32,
    pub beats: Vec<f32>,
    pub downbeat: u8,
}

/// One song as the queue wants it played.
#[derive(Clone, Debug)]
pub struct TrackPlan {
    pub token: u64,
    pub path: PathBuf,
    pub start: f64,
    pub end: Option<f64>,
    pub duration: f64,
    pub sound: TrackSound,
    pub rhythm: Option<Arc<Rhythm>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransitionSpec {
    pub kind: TransitionKind,
    pub seconds: f32,
    pub snap: bool,
}

#[derive(Clone, Debug)]
pub enum EngineEvent {
    Began(u64),
    Crossing { from: u64, to: u64 },
    Advanced(u64),
    Ended(u64),
    Failed { token: u64, message: String },
    Devices(Vec<Device>),
}

/// Where a transition lands, in seconds: the outgoing song's cue, how long both play, and the
/// incoming song's first moment.
#[derive(Clone, Copy, Debug)]
pub struct Placed {
    pub kind: TransitionKind,
    pub cue: f64,
    pub overlap: f64,
    pub start: f64,
}

pub fn place(outgoing: &TrackPlan, incoming: &TrackPlan, spec: &TransitionSpec) -> Placed {
    let (placement, start) = conductor::place(outgoing, incoming, spec);
    let seconds = |frames: u64| frames as f64 / f64::from(RATE);
    Placed {
        kind: placement.kind,
        cue: seconds(placement.cue),
        overlap: seconds(placement.overlap),
        start: seconds(start),
    }
}

/// The player: two decks mixed on a real-time thread and played through PipeWire.
pub struct Engine {
    requests: Sender<Request>,
    shared: Arc<Shared>,
    output: Output,
    visual: Visual,
}

impl Engine {
    pub fn start(
        target: Option<String>,
        on_event: impl Fn(EngineEvent) + Send + Sync + 'static,
        on_visual: impl Fn(&[u8]) + Send + 'static,
    ) -> Engine {
        let (commands, command_consumer) = RingBuffer::new(256);
        let (event_producer, events) = RingBuffer::new(256);
        let (garbage_producer, garbage) = RingBuffer::new(1024);
        let (tap, tap_consumer) = RingBuffer::new(RATE as usize / 4);
        let shared = Arc::new(Shared::default());
        let mixer = Mixer::new(
            Channels {
                commands: command_consumer,
                events: event_producer,
                garbage: garbage_producer,
                tap,
            },
            Arc::clone(&shared),
        );
        let on_event = Arc::new(on_event);
        let devices_event = Arc::clone(&on_event);
        let output = Output::start(mixer, target, move |devices| {
            devices_event(EngineEvent::Devices(devices))
        });
        let visual = Visual::start(tap_consumer, on_visual);
        let (requests, receiver) = crossbeam_channel::unbounded();
        std::thread::Builder::new()
            .name("attacca-conductor".into())
            .spawn(move || {
                Conductor {
                    requests: receiver,
                    commands,
                    events,
                    garbage,
                    on_event: |event| on_event(event),
                    current: None,
                    next: None,
                }
                .run();
            })
            .expect("the conductor thread starts");
        Engine {
            requests,
            shared,
            output,
            visual,
        }
    }

    fn request(&self, request: Request) {
        let _ = self.requests.send(request);
    }

    /// Plays a song now. With `skip`, a playing song gives way with a short fade.
    pub fn play(&self, track: TrackPlan, skip: bool) {
        self.request(Request::Play { track, skip });
    }

    /// Sets what follows the playing song and how the two meet.
    pub fn cue(&self, next: Option<(TrackPlan, TransitionSpec)>) {
        self.request(Request::Cue(next));
    }

    pub fn seek(&self, seconds: f64) {
        self.request(Request::Seek(seconds));
    }

    pub fn pause(&self) {
        self.request(Request::Pause);
    }

    pub fn resume(&self) {
        self.request(Request::Resume);
    }

    pub fn stop(&self) {
        self.request(Request::Stop);
    }

    pub fn set_track_sound(&self, token: u64, sound: TrackSound, end: Option<f64>) {
        self.request(Request::Sound { token, sound, end });
    }

    pub fn set_global(&self, sound: Sound) {
        self.request(Request::Global(sound));
    }

    /// Loops a short stretch of a song in place of the mix, or returns to the mix with `None`.
    pub fn audition(&self, audition: Option<Audition>) {
        self.request(Request::Audition(audition));
    }

    /// Sets the listening volume, from silent at 0 to full at 1, on a perceptual curve.
    pub fn set_volume(&self, volume: f32) {
        self.request(Request::Volume(volume));
    }

    pub fn set_output(&self, target: Option<String>) {
        self.output.set_target(target);
    }

    pub fn set_visual(&self, active: bool) {
        self.visual.set_active(active);
    }

    pub fn snapshot(&self) -> Snapshot {
        self.shared.snapshot()
    }
}
