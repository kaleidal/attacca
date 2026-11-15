use crate::RATE;
use crate::mixer::transition::{Placement, TransitionKind};
use crate::{Rhythm, TrackPlan, TransitionSpec};

const STRETCH_LIMIT: f32 = 0.06;
const BLEND_LEAD_BEATS: f32 = 8.0;

impl Rhythm {
    fn beat_seconds(&self) -> f32 {
        60.0 / self.tempo
    }

    fn nearest(&self, seconds: f32, downbeats_only: bool) -> Option<f32> {
        self.beats
            .iter()
            .enumerate()
            .filter(|(index, _)| !downbeats_only || index % 4 == usize::from(self.downbeat))
            .map(|(_, beat)| *beat)
            .min_by(|a, b| (a - seconds).abs().total_cmp(&(b - seconds).abs()))
    }

    fn latest(&self, seconds: f32, downbeats_only: bool) -> Option<f32> {
        self.beats
            .iter()
            .enumerate()
            .rev()
            .find(|(index, beat)| {
                (!downbeats_only || index % 4 == usize::from(self.downbeat)) && **beat <= seconds
            })
            .map(|(_, beat)| *beat)
    }

    /// The downbeat closest to `seconds` when one is within two beats, otherwise the closest beat.
    /// Never later than `limit`.
    fn snap_cue(&self, seconds: f32, limit: f32) -> f32 {
        let reach = self.beat_seconds() * 2.0;
        let snapped = self
            .nearest(seconds, true)
            .filter(|downbeat| (downbeat - seconds).abs() <= reach)
            .or_else(|| self.nearest(seconds, false))
            .unwrap_or(seconds);
        if snapped <= limit {
            return snapped;
        }
        self.latest(limit, true)
            .filter(|downbeat| limit - downbeat <= reach)
            .or_else(|| self.latest(limit, false))
            .unwrap_or(limit)
    }

    fn first_downbeat_from(&self, seconds: f32) -> f32 {
        let tolerance = 0.05;
        self.beats
            .iter()
            .enumerate()
            .find(|(index, beat)| {
                index % 4 == usize::from(self.downbeat) && **beat >= seconds - tolerance
            })
            .map_or(seconds, |(_, beat)| beat.max(0.0))
    }
}

/// Tempo ratio between two songs, folded by octaves so a tempo detected at half or double speed
/// still matches.
fn tempo_ratio(outgoing: f32, incoming: f32) -> f32 {
    let mut ratio = incoming / outgoing;
    while ratio > std::f32::consts::SQRT_2 {
        ratio /= 2.0;
    }
    while ratio < std::f32::consts::FRAC_1_SQRT_2 {
        ratio *= 2.0;
    }
    ratio
}

/// How much of the outgoing song a transition plays past its cue, per second of overlap.
fn consumption(kind: TransitionKind, stretch: f32) -> f32 {
    match kind {
        TransitionKind::None | TransitionKind::Slam => 0.0,
        TransitionKind::Melt => 0.5,
        TransitionKind::Blend => stretch,
        TransitionKind::Fade
        | TransitionKind::Rise
        | TransitionKind::Wave
        | TransitionKind::Skip => 1.0,
    }
}

fn frames(seconds: f32) -> u64 {
    (seconds.max(0.0) * RATE as f32).round() as u64
}

/// Places a transition between the end of `outgoing` and the start of `incoming`. Returns the
/// placement and the incoming song's first frame.
pub fn place(
    outgoing: &TrackPlan,
    incoming: &TrackPlan,
    spec: &TransitionSpec,
) -> (Placement, u64) {
    let kind = spec.kind;
    let mut cue = outgoing.end.unwrap_or(outgoing.duration) as f32;
    let mut start = incoming.start as f32;
    let mut seconds = if kind == TransitionKind::None {
        0.0
    } else {
        spec.seconds
    };
    let rhythms = match (&outgoing.rhythm, &incoming.rhythm) {
        (Some(out), Some(inc)) if spec.snap && kind != TransitionKind::None => Some((out, inc)),
        _ => None,
    };
    let mut stretch = 1.0;
    let mut swap = 0.5;
    if let Some((out, inc)) = rhythms {
        start = inc.first_downbeat_from(start);
        let bar = inc.beat_seconds() * 4.0;
        if !matches!(kind, TransitionKind::Slam | TransitionKind::Skip) {
            let bars = (seconds / bar).round().max(1.0);
            seconds = bars * bar;
            if bars >= 2.0 {
                swap = (bars / 2.0).floor() / bars;
            }
        }
        let ratio = tempo_ratio(out.tempo, inc.tempo);
        if kind == TransitionKind::Blend && (ratio - 1.0).abs() <= STRETCH_LIMIT {
            stretch = ratio;
        }
    }
    let incoming_end = incoming.end.unwrap_or(incoming.duration) as f32;
    seconds = seconds.min((incoming_end - start).max(0.0));
    let room = outgoing.duration as f32 - seconds * consumption(kind, stretch);
    let limit = room.max(outgoing.start as f32);
    cue = match rhythms {
        Some((out, _)) => out.snap_cue(cue.min(limit), limit),
        None => cue.min(limit),
    };
    let out_beat = outgoing
        .rhythm
        .as_ref()
        .map_or(0.5, |rhythm| rhythm.beat_seconds());
    let lead = match kind {
        TransitionKind::Blend if stretch != 1.0 => out_beat * BLEND_LEAD_BEATS,
        TransitionKind::Slam => out_beat * 0.5,
        _ => 0.0,
    };
    let lead_in = (cue - lead).max(outgoing.start as f32);
    let placement = Placement {
        kind,
        cue: frames(cue),
        lead_in: frames(lead_in),
        overlap: frames(seconds),
        stretch,
        echo: frames(out_beat * 0.5) as u32,
        swap,
    };
    (placement, frames(start))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use super::place;
    use crate::{RATE, Rhythm, TrackPlan, TrackSound, TransitionKind, TransitionSpec};

    fn track(duration: f64, tempo: f32) -> TrackPlan {
        let beat = 60.0 / tempo;
        TrackPlan {
            token: 1,
            path: PathBuf::new(),
            start: 0.0,
            end: None,
            duration,
            sound: TrackSound::default(),
            rhythm: Some(Arc::new(Rhythm {
                tempo,
                beats: (0..(duration as f32 / beat) as usize)
                    .map(|k| k as f32 * beat)
                    .collect(),
                downbeat: 0,
            })),
        }
    }

    #[test]
    fn a_transition_at_the_very_end_moves_back_onto_a_beat_where_it_fits() {
        let outgoing = track(60.0, 120.0);
        let incoming = track(60.0, 120.0);
        let spec = TransitionSpec {
            kind: TransitionKind::Fade,
            seconds: 6.0,
            snap: true,
        };
        let (placement, _) = place(&outgoing, &incoming, &spec);
        let cue = placement.cue as f64 / f64::from(RATE);
        let overlap = placement.overlap as f64 / f64::from(RATE);
        assert!(cue + overlap <= 60.0 + 1e-3, "cue {cue} overlap {overlap}");
        assert!(
            (cue * 2.0 - (cue * 2.0).round()).abs() < 1e-3,
            "cue {cue} is off the beat"
        );
    }
}
