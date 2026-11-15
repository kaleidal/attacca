use std::path::{Path, PathBuf};

use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat};
use lofty::config::WriteOptions;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::picture::{MimeType, Picture, PictureType};
use lofty::tag::Tag;
use serde::Deserialize;
use specta::Type;

use crate::LibraryError;

const FULL: u32 = 1200;
const SMALL: u32 = 360;

/// The part of an image to keep, as fractions of its width and height.
#[derive(Clone, Copy, Debug, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Crop {
    pub x: f32,
    pub y: f32,
    pub size: f32,
}

/// Cover images, kept as JPEG files named after a hash of their source.
pub struct Covers {
    dir: PathBuf,
}

fn encode(image: &DynamicImage, quality: u8) -> Vec<u8> {
    let mut bytes = Vec::new();
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, quality);
    image
        .to_rgb8()
        .write_with_encoder(encoder)
        .expect("encoding into memory succeeds");
    bytes
}

fn fit(image: &DynamicImage, size: u32) -> DynamicImage {
    if image.width() <= size && image.height() <= size {
        return image.clone();
    }
    image.resize(size, size, FilterType::Lanczos3)
}

impl Covers {
    pub fn new(dir: PathBuf) -> std::io::Result<Self> {
        std::fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Stores an image and returns its name. Images already stored are not decoded again.
    pub fn store(&self, bytes: &[u8]) -> Option<String> {
        let name = blake3::hash(bytes).to_hex()[..20].to_string();
        let full = self.dir.join(format!("{name}.jpg"));
        if full.exists() {
            return Some(name);
        }
        let image = image::load_from_memory(bytes).ok()?;
        std::fs::write(&full, encode(&fit(&image, FULL), 90)).ok()?;
        std::fs::write(
            self.dir.join(format!("{name}-small.jpg")),
            encode(&fit(&image, SMALL), 86),
        )
        .ok()?;
        Some(name)
    }

    pub fn read(&self, name: &str) -> Option<Vec<u8>> {
        std::fs::read(self.dir.join(format!("{name}.jpg"))).ok()
    }
}

/// Crops an image to a square and encodes it for a cover.
pub fn square(bytes: &[u8], crop: Crop) -> Result<Vec<u8>, LibraryError> {
    let image = image::load_from_memory(bytes)
        .map_err(|_| LibraryError::new("That image can't be read"))?;
    let (width, height) = (image.width() as f32, image.height() as f32);
    let side = (crop.size * width.min(height)).clamp(1.0, width.min(height));
    let x = (crop.x * width).clamp(0.0, width - side);
    let y = (crop.y * height).clamp(0.0, height - side);
    let cropped = image.crop_imm(x as u32, y as u32, side as u32, side as u32);
    Ok(encode(&fit(&cropped, FULL), 92))
}

pub fn is_image(bytes: &[u8]) -> bool {
    image::guess_format(bytes).is_ok_and(|format| {
        matches!(
            format,
            ImageFormat::Jpeg
                | ImageFormat::Png
                | ImageFormat::WebP
                | ImageFormat::Gif
                | ImageFormat::Bmp
        )
    })
}

/// Writes a JPEG cover into a file's tags, in place of its front cover.
pub fn write_into(path: &Path, jpeg: &[u8]) -> Result<(), LibraryError> {
    let failed = |error: &dyn std::fmt::Display| {
        LibraryError::new(format!(
            "Couldn't write the cover into {}: {error}",
            path.display()
        ))
    };
    let mut file = lofty::read_from_path(path).map_err(|error| failed(&error))?;
    if file.primary_tag().is_none() {
        let kind = file.primary_tag_type();
        file.insert_tag(Tag::new(kind));
    }
    let tag = file
        .primary_tag_mut()
        .expect("a primary tag was just ensured");
    tag.remove_picture_type(PictureType::CoverFront);
    tag.push_picture(
        Picture::unchecked(jpeg.to_vec())
            .pic_type(PictureType::CoverFront)
            .mime_type(MimeType::Jpeg)
            .build(),
    );
    file.save_to_path(path, WriteOptions::default())
        .map_err(|error| failed(&error))?;
    Ok(())
}
