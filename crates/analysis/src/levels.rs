use ebur128::{EbuR128, Mode};

pub const WAVEFORM_RATE: u32 = 50;
const SILENCE_DB: f32 = -50.0;
const LONG_FADE_SECONDS: f32 = 4.0;

pub struct Measured {
    pub loudness: f32,
    pub peak: f32,
    pub sound_start: f32,
    pub sound_end: f32,
    pub fade_start: Option<f32>,
    pub waveform: Vec<u8>,
}

/// Loudness, silence, fades and the waveform, all from one pass over the samples.
pub struct Levels {
    meter: EbuR128,
    interleaved: Vec<f32>,
    bucket: usize,
    bucket_peak: f32,
    bucket_power: f64,
    bucket_count: usize,
    waveform: Vec<u8>,
    tens: Vec<f32>,
    ten: usize,
    ten_power: f64,
    ten_count: usize,
    peak: f32,
}

fn db(power: f64) -> f32 {
    (10.0 * power.max(1e-12).log10()) as f32
}

impl Levels {
    pub fn new(rate: u32) -> Self {
        Self {
            meter: EbuR128::new(2, rate, Mode::I)
                .expect("stereo at a decodable rate is measurable"),
            interleaved: Vec::new(),
            bucket: (rate / WAVEFORM_RATE) as usize,
            bucket_peak: 0.0,
            bucket_power: 0.0,
            bucket_count: 0,
            waveform: Vec::new(),
            tens: Vec::new(),
            ten: (rate / 100) as usize,
            ten_power: 0.0,
            ten_count: 0,
            peak: 0.0,
        }
    }

    pub fn push(&mut self, frames: &[[f32; 2]]) {
        self.interleaved.clear();
        self.interleaved.extend(frames.iter().flatten());
        let _ = self.meter.add_frames_f32(&self.interleaved);
        for frame in frames {
            let level = frame[0].abs().max(frame[1].abs());
            let power = f64::from(frame[0] * frame[0] + frame[1] * frame[1]) * 0.5;
            self.peak = self.peak.max(level);
            self.bucket_peak = self.bucket_peak.max(level);
            self.bucket_power += power;
            self.bucket_count += 1;
            if self.bucket_count == self.bucket {
                let rms = (self.bucket_power / self.bucket as f64).sqrt() as f32;
                self.waveform
                    .push((self.bucket_peak.min(1.0) * 255.0) as u8);
                self.waveform.push((rms.min(1.0) * 255.0) as u8);
                self.bucket_peak = 0.0;
                self.bucket_power = 0.0;
                self.bucket_count = 0;
            }
            self.ten_power += power;
            self.ten_count += 1;
            if self.ten_count == self.ten {
                self.tens.push(db(self.ten_power / self.ten as f64));
                self.ten_power = 0.0;
                self.ten_count = 0;
            }
        }
    }

    pub fn finish(self) -> Measured {
        let loudness = self.meter.loudness_global().unwrap_or(-70.0) as f32;
        let first = self
            .tens
            .iter()
            .position(|level| *level > SILENCE_DB)
            .unwrap_or(0);
        let last = self
            .tens
            .iter()
            .rposition(|level| *level > SILENCE_DB)
            .map_or(self.tens.len(), |last| last + 1);
        let sound_start = first as f32 / 100.0;
        let sound_end = last as f32 / 100.0;
        Measured {
            loudness: if loudness.is_finite() {
                loudness
            } else {
                -70.0
            },
            peak: self.peak,
            sound_start,
            sound_end,
            fade_start: fade_start(&self.tens[first..last.max(first)])
                .map(|offset| sound_start + offset),
            waveform: self.waveform,
        }
    }
}

/// Finds a closing fade: the level stays near the song's body until a point, then falls away
/// for several seconds to the end.
fn fade_start(tens: &[f32]) -> Option<f32> {
    let seconds: Vec<f32> = tens
        .chunks(100)
        .map(|chunk| {
            db(chunk
                .iter()
                .map(|level| f64::from(10f32.powf(level / 10.0)))
                .sum::<f64>()
                / chunk.len() as f64)
        })
        .collect();
    let windows: Vec<f32> = seconds
        .windows(2)
        .map(|pair| (pair[0] + pair[1]) * 0.5)
        .collect();
    if windows.len() < 20 {
        return None;
    }
    let mut sorted = windows.clone();
    sorted.sort_by(f32::total_cmp);
    let body = sorted[sorted.len() * 7 / 10];
    let held = windows.iter().rposition(|level| *level >= body - 3.0)?;
    let length = (windows.len() - held) as f32;
    let tail = windows[windows.len() - 1];
    (length >= LONG_FADE_SECONDS && tail < body - 12.0).then_some(held as f32 + 1.0)
}
