use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    pub id: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub track: Option<u32>,
    pub disc: Option<u32>,
    pub year: Option<u32>,
    pub genre: Option<String>,
    pub duration: f64,
    /// File name without its extension, shown when the song has no title.
    pub name: String,
    /// Name of the folder holding the file, shown when the song has no album.
    pub folder: String,
    pub cover: Option<String>,
    pub added: i64,
    pub tempo: Option<f32>,
    pub analysed: bool,
    pub edited: bool,
    /// The song's own Transition out, when it has one.
    pub transition: Option<Transition>,
}

impl Song {
    pub fn display_title(&self) -> &str {
        self.title.as_deref().unwrap_or(&self.name)
    }

    pub fn album_key(&self) -> Option<String> {
        let album = self.album.as_deref()?;
        let artist = self
            .album_artist
            .as_deref()
            .or(self.artist.as_deref())
            .unwrap_or_default();
        Some(format!(
            "{}\u{1f}{}",
            artist.to_lowercase(),
            album.to_lowercase()
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum TransitionName {
    None,
    Fade,
    Rise,
    Blend,
    Wave,
    Melt,
    Slam,
}

impl TransitionName {
    pub fn default_seconds(self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Fade => 6.0,
            Self::Rise => 8.0,
            Self::Blend => 16.0,
            Self::Wave => 10.0,
            Self::Melt => 5.0,
            Self::Slam => 0.8,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Transition {
    pub kind: TransitionName,
    pub seconds: f32,
    pub snap: bool,
}

impl Transition {
    pub const FADE: Transition = Transition {
        kind: TransitionName::Fade,
        seconds: 6.0,
        snap: true,
    };
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Equalizer {
    pub preamp: f32,
    pub gains: Vec<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Cut {
    pub hz: f32,
    pub slope: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Filters {
    pub highpass: Option<Cut>,
    pub lowpass: Option<Cut>,
}

/// A song's own settings. None of them changes the file.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Edits {
    pub start: Option<f64>,
    pub end: Option<f64>,
    /// Decibels on top of volume matching.
    #[serde(default)]
    pub volume: f32,
    pub equalizer: Option<Equalizer>,
    pub filters: Option<Filters>,
    pub transition: Option<Transition>,
}

impl Edits {
    pub fn is_empty(&self) -> bool {
        *self == Edits::default()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Rhythm {
    pub tempo: f32,
    pub beats: Vec<f32>,
    pub downbeat: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Listening {
    pub loudness: f32,
    pub peak: f32,
    pub rhythm: Option<Rhythm>,
    pub suggested_start: f32,
    pub suggested_end: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SongDetail {
    pub song: Song,
    pub edits: Edits,
    pub listening: Option<Listening>,
    pub path: Option<String>,
}

/// A playlist entry: a song in the library, or one shared with you that isn't here yet.
#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: i64,
    pub song: Option<String>,
    pub missing: Option<Missing>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Missing {
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Pair {
    pub from: String,
    pub to: String,
    pub transition: Transition,
}

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub id: i64,
    pub name: String,
    pub cover: Option<String>,
    pub transition: Option<Transition>,
    pub entries: Vec<Entry>,
    pub pairs: Vec<Pair>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistSummary {
    pub id: i64,
    pub name: String,
    pub cover: Option<String>,
    pub count: u32,
    pub covers: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AlbumSettings {
    pub transitions: bool,
}
