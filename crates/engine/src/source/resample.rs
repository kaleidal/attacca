use rubato::audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Fft, FixedSync, Indexing, Resampler as _};

use crate::RATE;

const CHUNK: usize = 1024;

/// Converts a song to the engine rate with a fixed-ratio FFT resampler, dropping its startup
/// delay so the first output frame lines up with the first input frame.
pub struct Resampler {
    inner: Fft<f32>,
    pending: Vec<f32>,
    output: Vec<f32>,
    delay: usize,
    taken: u64,
    given: u64,
}

impl Resampler {
    pub fn new(rate: u32) -> Self {
        let inner = Fft::new(rate as usize, RATE as usize, CHUNK, 2, FixedSync::Input)
            .expect("the engine rate and every decodable rate make a valid ratio");
        let delay = inner.output_delay();
        let output = vec![0.0; inner.output_frames_max() * 2];
        Self {
            inner,
            pending: Vec::with_capacity(CHUNK * 4),
            output,
            delay,
            taken: 0,
            given: 0,
        }
    }

    pub fn push(&mut self, frames: &[[f32; 2]], out: &mut Vec<[f32; 2]>) {
        self.taken += frames.len() as u64;
        self.pending.extend(frames.iter().flatten());
        let mut consumed = 0;
        while self.pending.len() - consumed >= CHUNK * 2 {
            self.run(consumed, None, out);
            consumed += CHUNK * 2;
        }
        self.pending.drain(..consumed);
    }

    /// Output frames owed for input already taken.
    pub fn unsent(&self) -> u64 {
        ((self.taken as f64 * self.inner.resample_ratio()).round() as u64)
            .saturating_sub(self.given)
    }

    /// Flushes the resampler, ending the output exactly where the input ends.
    pub fn finish(&mut self, out: &mut Vec<[f32; 2]>) {
        let total = (self.taken as f64 * self.inner.resample_ratio()).round() as u64;
        let remaining = self.pending.len() / 2;
        let start = out.len();
        self.pending.resize(CHUNK * 2, 0.0);
        self.run(0, Some(remaining), out);
        self.pending.fill(0.0);
        while self.given < total {
            self.run(0, Some(0), out);
        }
        let surplus = (self.given - total) as usize;
        out.truncate((out.len() - surplus).max(start));
        self.pending.clear();
    }

    fn run(&mut self, offset: usize, partial: Option<usize>, out: &mut Vec<[f32; 2]>) {
        let input = InterleavedSlice::new(&self.pending[offset..offset + CHUNK * 2], 2, CHUNK)
            .expect("chunks hold whole frames");
        let capacity = self.output.len() / 2;
        let mut output = InterleavedSlice::new_mut(&mut self.output, 2, capacity)
            .expect("the output buffer holds whole frames");
        let indexing = Indexing {
            input_offset: 0,
            output_offset: 0,
            partial_len: partial,
            active_channels_mask: None,
        };
        let (_, written) = self
            .inner
            .process_into_buffer(&input, &mut output, Some(&indexing))
            .expect("buffers are sized for the resampler");
        let skip = self.delay.min(written);
        self.delay -= skip;
        self.given += (written - skip) as u64;
        out.extend(
            self.output[skip * 2..written * 2]
                .as_chunks::<2>()
                .0
                .iter()
                .copied(),
        );
    }
}
