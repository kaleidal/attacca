use std::f32::consts::FRAC_PI_2;

use crate::dsp::filters::{OPEN_HIGH, OPEN_LOW};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum TransitionKind {
    None,
    Fade,
    Rise,
    Blend,
    Wave,
    Melt,
    Slam,
    Skip,
}

impl TransitionKind {
    pub fn default_seconds(self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Fade => 6.0,
            Self::Rise => 8.0,
            Self::Blend => 16.0,
            Self::Wave => 10.0,
            Self::Melt => 5.0,
            Self::Slam => 0.8,
            Self::Skip => 0.6,
        }
    }
}

/// A transition placed between two songs, in song frames of the outgoing song and output frames
/// of the overlap.
#[derive(Clone, Copy, Debug)]
pub struct Placement {
    pub kind: TransitionKind,
    /// Outgoing song frame where the incoming song starts.
    pub cue: u64,
    /// Outgoing song frame where the outgoing song's own automation begins.
    pub lead_in: u64,
    /// Output frames the two songs overlap.
    pub overlap: u64,
    /// Outgoing playback speed the blend settles on to match the incoming tempo.
    pub stretch: f32,
    /// Frames between echo repeats.
    pub echo: u32,
    /// Moment within the overlap, from 0 to 1, where the bass changes hands.
    pub swap: f32,
}

/// What a transition does to one deck at one moment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shape {
    pub gain: f32,
    pub lowpass: f32,
    pub highpass: f32,
    pub speed: f32,
    pub stretch: f32,
    /// Bass level relative to `gain`.
    pub bass: f32,
    pub send: f32,
}

impl Shape {
    pub const NEUTRAL: Shape = Shape {
        gain: 1.0,
        lowpass: OPEN_HIGH,
        highpass: OPEN_LOW,
        speed: 1.0,
        stretch: 1.0,
        bass: 1.0,
        send: 0.0,
    };
    const SILENT: Shape = Shape {
        gain: 0.0,
        ..Shape::NEUTRAL
    };
}

fn ramp(t: f32, from: f32, to: f32) -> f32 {
    ((t - from) / (to - from)).clamp(0.0, 1.0)
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn db(value: f32) -> f32 {
    10f32.powf(value / 20.0)
}

fn sweep(from: f32, to: f32, t: f32) -> f32 {
    from * (to / from).powf(t)
}

fn equal_power(balance: f32) -> (f32, f32) {
    let angle = balance.clamp(0.0, 1.0) * FRAC_PI_2;
    (angle.cos(), angle.sin())
}

/// The outgoing deck before the incoming song starts. `u` runs from 0 at `lead_in` to 1 at `cue`.
pub fn lead_in(placement: &Placement, u: f32) -> Shape {
    match placement.kind {
        TransitionKind::Blend => Shape {
            stretch: 1.0 + (placement.stretch - 1.0) * smooth(u),
            ..Shape::NEUTRAL
        },
        TransitionKind::Slam => Shape {
            send: 1.0,
            ..Shape::NEUTRAL
        },
        _ => Shape::NEUTRAL,
    }
}

/// Both decks during the overlap, outgoing first. `t` runs from 0 to 1.
pub fn overlap(placement: &Placement, t: f32) -> (Shape, Shape) {
    match placement.kind {
        TransitionKind::None => (Shape::SILENT, Shape::NEUTRAL),
        TransitionKind::Fade | TransitionKind::Skip => {
            let (out, inc) = equal_power(t);
            (
                Shape {
                    gain: out,
                    ..Shape::NEUTRAL
                },
                Shape {
                    gain: inc,
                    ..Shape::NEUTRAL
                },
            )
        }
        TransitionKind::Rise => rise(t),
        TransitionKind::Blend => blend(placement, t),
        TransitionKind::Wave => {
            let balance = t + 0.32
                * (std::f32::consts::PI * t).sin()
                * (6.0 * std::f32::consts::PI * t).sin();
            let (out, inc) = equal_power(balance);
            (
                Shape {
                    gain: out,
                    ..Shape::NEUTRAL
                },
                Shape {
                    gain: inc,
                    ..Shape::NEUTRAL
                },
            )
        }
        TransitionKind::Melt => melt(t),
        TransitionKind::Slam => (Shape::SILENT, Shape::NEUTRAL),
    }
}

const RISE_PEAK: f32 = 0.86;
const RISE_DROP: f32 = 0.06;

fn rise(t: f32) -> (Shape, Shape) {
    let climb = ramp(t, 0.0, RISE_PEAK);
    let incoming = Shape {
        gain: db(-30.0 * (1.0 - climb)),
        lowpass: sweep(160.0, OPEN_HIGH, climb.powf(1.6)),
        ..Shape::NEUTRAL
    };
    let drop = equal_power(ramp(t, RISE_PEAK, RISE_PEAK + RISE_DROP)).0;
    let outgoing = Shape {
        gain: db(-6.0 * climb) * drop,
        ..Shape::NEUTRAL
    };
    (outgoing, incoming)
}

fn blend(placement: &Placement, t: f32) -> (Shape, Shape) {
    let (out_gain, in_gain) = equal_power(t);
    let handover = smooth(ramp(t, placement.swap - 0.03, placement.swap + 0.03));
    let outgoing = Shape {
        gain: out_gain,
        stretch: placement.stretch,
        bass: relative(1.0 - handover, out_gain),
        ..Shape::NEUTRAL
    };
    let incoming = Shape {
        gain: in_gain,
        bass: relative(handover, in_gain),
        ..Shape::NEUTRAL
    };
    (outgoing, incoming)
}

fn relative(bass: f32, gain: f32) -> f32 {
    (bass / gain.max(1e-3)).min(8.0)
}

fn melt(t: f32) -> (Shape, Shape) {
    let outgoing = Shape {
        speed: (1.0 - t).max(0.0),
        lowpass: sweep(OPEN_HIGH, 320.0, t.powf(0.7)),
        gain: equal_power(ramp(t, 0.4, 1.0)).0,
        ..Shape::NEUTRAL
    };
    let incoming = Shape {
        gain: equal_power(ramp(t, 0.05, 0.65)).1,
        ..Shape::NEUTRAL
    };
    (outgoing, incoming)
}
