use std::sync::Arc;

use realfft::num_complex::Complex;
use realfft::{RealFftPlanner, RealToComplex};

pub const BANDS: usize = 16;
const FRAMES_PER_SECOND: u32 = 200;
const ONSET_BANDS: usize = 36;
const ONSET_FROM_HZ: f32 = 30.0;
const ONSET_TO_HZ: f32 = 8000.0;
const LOW_HZ: f32 = 180.0;
const BANDS_FROM_HZ: f32 = 150.0;
const BANDS_TO_HZ: f32 = 4000.0;

/// Onset strength over time, for finding tempo, beats and bars.
pub struct Onsets {
    pub rate: f32,
    /// Seconds between the start of an onset frame and the moment its onset is heard.
    pub latency: f32,
    pub flux: Vec<f32>,
    pub low: Vec<f32>,
    pub bands: Vec<[f32; BANDS]>,
}

/// A short-time spectrum of the song's mono sum, stepped every five milliseconds.
pub struct Spectrum {
    size: usize,
    hop: usize,
    window: Vec<f32>,
    fft: Arc<dyn RealToComplex<f32>>,
    input: Vec<f32>,
    output: Vec<Complex<f32>>,
    pending: Vec<f32>,
    levels: Vec<f32>,
    previous: [f32; ONSET_BANDS],
    onset_edges: [usize; ONSET_BANDS + 1],
    low_bands: usize,
    band_edges: [usize; BANDS + 1],
    onsets: Onsets,
}

impl Spectrum {
    pub fn new(rate: u32) -> Self {
        let size = if rate > 50_000 { 2048 } else { 1024 };
        let hop = (rate / FRAMES_PER_SECOND) as usize;
        let fft = RealFftPlanner::<f32>::new().plan_fft_forward(size);
        let bin_hz = rate as f32 / size as f32;
        let bin = |hz: f32| ((hz / bin_hz) as usize).clamp(1, size / 2);
        let band_edges = std::array::from_fn(|band| {
            let hz = BANDS_FROM_HZ * (BANDS_TO_HZ / BANDS_FROM_HZ).powf(band as f32 / BANDS as f32);
            bin(hz)
        });
        let onset_hz = |band: usize| {
            ONSET_FROM_HZ * (ONSET_TO_HZ / ONSET_FROM_HZ).powf(band as f32 / ONSET_BANDS as f32)
        };
        let onset_edges = std::array::from_fn(|band| bin(onset_hz(band)));
        let low_bands = (0..ONSET_BANDS)
            .take_while(|band| onset_hz(band + 1) <= LOW_HZ)
            .count();
        Self {
            window: (0..size)
                .map(|i| 0.5 - 0.5 * (std::f32::consts::TAU * i as f32 / size as f32).cos())
                .collect(),
            input: fft.make_input_vec(),
            output: fft.make_output_vec(),
            levels: vec![0.0; size / 2 + 1],
            previous: [0.0; ONSET_BANDS],
            onset_edges,
            low_bands,
            band_edges,
            fft,
            size,
            hop,
            pending: Vec::with_capacity(size * 4),
            onsets: Onsets {
                rate: rate as f32 / hop as f32,
                latency: (size as f32 * 0.5 + hop as f32 * 0.75) / rate as f32,
                flux: Vec::new(),
                low: Vec::new(),
                bands: Vec::new(),
            },
        }
    }

    pub fn push(&mut self, frames: &[[f32; 2]]) {
        self.pending
            .extend(frames.iter().map(|frame| (frame[0] + frame[1]) * 0.5));
        let mut start = 0;
        while self.pending.len() - start >= self.size {
            self.analyse(start);
            start += self.hop;
        }
        self.pending.drain(..start);
    }

    fn analyse(&mut self, start: usize) {
        for (i, value) in self.input.iter_mut().enumerate() {
            *value = self.pending[start + i] * self.window[i];
        }
        let _ = self.fft.process(&mut self.input, &mut self.output);
        for (level, bin) in self.levels.iter_mut().zip(&self.output) {
            *level = (1.0 + 100.0 * bin.norm()).ln();
        }
        let average = |edges: &[usize], band: usize, levels: &[f32]| {
            let (from, to) = (edges[band], edges[band + 1].max(edges[band] + 1));
            levels[from..to].iter().sum::<f32>() / (to - from) as f32
        };
        let current: [f32; ONSET_BANDS] =
            std::array::from_fn(|band| average(&self.onset_edges, band, &self.levels));
        let rises: [f32; ONSET_BANDS] =
            std::array::from_fn(|band| (current[band] - self.previous[band]).max(0.0));
        self.onsets.flux.push(rises.iter().sum());
        self.onsets.low.push(rises[..self.low_bands].iter().sum());
        self.onsets.bands.push(std::array::from_fn(|band| {
            average(&self.band_edges, band, &self.levels)
        }));
        self.previous = current;
    }

    pub fn finish(self) -> Onsets {
        self.onsets
    }
}
