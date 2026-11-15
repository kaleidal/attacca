use std::f32::consts::FRAC_1_SQRT_2;

use super::svf::{Shape, Svf};

const BUTTERWORTH_Q: [&[f32]; 4] = [
    &[FRAC_1_SQRT_2],
    &[0.5412, 1.3066],
    &[0.5176, FRAC_1_SQRT_2, 1.9319],
    &[0.5098, 0.6013, 0.9000, 2.5629],
];
pub const OPEN_LOW: f32 = 15.0;
pub const OPEN_HIGH: f32 = 21000.0;
const ENGAGE_STEP: f32 = 1.0 / 96.0;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Cut {
    pub hz: f32,
    /// Steepness in decibels per octave: 12, 24, 36 or 48.
    pub slope: u8,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct FilterSettings {
    pub highpass: Option<Cut>,
    pub lowpass: Option<Cut>,
}

#[derive(Clone)]
/// A Butterworth cut of 12 to 48 dB per octave whose cutoff glides in octaves, so sweeps and
/// slider moves stay smooth. Fully open, it steps out of the signal, crossfading on the way
/// out and back in.
pub struct SlopeFilter {
    shape: Shape,
    stages: [Svf; 4],
    count: usize,
    hz: f32,
    target_hz: f32,
    open: bool,
    wet: f32,
}

impl SlopeFilter {
    pub fn new(shape: Shape) -> Self {
        let open_hz = if shape == Shape::Lowpass {
            OPEN_HIGH
        } else {
            OPEN_LOW
        };
        Self {
            shape,
            stages: [Svf::default(); 4],
            count: 1,
            hz: open_hz,
            target_hz: open_hz,
            open: true,
            wet: 0.0,
        }
    }

    fn open_hz(&self) -> f32 {
        if self.shape == Shape::Lowpass {
            OPEN_HIGH
        } else {
            OPEN_LOW
        }
    }

    pub fn set(&mut self, cut: Option<Cut>) {
        match cut {
            Some(cut) => {
                let count = usize::from(cut.slope / 12).clamp(1, 4);
                if count != self.count {
                    self.count = count;
                    self.stages.iter_mut().for_each(Svf::reset);
                    self.apply();
                }
                self.target_hz = cut.hz;
            }
            None => self.target_hz = self.open_hz(),
        }
    }

    pub fn set_hz(&mut self, hz: f32) {
        self.target_hz = hz;
    }

    pub fn jump(&mut self) {
        self.hz = self.target_hz;
        self.apply();
    }

    fn apply(&mut self) {
        let qs = BUTTERWORTH_Q[self.count - 1];
        for (stage, q) in self.stages.iter_mut().zip(qs) {
            stage.set(self.shape, self.hz, *q, 0.0);
        }
    }

    pub fn update(&mut self) {
        if self.hz != self.target_hz {
            let octaves = (self.target_hz / self.hz).log2().clamp(-0.08, 0.08);
            self.hz = if octaves.abs() < 0.001 {
                self.target_hz
            } else {
                self.hz * octaves.exp2()
            };
            self.apply();
        }
        let open = match self.shape {
            Shape::Lowpass => self.hz >= OPEN_HIGH * 0.95,
            _ => self.hz <= OPEN_LOW * 1.05,
        };
        if self.open && !open && self.wet == 0.0 {
            self.stages.iter_mut().for_each(Svf::reset);
        }
        self.open = open;
    }

    #[inline]
    pub fn process(&mut self, frame: [f32; 2]) -> [f32; 2] {
        let target = if self.open { 0.0 } else { 1.0 };
        if self.wet == 0.0 && target == 0.0 {
            return frame;
        }
        self.wet =
            (self.wet + (target - self.wet).clamp(-ENGAGE_STEP, ENGAGE_STEP)).clamp(0.0, 1.0);
        let mut filtered = frame;
        for stage in &mut self.stages[..self.count] {
            filtered = stage.process(filtered);
        }
        let dry = 1.0 - self.wet;
        [
            frame[0] * dry + filtered[0] * self.wet,
            frame[1] * dry + filtered[1] * self.wet,
        ]
    }
}

#[derive(Clone)]
/// Splits a signal into bass and the rest with a Linkwitz-Riley crossover, whose two halves sum
/// back flat.
pub struct Crossover {
    low: [Svf; 2],
    high: [Svf; 2],
}

pub const CROSSOVER_HZ: f32 = 180.0;

impl Default for Crossover {
    fn default() -> Self {
        Self {
            low: [Svf::new(Shape::Lowpass, CROSSOVER_HZ, FRAC_1_SQRT_2, 0.0); 2],
            high: [Svf::new(Shape::Highpass, CROSSOVER_HZ, FRAC_1_SQRT_2, 0.0); 2],
        }
    }
}

impl Crossover {
    pub fn reset(&mut self) {
        self.low
            .iter_mut()
            .chain(&mut self.high)
            .for_each(Svf::reset);
    }

    #[inline]
    pub fn split(&mut self, frame: [f32; 2]) -> ([f32; 2], [f32; 2]) {
        let low = self.low[0].process(frame);
        let low = self.low[1].process(low);
        let high = self.high[0].process(frame);
        let high = self.high[1].process(high);
        (low, high)
    }
}
