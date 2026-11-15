mod analysis;
pub mod playlists;
mod settings;
mod songs;

use std::path::Path;

use rusqlite::Connection;

pub use songs::NewSong;

const SCHEMA: &str = "
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS folders (
    path TEXT PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS songs (
    id TEXT PRIMARY KEY,
    title TEXT,
    artist TEXT,
    album TEXT,
    album_artist TEXT,
    track INTEGER,
    disc INTEGER,
    year INTEGER,
    genre TEXT,
    duration REAL NOT NULL,
    name TEXT NOT NULL,
    folder TEXT NOT NULL,
    cover TEXT,
    replay_gain REAL,
    added INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS files (
    path TEXT PRIMARY KEY,
    song TEXT NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
    size INTEGER NOT NULL,
    modified INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS files_song ON files(song);

CREATE TABLE IF NOT EXISTS analysis (
    song TEXT PRIMARY KEY,
    version INTEGER NOT NULL,
    loudness REAL NOT NULL,
    peak REAL NOT NULL,
    tempo REAL,
    beats BLOB NOT NULL,
    downbeat INTEGER NOT NULL,
    sound_start REAL NOT NULL,
    sound_end REAL NOT NULL,
    fade_start REAL,
    waveform BLOB NOT NULL,
    fingerprint BLOB NOT NULL
);

CREATE TABLE IF NOT EXISTS edits (
    song TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS kept_covers (
    target TEXT PRIMARY KEY,
    cover TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS albums (
    key TEXT PRIMARY KEY,
    transitions INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS playlists (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    cover TEXT,
    transition TEXT,
    created INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS entries (
    id INTEGER PRIMARY KEY,
    playlist INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
    position REAL NOT NULL,
    song TEXT,
    missing TEXT
);
CREATE INDEX IF NOT EXISTS entries_playlist ON entries(playlist, position);

CREATE TABLE IF NOT EXISTS pairs (
    playlist INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
    from_song TEXT NOT NULL,
    to_song TEXT NOT NULL,
    transition TEXT NOT NULL,
    PRIMARY KEY (playlist, from_song, to_song)
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
";

pub type DbResult<T> = rusqlite::Result<T>;

pub struct Db {
    connection: Connection,
}

impl Db {
    pub fn open(path: &Path) -> DbResult<Self> {
        let connection = Connection::open(path)?;
        connection.execute_batch(SCHEMA)?;
        Ok(Self { connection })
    }
}

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs() as i64)
}
