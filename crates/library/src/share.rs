use std::path::Path;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::db::playlists::Awaited;
use crate::model::{Edits, Missing, Song, Transition};
use crate::{Inner, LibraryError, LibraryEvent};

const FORMAT: u32 = 1;
const SHARED_FINGERPRINT: usize = 480;
const LENGTH_TOLERANCE: f64 = 4.0;

#[derive(Serialize, Deserialize)]
struct SharedPlaylist {
    attacca: u32,
    name: String,
    transition: Option<Transition>,
    cover: Option<String>,
    songs: Vec<SharedSong>,
    pairs: Vec<SharedPair>,
}

#[derive(Serialize, Deserialize)]
struct SharedSong {
    title: String,
    artist: Option<String>,
    album: Option<String>,
    duration: f64,
    fingerprint: String,
    edits: Edits,
}

#[derive(Serialize, Deserialize)]
struct SharedPair {
    from: usize,
    to: usize,
    transition: Transition,
}

#[derive(Clone, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    pub name: String,
    pub songs: u32,
    pub found: u32,
    pub missing: u32,
    /// Songs you already edited that the sender edited too.
    pub both_edited: u32,
}

fn encode_fingerprint(fingerprint: &[u32]) -> String {
    let words = &fingerprint[..fingerprint.len().min(SHARED_FINGERPRINT)];
    STANDARD.encode(
        words
            .iter()
            .flat_map(|word| word.to_le_bytes())
            .collect::<Vec<u8>>(),
    )
}

fn decode_fingerprint(text: &str) -> Vec<u32> {
    STANDARD
        .decode(text)
        .unwrap_or_default()
        .as_chunks::<4>()
        .0
        .iter()
        .map(|bytes| u32::from_le_bytes(*bytes))
        .collect()
}

fn same_text(a: Option<&str>, b: Option<&str>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => a.trim().eq_ignore_ascii_case(b.trim()),
        _ => false,
    }
}

fn shift(edits: &mut Edits, offset: f64) {
    edits.start = edits.start.map(|start| (start + offset).max(0.0));
    edits.end = edits.end.map(|end| (end + offset).max(0.0));
}

/// Finds a song in the library: by its sound first, then by its tags and length. Returns the
/// song and how far its audio is shifted against the sender's copy.
fn find(
    inner: &Inner,
    songs: &[Song],
    title: &str,
    artist: Option<&str>,
    duration: f64,
    fingerprint: &[u32],
) -> Option<(String, f64)> {
    let candidates = inner
        .db
        .lock()
        .fingerprints_near(duration, LENGTH_TOLERANCE)
        .unwrap_or_default();
    let by_sound = candidates.iter().find_map(|(song, ours)| {
        attacca_analysis::fingerprint_offset(fingerprint, ours)
            .map(|offset| (song.clone(), f64::from(offset)))
    });
    by_sound.or_else(|| {
        songs
            .iter()
            .find(|song| {
                same_text(song.title.as_deref().or(Some(&song.name)), Some(title))
                    && (artist.is_none() || same_text(song.artist.as_deref(), artist))
                    && (song.duration - duration).abs() <= LENGTH_TOLERANCE
            })
            .map(|song| (song.id.clone(), 0.0))
    })
}

impl Inner {
    pub fn export(&self, playlist: i64, path: &Path) -> Result<(), LibraryError> {
        let db = self.db.lock();
        let list = db
            .playlist(playlist)?
            .ok_or_else(|| LibraryError::new("That playlist is gone"))?;
        let songs: Vec<(String, Song)> = list
            .entries
            .iter()
            .filter_map(|entry| entry.song.clone())
            .filter_map(|id| db.song(&id).ok().flatten().map(|song| (id, song)))
            .collect();
        let shared = SharedPlaylist {
            attacca: FORMAT,
            name: list.name.clone(),
            transition: list.transition,
            cover: list
                .cover
                .as_deref()
                .and_then(|cover| self.covers.read(cover))
                .map(|bytes| STANDARD.encode(bytes)),
            songs: songs
                .iter()
                .map(|(id, song)| SharedSong {
                    title: song.display_title().to_string(),
                    artist: song.artist.clone(),
                    album: song.album.clone(),
                    duration: song.duration,
                    fingerprint: db
                        .analysis(id)
                        .ok()
                        .flatten()
                        .map(|stored| encode_fingerprint(&stored.fingerprint))
                        .unwrap_or_default(),
                    edits: db.edits(id).unwrap_or_default(),
                })
                .collect(),
            pairs: list
                .pairs
                .iter()
                .filter_map(|pair| {
                    let from = songs.iter().position(|(id, _)| *id == pair.from)?;
                    let to = songs.iter().position(|(id, _)| *id == pair.to)?;
                    Some(SharedPair {
                        from,
                        to,
                        transition: pair.transition,
                    })
                })
                .collect(),
        };
        let text = serde_json::to_string(&shared).expect("shared playlists serialize");
        std::fs::write(path, text)
            .map_err(|error| LibraryError::new(format!("Couldn't save the playlist: {error}")))
    }

    fn read_shared(path: &Path) -> Result<SharedPlaylist, LibraryError> {
        let text = std::fs::read_to_string(path).map_err(|error| {
            LibraryError::new(format!("Couldn't open {}: {error}", path.display()))
        })?;
        let shared: SharedPlaylist = serde_json::from_str(&text)
            .map_err(|_| LibraryError::new("That isn't an Attacca playlist"))?;
        if shared.attacca > FORMAT {
            return Err(LibraryError::new(
                "This playlist was made by a newer Attacca",
            ));
        }
        Ok(shared)
    }

    fn matches(&self, shared: &SharedPlaylist) -> Vec<Option<(String, f64)>> {
        let songs = self.db.lock().songs().unwrap_or_default();
        shared
            .songs
            .iter()
            .map(|song| {
                find(
                    self,
                    &songs,
                    &song.title,
                    song.artist.as_deref(),
                    song.duration,
                    &decode_fingerprint(&song.fingerprint),
                )
            })
            .collect()
    }

    pub fn preview_import(&self, path: &Path) -> Result<ImportPreview, LibraryError> {
        let shared = Self::read_shared(path)?;
        let matches = self.matches(&shared);
        let db = self.db.lock();
        let both_edited = shared
            .songs
            .iter()
            .zip(&matches)
            .filter(|(song, found)| {
                !song.edits.is_empty()
                    && found
                        .as_ref()
                        .is_some_and(|(id, _)| !db.edits(id).unwrap_or_default().is_empty())
            })
            .count();
        let found = matches.iter().filter(|found| found.is_some()).count();
        Ok(ImportPreview {
            name: shared.name,
            songs: shared.songs.len() as u32,
            found: found as u32,
            missing: (shared.songs.len() - found) as u32,
            both_edited: both_edited as u32,
        })
    }

    /// Adds a shared playlist. Songs you have join it with the sender's edits, unless you edited
    /// them yourself and `keep_own` is set; the rest wait in it until they appear.
    pub fn import(&self, path: &Path, keep_own: bool) -> Result<i64, LibraryError> {
        let shared = Self::read_shared(path)?;
        let matches = self.matches(&shared);
        let cover = shared
            .cover
            .as_deref()
            .and_then(|cover| STANDARD.decode(cover).ok())
            .and_then(|bytes| self.covers.store(&bytes));
        let db = self.db.lock();
        let playlist = db.create_playlist(&shared.name)?;
        db.set_playlist_transition(playlist, shared.transition.as_ref())?;
        db.set_kept_cover(&format!("playlist:{playlist}"), cover.as_deref())?;
        let mut ids = Vec::with_capacity(shared.songs.len());
        for (song, found) in shared.songs.iter().zip(&matches) {
            match found {
                Some((id, offset)) => {
                    let own = db.edits(id)?;
                    if !song.edits.is_empty() && (own.is_empty() || !keep_own) {
                        let mut edits = song.edits.clone();
                        shift(&mut edits, *offset);
                        db.set_edits(id, &edits)?;
                    }
                    db.append_songs(playlist, std::slice::from_ref(id))?;
                    ids.push(Some(id.clone()));
                }
                None => {
                    db.append_awaited(
                        playlist,
                        &Awaited {
                            missing: Missing {
                                title: song.title.clone(),
                                artist: song.artist.clone(),
                                album: song.album.clone(),
                                duration: song.duration,
                            },
                            fingerprint: decode_fingerprint(&song.fingerprint),
                            edits: song.edits.clone(),
                        },
                    )?;
                    ids.push(None);
                }
            }
        }
        for pair in &shared.pairs {
            if let (Some(Some(from)), Some(Some(to))) = (ids.get(pair.from), ids.get(pair.to)) {
                db.set_pair(playlist, from, to, Some(&pair.transition))?;
            }
        }
        drop(db);
        self.emit(LibraryEvent::Playlists);
        Ok(playlist)
    }
}

/// Puts a newly analysed song into the shared playlists that were waiting for it.
pub fn welcome(inner: &Inner, song: &str, fingerprint: &[u32]) {
    let db = inner.db.lock();
    let Ok(awaited) = db.awaited() else { return };
    if awaited.is_empty() {
        return;
    }
    let Some(details) = db.song(song).ok().flatten() else {
        return;
    };
    let mut arrived = false;
    for (entry, waiting) in awaited {
        if (waiting.missing.duration - details.duration).abs() > LENGTH_TOLERANCE {
            continue;
        }
        let offset = attacca_analysis::fingerprint_offset(&waiting.fingerprint, fingerprint)
            .map(f64::from)
            .or_else(|| {
                (same_text(Some(details.display_title()), Some(&waiting.missing.title))
                    && (waiting.missing.artist.is_none()
                        || same_text(details.artist.as_deref(), waiting.missing.artist.as_deref())))
                .then_some(0.0)
            });
        let Some(offset) = offset else { continue };
        if db.arrive(entry, song).ok().flatten().is_some() {
            arrived = true;
            if db.edits(song).unwrap_or_default().is_empty() && !waiting.edits.is_empty() {
                let mut edits = waiting.edits.clone();
                shift(&mut edits, offset);
                let _ = db.set_edits(song, &edits);
            }
        }
    }
    drop(db);
    if arrived {
        inner.emit(LibraryEvent::Playlists);
    }
}
