use std::fmt;
use std::fs::File;
use std::path::Path;
use std::sync::OnceLock;

use symphonia::core::codecs::audio::well_known::{CODEC_ID_AAC, CODEC_ID_OPUS};
use symphonia::core::codecs::audio::{AudioDecoder, AudioDecoderOptions};
use symphonia::core::codecs::registry::CodecRegistry;
use symphonia::core::common::Tier;
use symphonia::core::errors::Error;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::units::{Time, TimeBase, Timestamp};

#[derive(Debug)]
pub struct MediaError(String);

impl fmt::Display for MediaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for MediaError {}

impl From<Error> for MediaError {
    fn from(error: Error) -> Self {
        MediaError(error.to_string())
    }
}

impl From<std::io::Error> for MediaError {
    fn from(error: std::io::Error) -> Self {
        MediaError(error.to_string())
    }
}

fn codecs() -> &'static CodecRegistry {
    static REGISTRY: OnceLock<CodecRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut registry = CodecRegistry::new();
        symphonia::default::register_enabled_codecs(&mut registry);
        registry.register_audio_decoder_at_tier::<symphonia_opus::OpusDecoder>(Tier::Fallback);
        registry
    })
}

fn open_format(path: &Path) -> Result<Box<dyn FormatReader>, MediaError> {
    let source = MediaSourceStream::new(Box::new(File::open(path)?), Default::default());
    let mut hint = Hint::new();
    if let Some(extension) = path.extension().and_then(|extension| extension.to_str()) {
        hint.with_extension(extension);
    }
    let metadata = MetadataOptions::default();
    Ok(symphonia::default::get_probe().probe(&hint, source, FormatOptions::default(), metadata)?)
}

/// Decodes the default audio track of a file into stereo frames, trimming encoder delay and
/// padding so consecutive files join without gaps.
pub struct Decoder {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn AudioDecoder>,
    track_id: u32,
    time_base: Option<TimeBase>,
    pub sample_rate: u32,
    pub channels: usize,
    pub frames: Option<u64>,
    /// Frames the codec puts before the song that its demuxer leaves in, such as Opus pre-skip.
    lead: u64,
    skip: u64,
    /// Frames of the song itself, when the container says where it ends.
    length: Option<u64>,
    remaining: Option<u64>,
    interleaved: Vec<f32>,
}

impl Decoder {
    pub fn open(path: &Path) -> Result<Self, MediaError> {
        let format = open_format(path)?;
        let track = format
            .default_track(TrackType::Audio)
            .ok_or_else(|| MediaError("the file has no audio".into()))?;
        let params = track
            .codec_params
            .as_ref()
            .and_then(|params| params.audio())
            .ok_or_else(|| MediaError("the audio track has no codec".into()))?;
        let sample_rate = params
            .sample_rate
            .ok_or_else(|| MediaError("the audio track has no sample rate".into()))?;
        let channels = params
            .channels
            .as_ref()
            .map_or(2, |channels| channels.count());
        let decoder = codecs().make_audio_decoder(params, &AudioDecoderOptions::default())?;
        let edit = (params.codec == CODEC_ID_AAC)
            .then(|| crate::mp4::edit(path))
            .flatten();
        let scaled = |value: u64, scale: u64| value * u64::from(sample_rate) / scale.max(1);
        let (lead, length) = match (params.codec, edit) {
            (_, Some(edit)) => (
                scaled(edit.lead, edit.scale),
                Some(scaled(edit.frames, edit.scale)),
            ),
            (CODEC_ID_OPUS, None) => (u64::from(track.delay.unwrap_or(0)), None),
            _ => (0, None),
        };
        Ok(Self {
            track_id: track.id,
            time_base: track.time_base,
            frames: length.or(track.num_frames.map(|frames| frames.saturating_sub(lead))),
            lead,
            skip: lead,
            length,
            remaining: length,
            sample_rate,
            channels,
            decoder,
            format,
            interleaved: Vec::new(),
        })
    }

    pub fn duration_seconds(&self) -> Option<f64> {
        self.frames
            .map(|frames| frames as f64 / f64::from(self.sample_rate))
    }

    /// Moves to a frame at the file's own rate. Decoding resumes exactly there.
    ///
    /// Lossy decoders need a few frames of the past before their output is right, so the seek
    /// lands a little earlier and decodes up to the frame.
    pub fn seek_frame(&mut self, frame: u64) -> Result<(), MediaError> {
        if frame == 0 {
            return Ok(());
        }
        self.remaining = self.length.map(|length| length.saturating_sub(frame));
        let frame = frame + self.lead;
        let warmup = u64::from(self.sample_rate / 10);
        let target = frame.saturating_sub(warmup);
        let time = Time::try_from_secs_f64(target as f64 / f64::from(self.sample_rate))
            .ok_or_else(|| MediaError("seek position out of range".into()))?;
        let seeked = self.format.seek(
            SeekMode::Accurate,
            SeekTo::Time {
                time,
                track_id: Some(self.track_id),
            },
        )?;
        self.decoder.reset();
        self.skip = frame.saturating_sub(self.timestamp_frames(seeked.actual_ts));
        Ok(())
    }

    fn timestamp_frames(&self, ts: Timestamp) -> u64 {
        match self.time_base {
            Some(base) => {
                let seconds = base.calc_time_saturating(ts).as_secs_f64();
                (seconds * f64::from(self.sample_rate)).round() as u64
            }
            None => ts.get().max(0) as u64,
        }
    }

    /// Appends the next decoded stereo frames to `out`. Returns `false` at the end of the stream.
    pub fn decode_into(&mut self, out: &mut Vec<[f32; 2]>) -> Result<bool, MediaError> {
        loop {
            let packet = match self.format.next_packet() {
                Ok(Some(packet)) => packet,
                Ok(None) => return Ok(false),
                Err(Error::ResetRequired) => return Ok(false),
                Err(Error::IoError(error)) if error.kind() == std::io::ErrorKind::UnexpectedEof => {
                    return Ok(false);
                }
                Err(error) => return Err(error.into()),
            };
            if packet.track_id != self.track_id {
                continue;
            }
            let decoded = match self.decoder.decode(&packet) {
                Ok(decoded) => decoded,
                Err(Error::DecodeError(_)) => continue,
                Err(error) => return Err(error.into()),
            };
            let channels = decoded.spec().channels().count().max(1);
            let trims = packet.trim_start.get() + packet.trim_end.get();
            let length = decoded.frames() as u64;
            let untrimmed = length == packet.dur.get() || length == packet.dur.get() + trims;
            self.interleaved.resize(decoded.samples_interleaved(), 0.0);
            decoded.copy_to_slice_interleaved(&mut self.interleaved);
            let mut frames = self.interleaved.len() / channels;
            let mut lead = 0;
            if untrimmed {
                lead = (packet.trim_start.get() as usize).min(frames);
                frames -= (packet.trim_end.get() as usize).min(frames - lead);
            }
            let skip = (self.skip as usize).min(frames - lead);
            self.skip -= skip as u64;
            let mut count = frames - lead - skip;
            if let Some(remaining) = &mut self.remaining {
                count = count.min(*remaining as usize);
                *remaining -= count as u64;
            }
            out.reserve(count);
            let first = lead + skip;
            for frame in self.interleaved[first * channels..(first + count) * channels]
                .chunks_exact(channels)
            {
                out.push(match channels {
                    1 => [frame[0], frame[0]],
                    _ => [frame[0], frame[1]],
                });
            }
            if self.remaining == Some(0) {
                return Ok(false);
            }
            if count > 0 {
                return Ok(true);
            }
        }
    }
}

/// A file's identity and length.
pub struct Identity {
    pub id: String,
    pub duration: f64,
}

/// Identifies a file by its encoded audio alone, so retagging, moving or renaming it keeps the
/// same identity.
pub fn identify(path: &Path) -> Result<Identity, MediaError> {
    let duration = Decoder::open(path)?;
    let mut format = open_format(path)?;
    let track_id = format
        .default_track(TrackType::Audio)
        .ok_or_else(|| MediaError("the file has no audio".into()))?
        .id;
    let mut hasher = blake3::Hasher::new();
    let mut frames = 0u64;
    loop {
        match format.next_packet() {
            Ok(Some(packet)) if packet.track_id == track_id => {
                frames += packet.dur.get();
                hasher.update(&packet.data);
            }
            Ok(Some(_)) => {}
            Ok(None) | Err(Error::ResetRequired) => break,
            Err(Error::IoError(error)) if error.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(error) => return Err(error.into()),
        }
    }
    let seconds = duration
        .duration_seconds()
        .unwrap_or(frames as f64 / f64::from(duration.sample_rate));
    Ok(Identity {
        id: hasher.finalize().to_hex()[..32].to_string(),
        duration: seconds,
    })
}
