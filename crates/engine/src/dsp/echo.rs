use std::f32::consts::FRAC_1_SQRT_2;

use super::svf::{Shape, Svf};
use crate::RATE;

const CAPACITY: usize = RATE as usize * 2;
const FEEDBACK: f32 = 0.42;

/// A tempo-synced echo with darkening repeats, fed by the outgoing song when a transition slams.
pub struct Echo {
    buffer: Box<[[f32; 2]]>,
    write: usize,
    delay: usize,
    tone: [Svf; 2],
    active_tail: usize,
}

impl Default for Echo {
    fn default() -> Self {
        Self {
            buffer: vec![[0.0; 2]; CAPACITY].into_boxed_slice(),
            write: 0,
            delay: RATE as usize / 4,
            tone: [
                Svf::new(Shape::Lowpass, 3800.0, FRAC_1_SQRT_2, 0.0),
                Svf::new(Shape::Highpass, 220.0, FRAC_1_SQRT_2, 0.0),
            ],
            active_tail: 0,
        }
    }
}

impl Echo {
    pub fn set_delay(&mut self, frames: usize) {
        self.delay = frames.clamp(1, CAPACITY - 1);
    }

    #[inline]
    pub fn process(&mut self, send: [f32; 2]) -> [f32; 2] {
        if send != [0.0; 2] {
            self.active_tail = RATE as usize * 4;
        } else if self.active_tail == 0 {
            return [0.0; 2];
        } else {
            self.active_tail -= 1;
        }
        let read = (self.write + CAPACITY - self.delay) % CAPACITY;
        let wet = self.buffer[read];
        let darker = self.tone[0].process(wet);
        let feedback = self.tone[1].process(darker);
        self.buffer[self.write] = [
            send[0] + feedback[0] * FEEDBACK,
            send[1] + feedback[1] * FEEDBACK,
        ];
        self.write = (self.write + 1) % CAPACITY;
        wet
    }
}
