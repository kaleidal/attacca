use crate::mixer::history::History;

const SIZE: usize = 2048;
const HOP: usize = SIZE / 2;
const SEARCH: i64 = 384;
const SPAN: usize = HOP + 2 * SEARCH as usize;

/// Time-stretching by waveform-similarity overlap-add: each new grain is placed where it best
/// continues the previous one, which keeps pitch and transients intact for small tempo changes.
pub struct Stretcher {
    window: Box<[f32]>,
    tail: Box<[[f32; 2]]>,
    hop: Box<[[f32; 2]]>,
    reference: Box<[f32]>,
    candidates: Box<[f32]>,
    hop_index: usize,
    natural: i64,
    nominal: f64,
    grain: i64,
}

impl Default for Stretcher {
    fn default() -> Self {
        let window = (0..SIZE)
            .map(|i| {
                let phase = std::f32::consts::TAU * i as f32 / SIZE as f32;
                0.5 - 0.5 * phase.cos()
            })
            .collect();
        Self {
            window,
            tail: vec![[0.0; 2]; HOP].into_boxed_slice(),
            hop: vec![[0.0; 2]; HOP].into_boxed_slice(),
            reference: vec![0.0; HOP].into_boxed_slice(),
            candidates: vec![0.0; SPAN].into_boxed_slice(),
            hop_index: HOP,
            natural: 0,
            nominal: 0.0,
            grain: 0,
        }
    }
}

pub const LOOKAHEAD: u64 = (SIZE as i64 + SEARCH) as u64 + 2;

impl Stretcher {
    /// Starts at a song frame. At a ratio of one the output equals the input sample for sample,
    /// so switching from direct playback is seamless.
    pub fn start(&mut self, history: &History, position: i64) {
        for (k, tail) in self.tail.iter_mut().enumerate() {
            let frame = history.at(position + k as i64);
            let w = self.window[HOP + k];
            *tail = [frame[0] * w, frame[1] * w];
        }
        self.natural = position;
        self.nominal = position as f64;
        self.grain = position - HOP as i64;
        self.hop_index = HOP;
    }

    pub fn needed_until(&self) -> u64 {
        (self.nominal.max(0.0) as u64) + LOOKAHEAD
    }

    pub fn position(&self) -> f64 {
        (self.grain + self.hop_index as i64) as f64
    }

    #[inline]
    pub fn next(&mut self, history: &History, ratio: f32) -> [f32; 2] {
        if self.hop_index == HOP {
            self.advance(history, ratio);
        }
        let frame = self.hop[self.hop_index];
        self.hop_index += 1;
        frame
    }

    fn advance(&mut self, history: &History, ratio: f32) {
        let target = self.nominal.round() as i64;
        let grain = target + self.best_offset(history, target);
        for k in 0..HOP {
            let frame = history.at(grain + k as i64);
            let w = self.window[k];
            self.hop[k] = [
                self.tail[k][0] + frame[0] * w,
                self.tail[k][1] + frame[1] * w,
            ];
        }
        for k in 0..HOP {
            let frame = history.at(grain + (HOP + k) as i64);
            let w = self.window[HOP + k];
            self.tail[k] = [frame[0] * w, frame[1] * w];
        }
        self.natural = grain + HOP as i64;
        self.grain = grain;
        self.nominal += HOP as f64 * f64::from(ratio);
        self.hop_index = 0;
    }

    fn best_offset(&mut self, history: &History, target: i64) -> i64 {
        for (k, value) in self.reference.iter_mut().enumerate() {
            let frame = history.at(self.natural + k as i64);
            *value = frame[0] + frame[1];
        }
        let first = target - SEARCH;
        for (k, value) in self.candidates.iter_mut().enumerate() {
            let frame = history.at(first + k as i64);
            *value = frame[0] + frame[1];
        }
        let coarse = self.scan(-SEARCH, SEARCH, 8, 4);
        self.scan((coarse - 7).max(-SEARCH), (coarse + 7).min(SEARCH), 1, 2)
    }

    fn scan(&self, from: i64, to: i64, step: usize, stride: usize) -> i64 {
        let mut best = (f32::MIN, 0);
        for offset in (from..=to).step_by(step) {
            let base = (offset + SEARCH) as usize;
            let mut correlation = 0.0;
            let mut energy = 1e-9;
            for k in (0..HOP).step_by(stride) {
                let candidate = self.candidates[base + k];
                correlation += self.reference[k] * candidate;
                energy += candidate * candidate;
            }
            let score = correlation / energy.sqrt();
            if score > best.0 {
                best = (score, offset);
            }
        }
        best.1
    }
}
