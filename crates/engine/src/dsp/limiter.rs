use crate::RATE;

const LOOKAHEAD: usize = (RATE as usize) / 200;
const CEILING: f32 = 0.891;
const RELEASE_SECONDS: f32 = 0.12;

/// A lookahead brickwall limiter. The gain it needs is held across the lookahead window and then
/// averaged over the same window, which reaches the needed gain exactly when the peak arrives
/// and never moves faster than the window allows.
pub struct Limiter {
    delay: [[f32; 2]; LOOKAHEAD],
    gains: [f32; LOOKAHEAD + 1],
    minima: [usize; LOOKAHEAD + 1],
    minima_head: usize,
    minima_len: usize,
    average: [f32; LOOKAHEAD],
    average_sum: f64,
    index: usize,
    count: usize,
    envelope: f32,
    release: f32,
}

impl Default for Limiter {
    fn default() -> Self {
        Self {
            delay: [[0.0; 2]; LOOKAHEAD],
            gains: [1.0; LOOKAHEAD + 1],
            minima: [0; LOOKAHEAD + 1],
            minima_head: 0,
            minima_len: 0,
            average: [1.0; LOOKAHEAD],
            average_sum: LOOKAHEAD as f64,
            index: 0,
            count: 0,
            envelope: 1.0,
            release: 1.0 - (-1.0 / (RELEASE_SECONDS * RATE as f32)).exp(),
        }
    }
}

impl Limiter {
    fn window_min(&mut self, gain: f32) -> f32 {
        if self.minima_len > 0 && self.count - self.minima[self.minima_head] > LOOKAHEAD {
            self.minima_head = (self.minima_head + 1) % (LOOKAHEAD + 1);
            self.minima_len -= 1;
        }
        let slot = self.count % (LOOKAHEAD + 1);
        self.gains[slot] = gain;
        while self.minima_len > 0 {
            let last = (self.minima_head + self.minima_len - 1) % (LOOKAHEAD + 1);
            if self.gains[self.minima[last] % (LOOKAHEAD + 1)] > gain {
                self.minima_len -= 1;
            } else {
                break;
            }
        }
        let tail = (self.minima_head + self.minima_len) % (LOOKAHEAD + 1);
        self.minima[tail] = self.count;
        self.minima_len += 1;
        self.count += 1;
        self.gains[self.minima[self.minima_head] % (LOOKAHEAD + 1)]
    }

    #[inline]
    pub fn process(&mut self, frame: [f32; 2]) -> [f32; 2] {
        let peak = frame[0].abs().max(frame[1].abs());
        let needed = if peak > CEILING { CEILING / peak } else { 1.0 };
        let held = self.window_min(needed);
        self.envelope = if held < self.envelope {
            held
        } else {
            self.envelope + (held - self.envelope) * self.release
        };
        self.average_sum += f64::from(self.envelope - self.average[self.index]);
        self.average[self.index] = self.envelope;
        let gain = (self.average_sum / LOOKAHEAD as f64) as f32;
        let delayed = self.delay[self.index];
        self.delay[self.index] = frame;
        self.index = (self.index + 1) % LOOKAHEAD;
        [delayed[0] * gain, delayed[1] * gain]
    }

    /// Decibels the limiter currently takes off.
    pub fn reduction_db(&self) -> f32 {
        let gain = (self.average_sum / LOOKAHEAD as f64) as f32;
        -20.0 * gain.max(1e-6).log10()
    }
}

#[cfg(test)]
mod tests {
    use super::{CEILING, Limiter};

    #[test]
    fn peaks_never_pass_the_ceiling() {
        let mut limiter = Limiter::default();
        let mut loudest = 0.0f32;
        for i in 0..48_000 {
            let burst = if (i / 3_000) % 2 == 0 { 3.5 } else { 0.4 };
            let sample = burst * (i as f32 * 0.071).sin();
            let out = limiter.process([sample, -sample]);
            loudest = loudest.max(out[0].abs()).max(out[1].abs());
        }
        assert!(loudest <= CEILING + 1e-4, "peak {loudest}");
    }
}
