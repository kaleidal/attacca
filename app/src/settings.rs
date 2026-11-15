use attacca_engine::{
    Cut as EngineCut, EqCurve, FilterSettings, Sound, TransitionKind, TransitionSpec,
};
use attacca_library::{Cut, Equalizer, Filters, Library, Transition, TransitionName};
use serde::{Deserialize, Serialize};
use specta::Type;

pub const TARGET_LUFS: f32 = -16.0;
const REPLAY_GAIN_REFERENCE: f32 = -18.0;
const UNKNOWN_GAIN_DB: f32 = -6.0;
const PEAK_ROOM_DB: f32 = 3.0;

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub name: String,
    pub equalizer: Equalizer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum CoverWrites {
    Ask,
    Files,
    Attacca,
}

/// How everything sounds, unless a song says otherwise.
#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SoundSettings {
    pub equalizer: Equalizer,
    pub filters: Filters,
    pub matching: bool,
    pub presets: Vec<Preset>,
    pub output: Option<String>,
    pub volume: f32,
    pub cover_writes: CoverWrites,
}

impl Default for SoundSettings {
    fn default() -> Self {
        Self {
            equalizer: flat(),
            filters: Filters::default(),
            matching: true,
            presets: default_presets(),
            output: None,
            volume: 0.85,
            cover_writes: CoverWrites::Ask,
        }
    }
}

pub fn flat() -> Equalizer {
    Equalizer {
        preamp: 0.0,
        gains: vec![0.0; 10],
    }
}

fn preset(name: &str, gains: [f32; 10]) -> Preset {
    Preset {
        name: name.to_string(),
        equalizer: Equalizer {
            preamp: -gains.iter().copied().fold(0.0, f32::max).max(0.0) * 0.5,
            gains: gains.to_vec(),
        },
    }
}

fn default_presets() -> Vec<Preset> {
    vec![
        preset(
            "Warm",
            [3.0, 2.5, 1.5, 0.5, 0.0, -0.5, -1.0, -1.5, -2.0, -2.5],
        ),
        preset("Air", [0.0, 0.0, -0.5, -0.5, 0.0, 0.5, 1.5, 2.5, 3.5, 4.0]),
        preset(
            "Low end",
            [5.0, 4.0, 2.5, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ),
        preset(
            "Voice",
            [-2.0, -1.5, -1.0, 0.5, 2.0, 3.0, 2.5, 1.0, 0.0, -1.0],
        ),
        preset(
            "Night",
            [-3.0, -2.0, -1.0, 0.0, 0.5, 1.0, 1.0, 0.0, -1.5, -3.0],
        ),
    ]
}

pub fn load(library: &Library) -> SoundSettings {
    library.setting("sound").unwrap_or_default()
}

pub fn curve(equalizer: &Equalizer) -> EqCurve {
    let mut gains = [0.0; 10];
    for (gain, value) in gains.iter_mut().zip(&equalizer.gains) {
        *gain = value.clamp(-12.0, 12.0);
    }
    EqCurve {
        preamp: equalizer.preamp.clamp(-12.0, 12.0),
        gains,
    }
}

fn cut(cut: Option<Cut>) -> Option<EngineCut> {
    cut.map(|cut| EngineCut {
        hz: cut.hz.clamp(15.0, 20_000.0),
        slope: cut.slope.clamp(12, 48),
    })
}

pub fn filter_settings(filters: &Filters) -> FilterSettings {
    FilterSettings {
        highpass: cut(filters.highpass),
        lowpass: cut(filters.lowpass),
    }
}

pub fn global_sound(settings: &SoundSettings) -> Sound {
    Sound {
        eq: curve(&settings.equalizer),
        filters: filter_settings(&settings.filters),
    }
}

pub fn spec(transition: &Transition) -> TransitionSpec {
    let kind = match transition.kind {
        TransitionName::None => TransitionKind::None,
        TransitionName::Fade => TransitionKind::Fade,
        TransitionName::Rise => TransitionKind::Rise,
        TransitionName::Blend => TransitionKind::Blend,
        TransitionName::Wave => TransitionKind::Wave,
        TransitionName::Melt => TransitionKind::Melt,
        TransitionName::Slam => TransitionKind::Slam,
    };
    TransitionSpec {
        kind,
        seconds: transition.seconds.clamp(0.0, 60.0),
        snap: transition.snap,
    }
}

pub fn name(kind: TransitionKind) -> TransitionName {
    match kind {
        TransitionKind::None => TransitionName::None,
        TransitionKind::Fade | TransitionKind::Skip => TransitionName::Fade,
        TransitionKind::Rise => TransitionName::Rise,
        TransitionKind::Blend => TransitionName::Blend,
        TransitionKind::Wave => TransitionName::Wave,
        TransitionKind::Melt => TransitionName::Melt,
        TransitionKind::Slam => TransitionName::Slam,
    }
}

/// The gain that brings a song to the target loudness: its own, or its album's when the album
/// keeps the differences between its tracks.
pub fn matched_gain(library: &Library, song: &str, album: Option<&str>) -> f32 {
    let loudness = album
        .and_then(|key| library.album_loudness(key))
        .or_else(|| library.loudness(song));
    match loudness {
        Some(loudness) => {
            let peak = library
                .listening(song)
                .map_or(1.0, |listening| listening.peak.max(1e-4));
            (TARGET_LUFS - loudness).min(-20.0 * peak.log10() + PEAK_ROOM_DB)
        }
        None => library.replay_gain(song).map_or(UNKNOWN_GAIN_DB, |gain| {
            gain + TARGET_LUFS - REPLAY_GAIN_REFERENCE
        }),
    }
}
