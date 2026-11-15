mod decoder;
mod mp4;

pub use decoder::{Decoder, Identity, MediaError, identify};

pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "flac", "mp3", "m4a", "aac", "mp4", "alac", "ogg", "oga", "opus", "wav", "aif", "aiff", "aifc",
];

pub fn is_supported(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            SUPPORTED_EXTENSIONS
                .iter()
                .any(|supported| extension.eq_ignore_ascii_case(supported))
        })
}
