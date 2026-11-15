use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use realfft::RealFftPlanner;
use rtrb::Consumer;

use crate::RATE;

pub const BANDS: usize = 64;
pub const WAVE: usize = 256;
const SIZE: usize = 2048;
const FRAME: Duration = Duration::from_micros(16_667);
const FLOOR_DB: f32 = -72.0;

/// Turns the mixed output into spectrum bands and a waveform about sixty times a second, while
/// someone is watching.
pub struct Visual {
    active: Arc<AtomicBool>,
}

impl Visual {
    pub fn start(mut tap: Consumer<[f32; 2]>, on_frame: impl Fn(&[u8]) + Send + 'static) -> Self {
        let active = Arc::new(AtomicBool::new(false));
        let watching = Arc::clone(&active);
        std::thread::Builder::new()
            .name("attacca-visual".into())
            .spawn(move || {
                let mut analyser = Analyser::new();
                loop {
                    let tick = Instant::now();
                    if watching.load(Ordering::Relaxed) {
                        while let Ok(frame) = tap.pop() {
                            analyser.push(frame);
                        }
                        on_frame(&analyser.frame());
                    } else {
                        while tap.pop().is_ok() {}
                        std::thread::park_timeout(Duration::from_millis(250));
                    }
                    if tap.is_abandoned() {
                        return;
                    }
                    std::thread::sleep(FRAME.saturating_sub(tick.elapsed()));
                }
            })
            .expect("the visualizer thread starts");
        Self { active }
    }

    pub fn set_active(&self, active: bool) {
        self.active.store(active, Ordering::Relaxed);
    }
}

struct Analyser {
    ring: Vec<f32>,
    write: usize,
    window: Vec<f32>,
    input: Vec<f32>,
    spectrum: Vec<realfft::num_complex::Complex<f32>>,
    fft: Arc<dyn realfft::RealToComplex<f32>>,
    edges: [usize; BANDS + 1],
    levels: [f32; BANDS],
}

impl Analyser {
    fn new() -> Self {
        let fft = RealFftPlanner::<f32>::new().plan_fft_forward(SIZE);
        let window = (0..SIZE)
            .map(|i| 0.5 - 0.5 * (std::f32::consts::TAU * i as f32 / SIZE as f32).cos())
            .collect();
        let bin_hz = RATE as f32 / SIZE as f32;
        let edges = std::array::from_fn(|band| {
            let hz = 30.0 * (16000.0f32 / 30.0).powf(band as f32 / BANDS as f32);
            ((hz / bin_hz).round() as usize).clamp(1, SIZE / 2)
        });
        Self {
            ring: vec![0.0; SIZE],
            write: 0,
            window,
            input: fft.make_input_vec(),
            spectrum: fft.make_output_vec(),
            fft,
            edges,
            levels: [0.0; BANDS],
        }
    }

    fn push(&mut self, frame: [f32; 2]) {
        self.ring[self.write] = (frame[0] + frame[1]) * 0.5;
        self.write = (self.write + 1) % SIZE;
    }

    fn frame(&mut self) -> Vec<u8> {
        for (i, value) in self.input.iter_mut().enumerate() {
            *value = self.ring[(self.write + i) % SIZE] * self.window[i];
        }
        let _ = self.fft.process(&mut self.input, &mut self.spectrum);
        let mut bytes = Vec::with_capacity(BANDS + WAVE);
        for band in 0..BANDS {
            let (from, to) = (
                self.edges[band],
                self.edges[band + 1].max(self.edges[band] + 1),
            );
            let power = self.spectrum[from..to]
                .iter()
                .map(|bin| bin.norm_sqr())
                .fold(0.0, f32::max);
            let db = 10.0 * (power.max(1e-12) / (SIZE as f32 * 0.25).powi(2)).log10();
            let level = ((db - FLOOR_DB) / -FLOOR_DB).clamp(0.0, 1.0);
            let previous = self.levels[band];
            self.levels[band] = if level > previous {
                level
            } else {
                previous * 0.86 + level * 0.14
            };
            bytes.push((self.levels[band] * 255.0) as u8);
        }
        let step = SIZE / 2 / WAVE;
        for point in 0..WAVE {
            let sample = self.ring[(self.write + SIZE / 2 + point * step) % SIZE];
            bytes.push(((sample.clamp(-1.0, 1.0) * 127.0) as i8) as u8);
        }
        bytes
    }
}
