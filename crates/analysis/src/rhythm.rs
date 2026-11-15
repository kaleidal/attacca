use crate::spectrum::{BANDS, Onsets};

const SLOWEST: f32 = 60.0;
const FASTEST: f32 = 200.0;
const PREFERRED: f32 = 120.0;
const TEMPO_WINDOW_SECONDS: f32 = 240.0;
const CONFIDENCE: f32 = 1.35;
const TIGHTNESS: f32 = 60.0;
const STEADY_SECONDS: f32 = 0.02;

pub struct Rhythm {
    pub tempo: f32,
    pub beats: Vec<f32>,
    pub downbeat: u8,
}

/// Finds the tempo, the beats and where bars begin.
pub fn track(onsets: &Onsets) -> Option<Rhythm> {
    let broad = normalise(&onsets.flux, onsets.rate);
    let low = normalise(&onsets.low, onsets.rate);
    let envelope: Vec<f32> = broad
        .iter()
        .zip(&low)
        .map(|(broad, low)| broad + 0.6 * low)
        .collect();
    if envelope.len() < (onsets.rate * 10.0) as usize {
        return None;
    }
    let period = estimate_period(&envelope, onsets.rate)?;
    let frames = follow_beats(&envelope, period);
    if frames.len() < 8 {
        return None;
    }
    let (positions, period) = straighten(&frames, period, onsets.rate);
    let downbeat = find_downbeat(&positions, onsets);
    let beats = positions
        .iter()
        .map(|frame| frame / onsets.rate + onsets.latency)
        .collect();
    Some(Rhythm {
        tempo: 60.0 * onsets.rate / period,
        beats,
        downbeat,
    })
}

/// Removes the slow swell of the onset curve and scales it to unit spread.
fn normalise(flux: &[f32], rate: f32) -> Vec<f32> {
    let half = (rate * 0.2) as usize;
    let mut prefix = Vec::with_capacity(flux.len() + 1);
    prefix.push(0.0f64);
    for value in flux {
        prefix.push(prefix.last().unwrap() + f64::from(*value));
    }
    let rectified: Vec<f32> = (0..flux.len())
        .map(|i| {
            let (from, to) = (i.saturating_sub(half), (i + half + 1).min(flux.len()));
            let mean = (prefix[to] - prefix[from]) / (to - from) as f64;
            (flux[i] - mean as f32).max(0.0)
        })
        .collect();
    let spread =
        (rectified.iter().map(|value| value * value).sum::<f32>() / rectified.len() as f32).sqrt();
    if spread <= f32::EPSILON {
        return rectified;
    }
    rectified.iter().map(|value| value / spread).collect()
}

fn autocorrelation(envelope: &[f32], lag: usize) -> f32 {
    if lag >= envelope.len() {
        return 0.0;
    }
    let pairs = envelope.len() - lag;
    envelope[..pairs]
        .iter()
        .zip(&envelope[lag..])
        .map(|(a, b)| a * b)
        .sum::<f32>()
        / pairs as f32
}

/// The beat period in onset frames, from the autocorrelation of the onset curve weighted
/// toward common tempos. `None` when nothing repeats clearly enough.
fn estimate_period(envelope: &[f32], rate: f32) -> Option<f32> {
    let window = &envelope[..envelope.len().min((rate * TEMPO_WINDOW_SECONDS) as usize)];
    let shortest = (rate * 60.0 / FASTEST).floor() as usize;
    let longest = (rate * 60.0 / SLOWEST).ceil() as usize;
    let correlations: Vec<f32> = (0..=longest * 2)
        .map(|lag| autocorrelation(window, lag))
        .collect();
    let score = |lag: usize| {
        let bpm = 60.0 * rate / lag as f32;
        let prior = (-0.5 * ((bpm / PREFERRED).log2() / 0.9).powi(2)).exp();
        prior * (correlations[lag] + 0.5 * correlations[lag * 2])
    };
    let scores: Vec<f32> = (shortest..=longest).map(score).collect();
    let (best, peak) = scores
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(index, value)| (index, *value))?;
    let mean = scores.iter().sum::<f32>() / scores.len() as f32;
    if mean <= 0.0 || peak / mean < CONFIDENCE {
        return None;
    }
    let left = scores.get(best.wrapping_sub(1)).copied().unwrap_or(peak);
    let right = scores.get(best + 1).copied().unwrap_or(peak);
    let curvature = left - 2.0 * peak + right;
    let shift = if curvature < 0.0 {
        0.5 * (left - right) / curvature
    } else {
        0.0
    };
    Some((shortest + best) as f32 + shift.clamp(-0.5, 0.5))
}

/// Dynamic-programming beat tracking: each beat is placed on a strong onset about one period
/// after the previous beat.
fn follow_beats(envelope: &[f32], period: f32) -> Vec<f32> {
    let mut score = vec![0.0f32; envelope.len()];
    let mut back = vec![usize::MAX; envelope.len()];
    let (near, far) = (
        (period * 0.5).round() as usize,
        (period * 2.0).round() as usize,
    );
    for t in 0..envelope.len() {
        let mut best = (0.0f32, usize::MAX);
        for previous in t.saturating_sub(far)..t.saturating_sub(near) {
            let gap = (t - previous) as f32 / period;
            let candidate = score[previous] - TIGHTNESS * gap.ln().powi(2);
            if (back[previous] != usize::MAX || score[previous] > 0.0)
                && (best.1 == usize::MAX || candidate > best.0)
            {
                best = (candidate, previous);
            }
        }
        score[t] = envelope[t] + best.0.max(0.0);
        back[t] = if best.0 > 0.0 { best.1 } else { usize::MAX };
    }
    let tail = envelope.len().saturating_sub(period.ceil() as usize * 2);
    let mut beat = (tail..envelope.len())
        .max_by(|a, b| score[*a].total_cmp(&score[*b]))
        .unwrap_or(0);
    let mut beats = vec![beat as f32];
    while back[beat] != usize::MAX {
        beat = back[beat];
        beats.push(beat as f32);
    }
    beats.reverse();
    beats
}

/// Replaces the tracked beats with an exact grid when the tempo is steady, which places every
/// beat more precisely than one onset frame.
fn straighten(beats: &[f32], period: f32, rate: f32) -> (Vec<f32>, f32) {
    let count = beats.len() as f32;
    let mean_index = (count - 1.0) / 2.0;
    let mean_frame = beats.iter().sum::<f32>() / count;
    let (mut covariance, mut variance) = (0.0, 0.0);
    for (index, frame) in beats.iter().enumerate() {
        let offset = index as f32 - mean_index;
        covariance += offset * (frame - mean_frame);
        variance += offset * offset;
    }
    let slope = covariance / variance;
    let intercept = mean_frame - slope * mean_index;
    let residual = (beats
        .iter()
        .enumerate()
        .map(|(index, frame)| (frame - (intercept + slope * index as f32)).powi(2))
        .sum::<f32>()
        / count)
        .sqrt();
    if residual / rate > STEADY_SECONDS || (slope / period - 1.0).abs() > 0.05 {
        return (beats.to_vec(), period);
    }
    let grid = (0..beats.len())
        .map(|index| intercept + slope * index as f32)
        .collect();
    (grid, slope)
}

/// Bars start where the harmony changes most, and where the bass hits.
fn find_downbeat(beats: &[f32], onsets: &Onsets) -> u8 {
    let mut profiles = Vec::with_capacity(beats.len());
    for pair in beats.windows(2) {
        let (from, to) = (
            pair[0].max(0.0) as usize,
            (pair[1].max(0.0) as usize).min(onsets.bands.len()),
        );
        let mut profile = [0.0f32; BANDS];
        for frame in onsets.bands.get(from..to).unwrap_or_default() {
            for band in 0..BANDS {
                profile[band] += frame[band];
            }
        }
        let length = (to.saturating_sub(from)).max(1) as f32;
        profile.iter_mut().for_each(|value| *value /= length);
        profiles.push(profile);
    }
    let mut change = [0.0f32; 4];
    let mut bass = [0.0f32; 4];
    for (index, pair) in profiles.windows(2).enumerate() {
        let beat = index + 1;
        change[beat % 4] += pair[0]
            .iter()
            .zip(&pair[1])
            .map(|(a, b)| (a - b).abs())
            .sum::<f32>();
        let frame = beats[beat].max(0.0) as usize;
        bass[beat % 4] += onsets
            .low
            .get(frame..frame + 3)
            .map_or(0.0, |low| low.iter().copied().fold(0.0, f32::max));
    }
    let scale = |values: [f32; 4]| {
        let total = values.iter().sum::<f32>().max(f32::EPSILON);
        values.map(|value| value / total)
    };
    let (change, bass) = (scale(change), scale(bass));
    (0..4)
        .max_by(|a, b| (change[*a] + 0.4 * bass[*a]).total_cmp(&(change[*b] + 0.4 * bass[*b])))
        .unwrap_or(0) as u8
}
