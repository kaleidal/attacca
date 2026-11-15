use crate::RATE;

/// Simper's trapezoidal state-variable filter. Stays stable while its cutoff moves, which the
/// transitions rely on when they sweep filters.
#[derive(Clone, Copy, Default)]
pub struct Svf {
    a1: f32,
    a2: f32,
    a3: f32,
    m0: f32,
    m1: f32,
    m2: f32,
    state: [[f32; 2]; 2],
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Shape {
    Lowpass,
    Highpass,
    Bell,
    LowShelf,
    HighShelf,
}

fn prewarp(hz: f32) -> f32 {
    (std::f32::consts::PI * hz.clamp(10.0, RATE as f32 * 0.49) / RATE as f32).tan()
}

impl Svf {
    pub fn new(shape: Shape, hz: f32, q: f32, gain_db: f32) -> Self {
        let mut filter = Self::default();
        filter.set(shape, hz, q, gain_db);
        filter
    }

    pub fn set(&mut self, shape: Shape, hz: f32, q: f32, gain_db: f32) {
        let a = 10f32.powf(gain_db / 40.0);
        let (g, k, m0, m1, m2) = match shape {
            Shape::Lowpass => (prewarp(hz), 1.0 / q, 0.0, 0.0, 1.0),
            Shape::Highpass => {
                let k = 1.0 / q;
                (prewarp(hz), k, 1.0, -k, -1.0)
            }
            Shape::Bell => {
                let k = 1.0 / (q * a);
                (prewarp(hz), k, 1.0, k * (a * a - 1.0), 0.0)
            }
            Shape::LowShelf => {
                let k = 1.0 / q;
                (prewarp(hz) / a.sqrt(), k, 1.0, k * (a - 1.0), a * a - 1.0)
            }
            Shape::HighShelf => {
                let k = 1.0 / q;
                (
                    prewarp(hz) * a.sqrt(),
                    k,
                    a * a,
                    k * (1.0 - a) * a,
                    1.0 - a * a,
                )
            }
        };
        self.a1 = 1.0 / (1.0 + g * (g + k));
        self.a2 = g * self.a1;
        self.a3 = g * self.a2;
        self.m0 = m0;
        self.m1 = m1;
        self.m2 = m2;
    }

    pub fn reset(&mut self) {
        self.state = [[0.0; 2]; 2];
    }

    #[inline]
    pub fn process(&mut self, frame: [f32; 2]) -> [f32; 2] {
        let mut out = [0.0; 2];
        for channel in 0..2 {
            let [ic1, ic2] = &mut self.state[channel];
            let v0 = frame[channel];
            let v3 = v0 - *ic2;
            let v1 = self.a1 * *ic1 + self.a2 * v3;
            let v2 = *ic2 + self.a2 * *ic1 + self.a3 * v3;
            *ic1 = 2.0 * v1 - *ic1;
            *ic2 = 2.0 * v2 - *ic2;
            out[channel] = self.m0 * v0 + self.m1 * v1 + self.m2 * v2;
        }
        out
    }
}
