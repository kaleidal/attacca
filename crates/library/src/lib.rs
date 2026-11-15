mod covers;
mod db;
mod listen;
mod model;
mod scan;
mod share;

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crossbeam_channel::Sender;
use parking_lot::Mutex;
use serde::Serialize;
use serde::de::DeserializeOwned;

use covers::Covers;
pub use covers::Crop;
use db::Db;
pub use model::*;
use scan::Scan;
pub use share::ImportPreview;

pub fn is_audio(path: &Path) -> bool {
    attacca_media::is_supported(path)
}

#[derive(Debug)]
pub struct LibraryError(String);

impl LibraryError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for LibraryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl From<rusqlite::Error> for LibraryError {
    fn from(error: rusqlite::Error) -> Self {
        Self(format!("The library couldn't be read: {error}"))
    }
}

#[derive(Clone, Debug)]
pub enum LibraryEvent {
    Songs {
        changed: Vec<String>,
        removed: Vec<String>,
    },
    Analysed(String),
    Scanning(bool),
    Playlists,
}

pub enum CoverTarget {
    Song(String),
    Album(String),
    Playlist(i64),
}

pub struct Inner {
    db: Mutex<Db>,
    covers: Covers,
    scans: Sender<Scan>,
    queue: listen::Queue,
    on_event: Box<dyn Fn(LibraryEvent) + Send + Sync>,
}

impl Inner {
    fn emit(&self, event: LibraryEvent) {
        (self.on_event)(event);
    }
}

/// The songs in your folders, what Attacca learned about them, and your edits and playlists.
pub struct Library {
    inner: Arc<Inner>,
    scans: Mutex<Option<crossbeam_channel::Receiver<Scan>>>,
}

impl Library {
    pub fn open(
        dir: &Path,
        on_event: impl Fn(LibraryEvent) + Send + Sync + 'static,
    ) -> Result<Library, LibraryError> {
        std::fs::create_dir_all(dir).map_err(|error| {
            LibraryError::new(format!("Couldn't create {}: {error}", dir.display()))
        })?;
        let covers = Covers::new(dir.join("covers"))
            .map_err(|error| LibraryError::new(error.to_string()))?;
        let (scans, receiver) = crossbeam_channel::unbounded();
        let inner = Arc::new(Inner {
            db: Mutex::new(Db::open(&dir.join("library.sqlite"))?),
            covers,
            scans,
            queue: listen::Queue::default(),
            on_event: Box::new(on_event),
        });
        Ok(Library {
            inner,
            scans: Mutex::new(Some(receiver)),
        })
    }

    /// Scans the folders, starts following them, and starts analysing songs.
    pub fn start(&self) {
        let Some(receiver) = self.scans.lock().take() else {
            return;
        };
        let inner = Arc::clone(&self.inner);
        std::thread::Builder::new()
            .name("attacca-scan".into())
            .spawn(move || scan::run(&inner, receiver))
            .expect("the scan thread starts");
        for folder in self.folders() {
            let _ = self.inner.scans.send(Scan::Folder(PathBuf::from(folder)));
        }
        self.inner
            .queue
            .push(self.inner.db.lock().unanalysed().unwrap_or_default());
        listen::workers(&self.inner);
    }

    pub fn covers_dir(&self) -> PathBuf {
        self.inner.covers.dir().to_path_buf()
    }

    pub fn folders(&self) -> Vec<String> {
        self.inner.db.lock().folders().unwrap_or_default()
    }

    pub fn add_folder(&self, path: &Path) -> Result<(), LibraryError> {
        let text = path.to_string_lossy();
        self.inner.db.lock().add_folder(&text)?;
        let _ = self.inner.scans.send(Scan::Folder(path.to_path_buf()));
        Ok(())
    }

    pub fn remove_folder(&self, path: &Path) -> Result<(), LibraryError> {
        self.inner
            .db
            .lock()
            .remove_folder(&path.to_string_lossy())?;
        let _ = self.inner.scans.send(Scan::Forget(path.to_path_buf()));
        self.inner.emit(LibraryEvent::Songs {
            changed: Vec::new(),
            removed: Vec::new(),
        });
        Ok(())
    }

    /// Adds one file to the library, wherever it is, and returns its song.
    pub fn include(&self, path: &Path) -> Result<String, LibraryError> {
        let path = path.canonicalize().map_err(|error| {
            LibraryError::new(format!("Couldn't open {}: {error}", path.display()))
        })?;
        let song = match scan::read_file(&self.inner, &path).map_err(LibraryError::new)? {
            scan::Outcome::Changed { song, new } => {
                if new {
                    self.inner.queue.push(vec![song.clone()]);
                }
                self.inner.emit(LibraryEvent::Songs {
                    changed: vec![song.clone()],
                    removed: Vec::new(),
                });
                song
            }
            scan::Outcome::Unchanged => self
                .inner
                .db
                .lock()
                .song_at(&path.to_string_lossy())?
                .ok_or_else(|| LibraryError::new("That file isn't in the library"))?,
        };
        Ok(song)
    }

    pub fn songs(&self) -> Result<Vec<Song>, LibraryError> {
        Ok(self.inner.db.lock().songs()?)
    }

    pub fn song(&self, id: &str) -> Option<Song> {
        self.inner.db.lock().song(id).ok().flatten()
    }

    pub fn detail(&self, id: &str) -> Option<SongDetail> {
        let db = self.inner.db.lock();
        let song = db.song(id).ok().flatten()?;
        Some(SongDetail {
            edits: db.edits(id).unwrap_or_default(),
            listening: db
                .analysis(id)
                .ok()
                .flatten()
                .map(|stored| stored.listening),
            path: db
                .paths_of(id)
                .ok()
                .and_then(|paths| paths.into_iter().next()),
            song,
        })
    }

    pub fn path(&self, id: &str) -> Option<PathBuf> {
        let paths = self.inner.db.lock().paths_of(id).ok()?;
        paths
            .into_iter()
            .map(PathBuf::from)
            .find(|path| path.exists())
    }

    pub fn listening(&self, id: &str) -> Option<Listening> {
        self.inner
            .db
            .lock()
            .analysis(id)
            .ok()
            .flatten()
            .map(|stored| stored.listening)
    }

    pub fn waveform(&self, id: &str) -> Option<Vec<u8>> {
        self.inner.db.lock().waveform(id).ok().flatten()
    }

    /// Analyses a song before the others, because someone is about to hear it.
    pub fn hurry(&self, id: &str) {
        if self.inner.db.lock().analysis_version(id).ok().flatten()
            != Some(attacca_analysis::VERSION)
        {
            self.inner.queue.hurry(id);
        }
    }

    pub fn edits(&self, id: &str) -> Edits {
        self.inner.db.lock().edits(id).unwrap_or_default()
    }

    pub fn set_edits(&self, id: &str, edits: &Edits) -> Result<(), LibraryError> {
        self.inner.db.lock().set_edits(id, edits)?;
        self.inner.emit(LibraryEvent::Songs {
            changed: vec![id.to_string()],
            removed: Vec::new(),
        });
        Ok(())
    }

    pub fn replay_gain(&self, id: &str) -> Option<f32> {
        self.inner.db.lock().replay_gain(id).ok().flatten()
    }

    pub fn loudness(&self, id: &str) -> Option<f32> {
        self.inner
            .db
            .lock()
            .loudness(id)
            .ok()
            .flatten()
            .map(|(loudness, _)| loudness)
    }

    /// The loudness of a whole album, from the energy of every analysed song on it.
    pub fn album_loudness(&self, key: &str) -> Option<f32> {
        let db = self.inner.db.lock();
        let songs = db.album_songs(key).ok()?;
        let (mut energy, mut length) = (0.0f64, 0.0f64);
        for song in songs {
            if let Some((loudness, duration)) = db.loudness(&song).ok().flatten() {
                energy += 10f64.powf(f64::from(loudness) / 10.0) * duration;
                length += duration;
            }
        }
        (length > 0.0).then(|| (10.0 * (energy / length).log10()) as f32)
    }

    pub fn album_settings(&self, key: &str) -> AlbumSettings {
        self.inner.db.lock().album_settings(key).unwrap_or_default()
    }

    pub fn set_album_settings(
        &self,
        key: &str,
        settings: &AlbumSettings,
    ) -> Result<(), LibraryError> {
        Ok(self.inner.db.lock().set_album_settings(key, settings)?)
    }

    pub fn playlists(&self) -> Result<Vec<PlaylistSummary>, LibraryError> {
        Ok(self.inner.db.lock().playlists()?)
    }

    pub fn playlist(&self, id: i64) -> Option<Playlist> {
        self.inner.db.lock().playlist(id).ok().flatten()
    }

    fn changed_playlists<T>(&self, result: rusqlite::Result<T>) -> Result<T, LibraryError> {
        let value = result?;
        self.inner.emit(LibraryEvent::Playlists);
        Ok(value)
    }

    pub fn create_playlist(&self, name: &str, songs: &[String]) -> Result<i64, LibraryError> {
        let db = self.inner.db.lock();
        let id = db.create_playlist(name)?;
        let result = db.append_songs(id, songs).map(|()| id);
        drop(db);
        self.changed_playlists(result)
    }

    pub fn rename_playlist(&self, id: i64, name: &str) -> Result<(), LibraryError> {
        let result = self.inner.db.lock().rename_playlist(id, name);
        self.changed_playlists(result)
    }

    pub fn delete_playlist(&self, id: i64) -> Result<(), LibraryError> {
        let result = self.inner.db.lock().delete_playlist(id);
        self.changed_playlists(result)
    }

    pub fn append_to_playlist(&self, id: i64, songs: &[String]) -> Result<(), LibraryError> {
        let result = self.inner.db.lock().append_songs(id, songs);
        self.changed_playlists(result)
    }

    pub fn order_playlist(&self, id: i64, entries: &[i64]) -> Result<(), LibraryError> {
        let result = self.inner.db.lock().order_entries(id, entries);
        self.changed_playlists(result)
    }

    pub fn set_playlist_transition(
        &self,
        id: i64,
        transition: Option<&Transition>,
    ) -> Result<(), LibraryError> {
        let result = self.inner.db.lock().set_playlist_transition(id, transition);
        self.changed_playlists(result)
    }

    pub fn set_pair(
        &self,
        id: i64,
        from: &str,
        to: &str,
        transition: Option<&Transition>,
    ) -> Result<(), LibraryError> {
        let result = self.inner.db.lock().set_pair(id, from, to, transition);
        self.changed_playlists(result)
    }

    /// Sets a cover from an image, cropped to a square. Song and album covers go into the files'
    /// tags when `into_files` is set, and are kept by Attacca otherwise.
    pub fn set_cover(
        &self,
        target: &CoverTarget,
        image: &[u8],
        crop: Crop,
        into_files: bool,
    ) -> Result<String, LibraryError> {
        if !covers::is_image(image) {
            return Err(LibraryError::new("That isn't an image Attacca can use"));
        }
        let jpeg = covers::square(image, crop)?;
        let name = self
            .inner
            .covers
            .store(&jpeg)
            .ok_or_else(|| LibraryError::new("The cover couldn't be saved"))?;
        let songs = match target {
            CoverTarget::Playlist(id) => {
                let result = self
                    .inner
                    .db
                    .lock()
                    .set_kept_cover(&format!("playlist:{id}"), Some(&name));
                self.changed_playlists(result)?;
                return Ok(name);
            }
            CoverTarget::Song(id) => vec![id.clone()],
            CoverTarget::Album(key) => self.inner.db.lock().album_songs(key)?,
        };
        let db = self.inner.db.lock();
        let mut written = Vec::new();
        for song in &songs {
            if into_files {
                for path in db.paths_of(song)? {
                    covers::write_into(Path::new(&path), &jpeg)?;
                    written.push(PathBuf::from(path));
                }
                db.set_kept_cover(&format!("song:{song}"), None)?;
                db.connection().execute(
                    "UPDATE songs SET cover = ?2 WHERE id = ?1",
                    rusqlite::params![song, name],
                )?;
            } else {
                db.set_kept_cover(&format!("song:{song}"), Some(&name))?;
            }
        }
        drop(db);
        let _ = self.inner.scans.send(Scan::Paths(written));
        self.inner.emit(LibraryEvent::Songs {
            changed: songs,
            removed: Vec::new(),
        });
        Ok(name)
    }

    pub fn export(&self, playlist: i64, path: &Path) -> Result<(), LibraryError> {
        self.inner.export(playlist, path)
    }

    pub fn preview_import(&self, path: &Path) -> Result<ImportPreview, LibraryError> {
        self.inner.preview_import(path)
    }

    pub fn import(&self, path: &Path, keep_own: bool) -> Result<i64, LibraryError> {
        self.inner.import(path, keep_own)
    }

    pub fn setting<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.inner.db.lock().setting(key).ok().flatten()
    }

    pub fn set_setting(&self, key: &str, value: &impl Serialize) -> Result<(), LibraryError> {
        Ok(self.inner.db.lock().set_setting(key, value)?)
    }
}
