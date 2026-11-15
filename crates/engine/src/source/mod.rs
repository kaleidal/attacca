mod resample;

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use attacca_media::{Decoder, MediaError};
use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use parking_lot::Mutex;
use rtrb::{Consumer, Producer, RingBuffer};

use crate::RATE;
use resample::Resampler;

const AHEAD: usize = RATE as usize * 4;
const POLL: Duration = Duration::from_millis(15);

/// Who gets this song's resampler when its input runs out, so the next song continues the same
/// resampled stream and a gapless join stays sample-continuous.
enum Successor {
    Open,
    Chained(Sender<Resampler>),
    Closed,
}

pub struct SourceStatus {
    finished: AtomicBool,
    rate: u32,
    successor: Mutex<Successor>,
    skew: AtomicU64,
}

impl SourceStatus {
    pub fn finished(&self) -> bool {
        self.finished.load(Ordering::Acquire)
    }

    /// Frames of the previous song at the start of this stream.
    pub fn skew(&self) -> u64 {
        self.skew.load(Ordering::Acquire)
    }
}

/// The playing side of a decoded song: frames at the engine rate, starting at song frame
/// `origin`.
pub struct Source {
    pub frames: Consumer<[f32; 2]>,
    pub status: Arc<SourceStatus>,
    pub origin: u64,
}

impl Source {
    /// Starts decoding `path` from song frame `origin` on its own thread.
    pub fn open(path: &Path, origin: u64) -> Result<Source, MediaError> {
        let decoder = Decoder::open(path)?;
        Self::spawn(path, decoder, origin, None)
    }

    /// Starts decoding `path` from its beginning as the gapless continuation of `previous`.
    pub fn open_following(path: &Path, previous: &SourceStatus) -> Result<Source, MediaError> {
        let decoder = Decoder::open(path)?;
        let mut carry = None;
        if decoder.sample_rate == previous.rate && decoder.sample_rate != RATE {
            let mut successor = previous.successor.lock();
            if matches!(*successor, Successor::Open) {
                let (sender, receiver) = crossbeam_channel::bounded(1);
                *successor = Successor::Chained(sender);
                carry = Some(receiver);
            }
        }
        Self::spawn(path, decoder, 0, carry)
    }

    fn spawn(
        path: &Path,
        decoder: Decoder,
        origin: u64,
        carry: Option<Receiver<Resampler>>,
    ) -> Result<Source, MediaError> {
        let (producer, frames) = RingBuffer::new(AHEAD);
        let status = Arc::new(SourceStatus {
            finished: AtomicBool::new(false),
            rate: decoder.sample_rate,
            successor: Mutex::new(Successor::Open),
            skew: AtomicU64::new(0),
        });
        let thread_status = Arc::clone(&status);
        let path = path.to_path_buf();
        std::thread::Builder::new()
            .name("attacca-decode".into())
            .spawn(move || {
                let mut stream = Stream {
                    decoder,
                    producer,
                    status: &thread_status,
                };
                if let Err(error) = stream.run(origin, carry) {
                    eprintln!("attacca: decoding {} stopped: {error}", path.display());
                }
                thread_status.finished.store(true, Ordering::Release);
            })
            .map_err(MediaError::from)?;
        Ok(Source {
            frames,
            status,
            origin,
        })
    }
}

struct Stream<'a> {
    decoder: Decoder,
    producer: Producer<[f32; 2]>,
    status: &'a SourceStatus,
}

impl Stream<'_> {
    fn run(&mut self, origin: u64, carry: Option<Receiver<Resampler>>) -> Result<(), MediaError> {
        let rate = self.decoder.sample_rate;
        let first = (origin as f64 * f64::from(rate) / f64::from(RATE)).round() as u64;
        self.decoder.seek_frame(first)?;
        let mut resampler = match carry {
            Some(carry) => match self.receive(carry) {
                Some(Some(resampler)) => Some(resampler),
                Some(None) => (rate != RATE).then(|| Resampler::new(rate)),
                None => return Ok(()),
            },
            None => (rate != RATE).then(|| Resampler::new(rate)),
        };
        if let Some(resampler) = &resampler {
            self.status
                .skew
                .store(resampler.unsent(), Ordering::Release);
        }
        let mut decoded = Vec::with_capacity(8192);
        let mut output = Vec::with_capacity(16384);
        loop {
            decoded.clear();
            let more = self.decoder.decode_into(&mut decoded)?;
            output.clear();
            match &mut resampler {
                Some(resampler) => resampler.push(&decoded, &mut output),
                None => output.extend_from_slice(&decoded),
            }
            if !more {
                if let Some(mut unclaimed) = resampler
                    .take()
                    .and_then(|resampler| self.hand_over(resampler))
                {
                    unclaimed.finish(&mut output);
                }
                *self.status.successor.lock() = Successor::Closed;
            }
            if !push_all(&mut self.producer, &output) || !more {
                return Ok(());
            }
        }
    }

    /// Waits for the previous song's resampler. `None` means the deck let go meanwhile;
    /// `Some(None)` means the previous song will not hand one over.
    fn receive(&self, carry: Receiver<Resampler>) -> Option<Option<Resampler>> {
        loop {
            match carry.recv_timeout(POLL) {
                Ok(resampler) => return Some(Some(resampler)),
                Err(RecvTimeoutError::Disconnected) => return Some(None),
                Err(RecvTimeoutError::Timeout) if self.producer.is_abandoned() => return None,
                Err(RecvTimeoutError::Timeout) => {}
            }
        }
    }

    /// Gives the resampler, with this song's last frames still inside it, to the next song.
    /// Returns it when no song takes it.
    fn hand_over(&self, resampler: Resampler) -> Option<Resampler> {
        let mut successor = self.status.successor.lock();
        match std::mem::replace(&mut *successor, Successor::Closed) {
            Successor::Chained(sender) => {
                sender.send(resampler).err().map(|error| error.into_inner())
            }
            _ => Some(resampler),
        }
    }
}

/// Waits for room while the deck is still listening. Returns `false` once the deck let go.
fn push_all(producer: &mut Producer<[f32; 2]>, mut frames: &[[f32; 2]]) -> bool {
    while !frames.is_empty() {
        if producer.is_abandoned() {
            return false;
        }
        let (_, rest) = producer.push_partial_slice(frames);
        if rest.len() == frames.len() {
            std::thread::sleep(POLL);
        }
        frames = rest;
    }
    true
}

/// A song decoded entirely into memory, for looping a short stretch while editing.
pub fn decode_clip(path: &Path, from: u64, frames: u64) -> Result<Vec<[f32; 2]>, MediaError> {
    let mut source = Source::open(path, from)?;
    let mut clip = Vec::with_capacity(frames as usize);
    while (clip.len() as u64) < frames {
        match source.frames.pop() {
            Ok(frame) => clip.push(frame),
            Err(_) if source.status.finished() && source.frames.is_empty() => break,
            Err(_) => std::thread::sleep(Duration::from_millis(2)),
        }
    }
    Ok(clip)
}
