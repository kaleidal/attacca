use super::svf::{Shape, Svf};

pub const BAND_HZ: [f32; 10] = [
    31.0, 62.0, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0,
];
const BAND_Q: f32 = 1.1;
const STEP_DB: f32 = 0.25;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct EqCurve {
    pub preamp: f32,
    pub gains: [f32; 10],
}

impl EqCurve {
    pub fn is_flat(&self) -> bool {
        self.preamp == 0.0 && self.gains.iter().all(|gain| *gain == 0.0)
    }
}

fn shape(band: usize) -> Shape {
    match band {
        0 => Shape::LowShelf,
        9 => Shape::HighShelf,
        _ => Shape::Bell,
    }
}

fn approach(current: f32, target: f32) -> f32 {
    current + (target - current).clamp(-STEP_DB, STEP_DB)
}

#[derive(Clone)]
/// Ten bands and a preamp. Gains glide toward a new curve a quarter decibel per block, so moving
/// a slider never clicks.
pub struct Equalizer {
    target: EqCurve,
    current: EqCurve,
    bands: [Svf; 10],
}

impl Default for Equalizer {
    fn default() -> Self {
        Self {
            target: EqCurve::default(),
            current: EqCurve::default(),
            bands: std::array::from_fn(|band| Svf::new(shape(band), BAND_HZ[band], BAND_Q, 0.0)),
        }
    }
}

impl Equalizer {
    pub fn set(&mut self, curve: EqCurve) {
        self.target = curve;
    }

    pub fn reset(&mut self) {
        self.current = self.target;
        for (band, filter) in self.bands.iter_mut().enumerate() {
            filter.set(shape(band), BAND_HZ[band], BAND_Q, self.current.gains[band]);
            filter.reset();
        }
    }

    pub fn preamp(&self) -> f32 {
        10f32.powf(self.current.preamp / 20.0)
    }

    /// Advances the glide by one block and returns the preamp gain for it.
    pub fn update(&mut self) -> f32 {
        self.current.preamp = approach(self.current.preamp, self.target.preamp);
        let gains = self.current.gains.iter_mut().zip(self.target.gains);
        for (band, ((gain, target), filter)) in gains.zip(&mut self.bands).enumerate() {
            if *gain != target {
                if *gain == 0.0 {
                    filter.reset();
                }
                *gain = approach(*gain, target);
                filter.set(shape(band), BAND_HZ[band], BAND_Q, *gain);
            }
        }
        self.preamp()
    }

    #[inline]
    pub fn process(&mut self, mut frame: [f32; 2]) -> [f32; 2] {
        for band in 0..10 {
            if self.current.gains[band] != 0.0 {
                frame = self.bands[band].process(frame);
            }
        }
        frame
    }
}
