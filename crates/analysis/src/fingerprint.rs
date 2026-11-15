use rusty_chromaprint::{Configuration, Fingerprinter, match_fingerprints};

pub const FINGERPRINT_SECONDS: u32 = 120;

fn configuration() -> Configuration {
    Configuration::preset_test2()
}

/// An acoustic fingerprint of the song's opening, the same for any copy of the same recording.
pub struct Fingerprint {
    inner: Fingerprinter,
    remaining: u64,
    samples: Vec<i16>,
}

impl Fingerprint {
    pub fn new(rate: u32) -> Self {
        let mut inner = Fingerprinter::new(&configuration());
        inner
            .start(rate, 2)
            .expect("stereo at a decodable rate can be fingerprinted");
        Self {
            inner,
            remaining: u64::from(rate * FINGERPRINT_SECONDS),
            samples: Vec::new(),
        }
    }

    pub fn push(&mut self, frames: &[[f32; 2]]) {
        let take = (frames.len() as u64).min(self.remaining) as usize;
        if take == 0 {
            return;
        }
        self.remaining -= take as u64;
        self.samples.clear();
        self.samples.extend(
            frames[..take]
                .iter()
                .flatten()
                .map(|sample| (sample.clamp(-1.0, 1.0) * 32767.0) as i16),
        );
        self.inner.consume(&self.samples);
    }

    pub fn finish(mut self) -> Vec<u32> {
        self.inner.finish();
        self.inner.fingerprint().to_vec()
    }
}

/// How far into `theirs` the same recording as `ours` starts, in seconds, when the two match.
pub fn offset(ours: &[u32], theirs: &[u32]) -> Option<f32> {
    let config = configuration();
    let segments = match_fingerprints(ours, theirs, &config).ok()?;
    let best = segments
        .iter()
        .filter(|segment| segment.duration(&config) >= 10.0)
        .max_by(|a, b| a.duration(&config).total_cmp(&b.duration(&config)))?;
    Some(best.start2(&config) - best.start1(&config))
}
