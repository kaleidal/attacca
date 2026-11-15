mod tags;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

use crossbeam_channel::{Receiver, RecvTimeoutError};
use notify::{RecursiveMode, Watcher};

use crate::db::NewSong;
use crate::{Inner, LibraryEvent};

const SETTLE: Duration = Duration::from_millis(800);

pub enum Scan {
    Folder(PathBuf),
    Forget(PathBuf),
    Paths(Vec<PathBuf>),
}

pub enum Outcome {
    Unchanged,
    Changed { song: String, new: bool },
}

/// Keeps the library in step with its folders: scans them, then follows what changes in them.
pub fn run(inner: &Inner, scans: Receiver<Scan>) {
    let sender = inner.scans.clone();
    let mut watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
        if let Ok(event) = event {
            let _ = sender.send(Scan::Paths(event.paths));
        }
    })
    .ok();
    let mut changed: HashSet<PathBuf> = HashSet::new();
    loop {
        let scan = if changed.is_empty() {
            scans.recv().map_err(|_| RecvTimeoutError::Disconnected)
        } else {
            scans.recv_timeout(SETTLE)
        };
        match scan {
            Ok(Scan::Folder(folder)) => {
                if let Some(watcher) = &mut watcher {
                    let _ = watcher.watch(&folder, RecursiveMode::Recursive);
                }
                scan_folder(inner, &folder);
            }
            Ok(Scan::Forget(folder)) => {
                if let Some(watcher) = &mut watcher {
                    let _ = watcher.unwatch(&folder);
                }
            }
            Ok(Scan::Paths(paths)) => changed.extend(paths),
            Err(RecvTimeoutError::Timeout) => rescan(inner, std::mem::take(&mut changed)),
            Err(RecvTimeoutError::Disconnected) => return,
        }
    }
}

fn walk(folder: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if entry.file_name().to_string_lossy().starts_with('.') {
            continue;
        }
        match entry.file_type() {
            Ok(kind) if kind.is_dir() => walk(&path, found),
            Ok(kind) if kind.is_file() && attacca_media::is_supported(&path) => found.push(path),
            _ => {}
        }
    }
}

fn scan_folder(inner: &Inner, folder: &Path) {
    inner.emit(LibraryEvent::Scanning(true));
    let mut found = Vec::new();
    walk(folder, &mut found);
    let seen: HashSet<String> = found
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect();
    process(inner, &found);
    let known = inner
        .db
        .lock()
        .paths_under(&folder.to_string_lossy())
        .unwrap_or_default();
    let gone: Vec<String> = known
        .into_iter()
        .filter(|path| !seen.contains(path))
        .collect();
    forget(inner, &gone);
    inner.emit(LibraryEvent::Scanning(false));
}

fn rescan(inner: &Inner, paths: HashSet<PathBuf>) {
    let mut found = Vec::new();
    let mut gone = Vec::new();
    for path in paths {
        if path.is_dir() {
            walk(&path, &mut found);
        } else if path.is_file() {
            if attacca_media::is_supported(&path) {
                found.push(path);
            }
        } else {
            let text = path.to_string_lossy().into_owned();
            gone.extend(inner.db.lock().paths_under(&text).unwrap_or_default());
            gone.push(text);
        }
    }
    process(inner, &found);
    forget(inner, &gone);
}

fn process(inner: &Inner, paths: &[PathBuf]) {
    let mut changed = Vec::new();
    let mut fresh = Vec::new();
    for (index, path) in paths.iter().enumerate() {
        match read_file(inner, path) {
            Ok(Outcome::Changed { song, new }) => {
                if new {
                    fresh.push(song.clone());
                }
                changed.push(song);
            }
            Ok(Outcome::Unchanged) => {}
            Err(error) => eprintln!("attacca: skipping {}: {error}", path.display()),
        }
        if changed.len() >= 200 || (index + 1 == paths.len() && !changed.is_empty()) {
            inner.emit(LibraryEvent::Songs {
                changed: std::mem::take(&mut changed),
                removed: Vec::new(),
            });
        }
    }
    inner.queue.push(fresh);
}

fn forget(inner: &Inner, paths: &[String]) {
    if paths.is_empty() {
        return;
    }
    let removed = inner.db.lock().remove_files(paths).unwrap_or_default();
    if !removed.is_empty() {
        inner.emit(LibraryEvent::Songs {
            changed: Vec::new(),
            removed,
        });
    }
}

const ART_NAMES: &[&str] = &["cover", "folder", "front", "album", "artwork"];
const ART_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp"];

/// The album art kept beside a song as a picture file, such as `cover.jpg`.
fn folder_art(song: &Path) -> Option<Vec<u8>> {
    let entries = std::fs::read_dir(song.parent()?).ok()?;
    let art = entries.flatten().map(|entry| entry.path()).find(|path| {
        let lower = |part: Option<&std::ffi::OsStr>| {
            part.and_then(|part| part.to_str())
                .unwrap_or_default()
                .to_lowercase()
        };
        ART_NAMES.contains(&lower(path.file_stem()).as_str())
            && ART_EXTENSIONS.contains(&lower(path.extension()).as_str())
    })?;
    std::fs::read(art).ok()
}

/// Gives a song that has no cover the art beside it, when there is some.
fn cover_from_folder(inner: &Inner, key: &str, path: &Path) -> Outcome {
    let Some(song) = inner.db.lock().coverless_song_at(key).ok().flatten() else {
        return Outcome::Unchanged;
    };
    let Some(cover) = folder_art(path).and_then(|bytes| inner.covers.store(&bytes)) else {
        return Outcome::Unchanged;
    };
    match inner.db.lock().set_song_cover(&song, &cover) {
        Ok(()) => Outcome::Changed { song, new: false },
        Err(_) => Outcome::Unchanged,
    }
}

pub fn read_file(inner: &Inner, path: &Path) -> Result<Outcome, String> {
    let metadata = std::fs::metadata(path).map_err(|error| error.to_string())?;
    let size = metadata.len() as i64;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |time| time.as_secs() as i64);
    let key = path.to_string_lossy().into_owned();
    let known = inner
        .db
        .lock()
        .file(&key)
        .map_err(|error| error.to_string())?;
    if known
        .as_ref()
        .is_some_and(|known| known.size == size && known.modified == modified)
    {
        return Ok(cover_from_folder(inner, &key, path));
    }
    let identity = attacca_media::identify(path).map_err(|error| error.to_string())?;
    let tags = tags::read(path);
    let cover = match tags.cover {
        Some(bytes) => inner.covers.store(&bytes),
        None => folder_art(path).and_then(|bytes| inner.covers.store(&bytes)),
    };
    let name = path
        .file_stem()
        .map_or_else(String::new, |stem| stem.to_string_lossy().into_owned());
    let folder = path
        .parent()
        .and_then(|parent| parent.file_name())
        .map_or_else(String::new, |folder| folder.to_string_lossy().into_owned());
    let song = NewSong {
        id: identity.id.clone(),
        title: tags.title,
        artist: tags.artist,
        album: tags.album,
        album_artist: tags.album_artist,
        track: tags.track,
        disc: tags.disc,
        year: tags.year,
        genre: tags.genre,
        duration: identity.duration,
        name,
        folder,
        cover,
        replay_gain: tags.replay_gain,
    };
    let new = inner
        .db
        .lock()
        .put_file(&key, size, modified, &song)
        .map_err(|error| error.to_string())?;
    Ok(Outcome::Changed {
        song: identity.id,
        new,
    })
}
