use rusqlite::{OptionalExtension, params};

use super::{Db, DbResult, now};
use crate::model::{Entry, Missing, Pair, Playlist, PlaylistSummary, Transition};

/// A shared song not yet in the library, kept with what it needs to be recognised and played as
/// its sender set it up.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Awaited {
    pub missing: Missing,
    pub fingerprint: Vec<u32>,
    pub edits: crate::model::Edits,
}

fn transition_from(value: Option<String>) -> Option<Transition> {
    value.and_then(|value| serde_json::from_str(&value).ok())
}

impl Db {
    pub fn playlists(&self) -> DbResult<Vec<PlaylistSummary>> {
        let mut statement = self.connection.prepare(
            "SELECT p.id, p.name, COALESCE(k.cover, p.cover), (SELECT COUNT(*) FROM entries e WHERE e.playlist = p.id)
             FROM playlists p LEFT JOIN kept_covers k ON k.target = 'playlist:' || p.id ORDER BY p.created DESC",
        )?;
        let mut summaries: Vec<PlaylistSummary> = statement
            .query_map([], |row| {
                Ok(PlaylistSummary {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    cover: row.get(2)?,
                    count: row.get(3)?,
                    covers: Vec::new(),
                })
            })?
            .collect::<DbResult<_>>()?;
        let mut covers = self.connection.prepare(
            "SELECT DISTINCT COALESCE(k.cover, s.cover) FROM entries e JOIN songs s ON s.id = e.song
             LEFT JOIN kept_covers k ON k.target = 'song:' || s.id
             WHERE e.playlist = ?1 AND COALESCE(k.cover, s.cover) IS NOT NULL ORDER BY e.position LIMIT 4",
        )?;
        for summary in &mut summaries {
            summary.covers = covers
                .query_map([summary.id], |row| row.get(0))?
                .collect::<DbResult<_>>()?;
        }
        Ok(summaries)
    }

    pub fn playlist(&self, id: i64) -> DbResult<Option<Playlist>> {
        let Some((name, cover, transition)) = self
            .connection
            .query_row(
                "SELECT p.name, COALESCE(k.cover, p.cover), p.transition FROM playlists p
                 LEFT JOIN kept_covers k ON k.target = 'playlist:' || p.id WHERE p.id = ?1",
                [id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, Option<String>>(2)?,
                    ))
                },
            )
            .optional()?
        else {
            return Ok(None);
        };
        let mut entries = self.connection.prepare(
            "SELECT id, song, missing FROM entries WHERE playlist = ?1 ORDER BY position",
        )?;
        let entries = entries
            .query_map([id], |row| {
                let missing: Option<String> = row.get(2)?;
                Ok(Entry {
                    id: row.get(0)?,
                    song: row.get(1)?,
                    missing: missing
                        .and_then(|missing| serde_json::from_str::<Awaited>(&missing).ok())
                        .map(|awaited| awaited.missing),
                })
            })?
            .collect::<DbResult<_>>()?;
        let mut pairs = self
            .connection
            .prepare("SELECT from_song, to_song, transition FROM pairs WHERE playlist = ?1")?;
        let pairs = pairs
            .query_map([id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?
            .filter_map(|row| {
                let (from, to, transition) = row.ok()?;
                Some(Pair {
                    from,
                    to,
                    transition: serde_json::from_str(&transition).ok()?,
                })
            })
            .collect();
        Ok(Some(Playlist {
            id,
            name,
            cover,
            transition: transition_from(transition),
            entries,
            pairs,
        }))
    }

    pub fn create_playlist(&self, name: &str) -> DbResult<i64> {
        self.connection.execute(
            "INSERT INTO playlists (name, created) VALUES (?1, ?2)",
            params![name, now()],
        )?;
        Ok(self.connection.last_insert_rowid())
    }

    pub fn rename_playlist(&self, id: i64, name: &str) -> DbResult<()> {
        self.connection.execute(
            "UPDATE playlists SET name = ?2 WHERE id = ?1",
            params![id, name],
        )?;
        Ok(())
    }

    pub fn delete_playlist(&self, id: i64) -> DbResult<()> {
        self.connection
            .execute("DELETE FROM playlists WHERE id = ?1", [id])?;
        self.connection.execute(
            "DELETE FROM kept_covers WHERE target = ?1",
            [format!("playlist:{id}")],
        )?;
        Ok(())
    }

    pub fn set_playlist_transition(
        &self,
        id: i64,
        transition: Option<&Transition>,
    ) -> DbResult<()> {
        let value = transition
            .map(|transition| serde_json::to_string(transition).expect("transitions serialize"));
        self.connection.execute(
            "UPDATE playlists SET transition = ?2 WHERE id = ?1",
            params![id, value],
        )?;
        Ok(())
    }

    pub fn set_pair(
        &self,
        playlist: i64,
        from: &str,
        to: &str,
        transition: Option<&Transition>,
    ) -> DbResult<()> {
        match transition {
            Some(transition) => self.connection.execute(
                "INSERT OR REPLACE INTO pairs (playlist, from_song, to_song, transition) VALUES (?1, ?2, ?3, ?4)",
                params![playlist, from, to, serde_json::to_string(transition).expect("transitions serialize")],
            )?,
            None => self.connection.execute(
                "DELETE FROM pairs WHERE playlist = ?1 AND from_song = ?2 AND to_song = ?3",
                params![playlist, from, to],
            )?,
        };
        Ok(())
    }

    fn last_position(&self, playlist: i64) -> DbResult<f64> {
        self.connection.query_row(
            "SELECT COALESCE(MAX(position), 0) FROM entries WHERE playlist = ?1",
            [playlist],
            |row| row.get(0),
        )
    }

    pub fn append_songs(&self, playlist: i64, songs: &[String]) -> DbResult<()> {
        let mut position = self.last_position(playlist)?;
        for song in songs {
            position += 1.0;
            self.connection.execute(
                "INSERT INTO entries (playlist, position, song) VALUES (?1, ?2, ?3)",
                params![playlist, position, song],
            )?;
        }
        Ok(())
    }

    pub fn append_awaited(&self, playlist: i64, awaited: &Awaited) -> DbResult<()> {
        let position = self.last_position(playlist)? + 1.0;
        self.connection.execute(
            "INSERT INTO entries (playlist, position, missing) VALUES (?1, ?2, ?3)",
            params![
                playlist,
                position,
                serde_json::to_string(awaited).expect("entries serialize")
            ],
        )?;
        Ok(())
    }

    /// Puts the entries in the given order. Entries left out are removed.
    pub fn order_entries(&mut self, playlist: i64, entries: &[i64]) -> DbResult<()> {
        let transaction = self.connection.transaction()?;
        transaction.execute(
            &format!(
                "DELETE FROM entries WHERE playlist = ?1 AND id NOT IN ({})",
                entries
                    .iter()
                    .map(i64::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            [playlist],
        )?;
        for (index, entry) in entries.iter().enumerate() {
            transaction.execute(
                "UPDATE entries SET position = ?3 WHERE playlist = ?1 AND id = ?2",
                params![playlist, entry, index as f64 + 1.0],
            )?;
        }
        transaction.commit()
    }

    pub fn awaited(&self) -> DbResult<Vec<(i64, Awaited)>> {
        let mut statement = self
            .connection
            .prepare("SELECT id, missing FROM entries WHERE missing IS NOT NULL")?;
        let rows = statement.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;
        Ok(rows
            .filter_map(|row| {
                let (id, missing) = row.ok()?;
                Some((id, serde_json::from_str(&missing).ok()?))
            })
            .collect())
    }

    pub fn arrive(&self, entry: i64, song: &str) -> DbResult<Option<i64>> {
        self.connection
            .query_row(
                "UPDATE entries SET song = ?2, missing = NULL WHERE id = ?1 RETURNING playlist",
                params![entry, song],
                |row| row.get(0),
            )
            .optional()
    }
}
