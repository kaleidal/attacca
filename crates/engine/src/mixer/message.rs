use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

use crate::mixer::transition::Placement;
use crate::sound::{Sound, TrackSound};
use crate::source::Source;

pub struct Load {
    pub source: Source,
    pub token: u64,
    pub sound: TrackSound,
    pub end: Option<u64>,
}

pub enum Command {
    Play {
        load: Load,
        skip: bool,
    },
    Cue {
        load: Load,
        placement: Placement,
    },
    Uncue,
    Seek {
        source: Source,
        token: u64,
    },
    Pause,
    Resume,
    Stop,
    Sound {
        token: u64,
        sound: TrackSound,
        end: Option<u64>,
    },
    Global(Sound),
    Volume(f32),
    Audition(Option<Arc<[[f32; 2]]>>),
}

#[derive(Clone, Copy, Debug)]
pub enum Event {
    /// A song started playing from a stop or a skip.
    Began { token: u64 },
    /// The next song started; the transition into it is running.
    Crossing { from: u64, to: u64 },
    /// The transition finished and only `token` plays.
    Advanced { token: u64 },
    /// The song ended with nothing after it.
    Ended { token: u64 },
}

/// Things the audio thread hands back so their memory is freed elsewhere.
#[expect(dead_code, reason = "held only to be dropped off the audio thread")]
pub enum Garbage {
    Source(Source),
    Clip(Arc<[[f32; 2]]>),
}

const NO_CROSSING: u32 = u32::MAX;

/// What the audio thread publishes about itself, readable at any time.
pub struct Shared {
    token: AtomicU64,
    position: AtomicU64,
    playing: AtomicBool,
    crossing: AtomicU32,
    reduction: AtomicU32,
}

impl Default for Shared {
    fn default() -> Self {
        Self {
            token: AtomicU64::new(0),
            position: AtomicU64::new(0),
            playing: AtomicBool::new(false),
            crossing: AtomicU32::new(NO_CROSSING),
            reduction: AtomicU32::new(0),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Snapshot {
    pub token: u64,
    pub seconds: f64,
    pub playing: bool,
    pub crossing: Option<f32>,
    pub limiting_db: f32,
}

impl Shared {
    pub fn publish(&self, snapshot: Snapshot) {
        self.token.store(snapshot.token, Ordering::Relaxed);
        self.position
            .store(snapshot.seconds.to_bits(), Ordering::Relaxed);
        self.playing.store(snapshot.playing, Ordering::Relaxed);
        let crossing = snapshot.crossing.map_or(NO_CROSSING, f32::to_bits);
        self.crossing.store(crossing, Ordering::Relaxed);
        self.reduction
            .store(snapshot.limiting_db.to_bits(), Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> Snapshot {
        let crossing = self.crossing.load(Ordering::Relaxed);
        Snapshot {
            token: self.token.load(Ordering::Relaxed),
            seconds: f64::from_bits(self.position.load(Ordering::Relaxed)),
            playing: self.playing.load(Ordering::Relaxed),
            crossing: (crossing != NO_CROSSING).then(|| f32::from_bits(crossing)),
            limiting_db: f32::from_bits(self.reduction.load(Ordering::Relaxed)),
        }
    }
}
