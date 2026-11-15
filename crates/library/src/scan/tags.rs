use std::path::Path;

use lofty::file::TaggedFileExt;
use lofty::picture::PictureType;
use lofty::prelude::{Accessor, ItemKey};
use lofty::tag::Tag;

#[derive(Default)]
pub struct Tags {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub track: Option<u32>,
    pub disc: Option<u32>,
    pub year: Option<u32>,
    pub genre: Option<String>,
    pub replay_gain: Option<f32>,
    pub cover: Option<Vec<u8>>,
}

fn text(value: Option<std::borrow::Cow<'_, str>>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn decibels(value: &str) -> Option<f32> {
    value
        .trim()
        .trim_end_matches("dB")
        .trim_end_matches("db")
        .trim()
        .parse()
        .ok()
}

fn cover(tag: &Tag) -> Option<Vec<u8>> {
    let pictures = tag.pictures();
    pictures
        .iter()
        .find(|picture| picture.pic_type() == PictureType::CoverFront)
        .or_else(|| pictures.first())
        .map(|picture| picture.data().to_vec())
}

/// Reads a file's tags. A file without tags reads as empty.
pub fn read(path: &Path) -> Tags {
    let Ok(file) = lofty::read_from_path(path) else {
        return Tags::default();
    };
    let Some(tag) = file.primary_tag().or_else(|| file.first_tag()) else {
        return Tags::default();
    };
    Tags {
        title: text(tag.title()),
        artist: text(tag.artist()),
        album: text(tag.album()),
        album_artist: tag
            .get_string(ItemKey::AlbumArtist)
            .map(str::to_string)
            .filter(|value| !value.is_empty()),
        track: tag.track(),
        disc: tag.disk(),
        year: tag.date().map(|date| u32::from(date.year)),
        genre: text(tag.genre()),
        replay_gain: tag
            .get_string(ItemKey::ReplayGainTrackGain)
            .and_then(decibels),
        cover: cover(tag).or_else(|| file.tags().iter().find_map(cover)),
    }
}
