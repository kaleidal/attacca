use rusqlite::{OptionalExtension, Row, params};

use super::{Db, DbResult, now};
use crate::model::{Edits, Song};

pub struct FileRecord {
    pub size: i64,
    pub modified: i64,
}

/// What a scan reads from a file.
pub struct NewSong {
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
    pub name: String,
    pub folder: String,
    pub cover: Option<String>,
    pub replay_gain: Option<f32>,
}

const SONG_COLUMNS: &str = "s.id, s.title, s.artist, s.album, s.album_artist, s.track, s.disc, s.year, s.genre, s.duration,
    s.name, s.folder, COALESCE(k.cover, s.cover), s.added, a.tempo, a.version IS NOT NULL, e.value";
const SONG_JOINS: &str = "FROM songs s
    LEFT JOIN analysis a ON a.song = s.id
    LEFT JOIN edits e ON e.song = s.id
    LEFT JOIN kept_covers k ON k.target = 'song:' || s.id";

fn song_from(row: &Row<'_>) -> rusqlite::Result<Song> {
    let edits: Option<String> = row.get(16)?;
    let transition = edits
        .as_deref()
        .and_then(|edits| serde_json::from_str::<Edits>(edits).ok())
        .and_then(|edits| edits.transition);
    Ok(Song {
        id: row.get(0)?,
        title: row.get(1)?,
        artist: row.get(2)?,
        album: row.get(3)?,
        album_artist: row.get(4)?,
        track: row.get(5)?,
        disc: row.get(6)?,
        year: row.get(7)?,
        genre: row.get(8)?,
        duration: row.get(9)?,
        name: row.get(10)?,
        folder: row.get(11)?,
        cover: row.get(12)?,
        added: row.get(13)?,
        tempo: row.get(14)?,
        analysed: row.get(15)?,
        edited: edits.is_some(),
        transition,
    })
}

impl Db {
    pub fn folders(&self) -> DbResult<Vec<String>> {
        let mut statement = self
            .connection
            .prepare("SELECT path FROM folders ORDER BY path")?;
        statement.query_map([], |row| row.get(0))?.collect()
    }

    pub fn add_folder(&self, path: &str) -> DbResult<()> {
        self.connection
            .execute("INSERT OR IGNORE INTO folders (path) VALUES (?1)", [path])?;
        Ok(())
    }

    pub fn remove_folder(&self, path: &str) -> DbResult<()> {
        self.connection
            .execute("DELETE FROM folders WHERE path = ?1", [path])?;
        let pattern = format!("{}/%", path.trim_end_matches('/'));
        self.connection
            .execute("DELETE FROM files WHERE path LIKE ?1", [pattern])?;
        self.remove_orphans()
    }

    pub fn songs(&self) -> DbResult<Vec<Song>> {
        let sql = format!("SELECT {SONG_COLUMNS} {SONG_JOINS} ORDER BY s.added DESC");
        let mut statement = self.connection.prepare(&sql)?;
        statement.query_map([], song_from)?.collect()
    }

    pub fn song(&self, id: &str) -> DbResult<Option<Song>> {
        let sql = format!("SELECT {SONG_COLUMNS} {SONG_JOINS} WHERE s.id = ?1");
        self.connection.query_row(&sql, [id], song_from).optional()
    }

    pub fn file(&self, path: &str) -> DbResult<Option<FileRecord>> {
        self.connection
            .query_row(
                "SELECT size, modified FROM files WHERE path = ?1",
                [path],
                |row| {
                    Ok(FileRecord {
                        size: row.get(0)?,
                        modified: row.get(1)?,
                    })
                },
            )
            .optional()
    }

    pub fn song_at(&self, path: &str) -> DbResult<Option<String>> {
        self.connection
            .query_row("SELECT song FROM files WHERE path = ?1", [path], |row| {
                row.get(0)
            })
            .optional()
    }

    pub fn paths_of(&self, song: &str) -> DbResult<Vec<String>> {
        let mut statement = self
            .connection
            .prepare("SELECT path FROM files WHERE song = ?1 ORDER BY path")?;
        statement.query_map([song], |row| row.get(0))?.collect()
    }

    pub fn paths_under(&self, folder: &str) -> DbResult<Vec<String>> {
        let pattern = format!("{}/%", folder.trim_end_matches('/'));
        let mut statement = self
            .connection
            .prepare("SELECT path FROM files WHERE path LIKE ?1")?;
        statement.query_map([pattern], |row| row.get(0))?.collect()
    }

    /// Records a file and the song in it. Returns whether the song is new to the library.
    pub fn put_file(
        &mut self,
        path: &str,
        size: i64,
        modified: i64,
        song: &NewSong,
    ) -> DbResult<bool> {
        let transaction = self.connection.transaction()?;
        let existed = transaction
            .query_row("SELECT 1 FROM songs WHERE id = ?1", [&song.id], |_| Ok(()))
            .optional()?
            .is_some();
        transaction.execute(
            "INSERT INTO songs (id, title, artist, album, album_artist, track, disc, year, genre, duration, name, folder, cover, replay_gain, added)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
             ON CONFLICT(id) DO UPDATE SET title = ?2, artist = ?3, album = ?4, album_artist = ?5, track = ?6, disc = ?7,
                year = ?8, genre = ?9, duration = ?10, name = ?11, folder = ?12, cover = ?13, replay_gain = ?14",
            params![
                song.id, song.title, song.artist, song.album, song.album_artist, song.track, song.disc, song.year,
                song.genre, song.duration, song.name, song.folder, song.cover, song.replay_gain, now()
            ],
        )?;
        let previous: Option<String> = transaction
            .query_row("SELECT song FROM files WHERE path = ?1", [path], |row| {
                row.get(0)
            })
            .optional()?;
        transaction.execute(
            "INSERT INTO files (path, song, size, modified) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(path) DO UPDATE SET song = ?2, size = ?3, modified = ?4",
            params![path, song.id, size, modified],
        )?;
        if let Some(previous) = previous.filter(|previous| previous != &song.id) {
            transaction.execute(
                "DELETE FROM songs WHERE id = ?1 AND NOT EXISTS (SELECT 1 FROM files WHERE song = ?1)",
                [previous],
            )?;
        }
        transaction.commit()?;
        Ok(!existed)
    }

    /// Forgets files that are gone. Returns the songs that left the library with them.
    pub fn remove_files(&mut self, paths: &[String]) -> DbResult<Vec<String>> {
        let transaction = self.connection.transaction()?;
        let mut songs = Vec::new();
        for path in paths {
            let song: Option<String> = transaction
                .query_row(
                    "DELETE FROM files WHERE path = ?1 RETURNING song",
                    [path],
                    |row| row.get(0),
                )
                .optional()?;
            songs.extend(song);
        }
        transaction.commit()?;
        let mut gone = Vec::new();
        for song in songs {
            let deleted = self.connection.execute(
                "DELETE FROM songs WHERE id = ?1 AND NOT EXISTS (SELECT 1 FROM files WHERE song = ?1)",
                [&song],
            )?;
            if deleted > 0 {
                gone.push(song);
            }
        }
        Ok(gone)
    }

    fn remove_orphans(&self) -> DbResult<()> {
        self.connection.execute(
            "DELETE FROM songs WHERE NOT EXISTS (SELECT 1 FROM files WHERE files.song = songs.id)",
            [],
        )?;
        Ok(())
    }

    pub fn replay_gain(&self, song: &str) -> DbResult<Option<f32>> {
        self.connection
            .query_row(
                "SELECT replay_gain FROM songs WHERE id = ?1",
                [song],
                |row| row.get(0),
            )
            .optional()
            .map(Option::flatten)
    }

    pub fn album_songs(&self, key: &str) -> DbResult<Vec<String>> {
        let songs = self.songs()?;
        Ok(songs
            .into_iter()
            .filter(|song| song.album_key().as_deref() == Some(key))
            .map(|song| song.id)
            .collect())
    }
}
