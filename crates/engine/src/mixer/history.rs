use rtrb::Consumer;

const CAPACITY: usize = 1 << 16;
const MASK: usize = CAPACITY - 1;

/// The recent past and near future of a deck's song, indexed by song frame, so the varispeed
/// and the time-stretcher can read around the play position.
pub struct History {
    frames: Box<[[f32; 2]]>,
    start: u64,
    end: u64,
}

impl Default for History {
    fn default() -> Self {
        Self {
            frames: vec![[0.0; 2]; CAPACITY].into_boxed_slice(),
            start: 0,
            end: 0,
        }
    }
}

impl History {
    pub fn reset(&mut self, origin: u64) {
        self.start = origin;
        self.end = origin;
    }

    pub fn end(&self) -> u64 {
        self.end
    }

    /// Pulls decoded frames until `upto` is available or the decoder has nothing more yet.
    pub fn fill(&mut self, source: &mut Consumer<[f32; 2]>, upto: u64) {
        while self.end < upto {
            let wanted = ((upto - self.end) as usize).min(source.slots());
            if wanted == 0 {
                return;
            }
            let Ok(chunk) = source.read_chunk(wanted) else {
                return;
            };
            let (first, second) = chunk.as_slices();
            for frame in first.iter().chain(second) {
                self.frames[self.end as usize & MASK] = *frame;
                self.end += 1;
            }
            chunk.commit_all();
            self.start = self.start.max(self.end.saturating_sub(CAPACITY as u64));
        }
    }

    #[inline]
    pub fn at(&self, index: i64) -> [f32; 2] {
        if index < self.start as i64 || index >= self.end as i64 {
            return [0.0; 2];
        }
        self.frames[index as usize & MASK]
    }

    /// Cubic Hermite interpolation between frames.
    #[inline]
    pub fn interpolate(&self, position: f64) -> [f32; 2] {
        let base = position.floor();
        let t = (position - base) as f32;
        let i = base as i64;
        let (a, b, c, d) = (self.at(i - 1), self.at(i), self.at(i + 1), self.at(i + 2));
        let mut out = [0.0; 2];
        for channel in 0..2 {
            let c1 = 0.5 * (c[channel] - a[channel]);
            let c2 = a[channel] - 2.5 * b[channel] + 2.0 * c[channel] - 0.5 * d[channel];
            let c3 = 0.5 * (d[channel] - a[channel]) + 1.5 * (b[channel] - c[channel]);
            out[channel] = ((c3 * t + c2) * t + c1) * t + b[channel];
        }
        out
    }
}
