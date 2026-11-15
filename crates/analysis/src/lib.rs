mod fingerprint;
mod levels;
mod rhythm;
mod spectrum;

use std::path::Path;

use attacca_media::{Decoder, MediaError};

pub use fingerprint::{FINGERPRINT_SECONDS, offset as fingerprint_offset};
pub use levels::WAVEFORM_RATE;

/// Bumped whenever analysis changes, so stored results are redone.
pub const VERSION: u32 = 1;

/// What Attacca learns about a song by listening to it once.
#[derive(Clone, Debug, Default)]
pub struct Analysis {
    pub duration: f64,
    /// Integrated loudness in LUFS.
    pub loudness: f32,
    /// Highest sample, linear.
    pub peak: f32,
    pub tempo: Option<f32>,
    pub beats: Vec<f32>,
    /// Index among the first four beats of the first downbeat.
    pub downbeat: u8,
    /// Where the leading silence ends.
    pub sound_start: f32,
    /// Where the trailing silence begins.
    pub sound_end: f32,
    /// Where a long closing fade begins.
    pub fade_start: Option<f32>,
    /// Peak and RMS per bucket, `WAVEFORM_RATE` buckets a second, two bytes each.
    pub waveform: Vec<u8>,
    pub fingerprint: Vec<u32>,
}

impl Analysis {
    pub fn suggested_start(&self) -> f32 {
        (self.sound_start - 0.01).max(0.0)
    }

    pub fn suggested_end(&self) -> f32 {
        self.fade_start.unwrap_or(self.sound_end)
    }
}

/// Decodes the whole song once and runs every measurement over it.
pub fn analyse(path: &Path) -> Result<Analysis, MediaError> {
    let mut decoder = Decoder::open(path)?;
    let rate = decoder.sample_rate;
    let mut levels = levels::Levels::new(rate);
    let mut spectrum = spectrum::Spectrum::new(rate);
    let mut fingerprint = fingerprint::Fingerprint::new(rate);
    let mut frames = Vec::with_capacity(16384);
    let mut total = 0u64;
    loop {
        frames.clear();
        let more = decoder.decode_into(&mut frames)?;
        total += frames.len() as u64;
        levels.push(&frames);
        spectrum.push(&frames);
        fingerprint.push(&frames);
        if !more {
            break;
        }
    }
    let duration = total as f64 / f64::from(rate);
    let measured = levels.finish();
    let onsets = spectrum.finish();
    let rhythm = rhythm::track(&onsets);
    Ok(Analysis {
        duration,
        loudness: measured.loudness,
        peak: measured.peak,
        tempo: rhythm.as_ref().map(|rhythm| rhythm.tempo),
        downbeat: rhythm.as_ref().map_or(0, |rhythm| rhythm.downbeat),
        beats: rhythm.map(|rhythm| rhythm.beats).unwrap_or_default(),
        sound_start: measured.sound_start,
        sound_end: measured.sound_end.min(duration as f32),
        fade_start: measured.fade_start,
        waveform: measured.waveform,
        fingerprint: fingerprint.finish(),
    })
}
