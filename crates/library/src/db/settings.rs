use rusqlite::{OptionalExtension, params};
use serde::Serialize;
use serde::de::DeserializeOwned;

use super::{Db, DbResult};
use crate::model::{AlbumSettings, Edits};

fn to_json(value: &impl Serialize) -> String {
    serde_json::to_string(value).expect("library values serialize")
}

impl Db {
    pub fn edits(&self, song: &str) -> DbResult<Edits> {
        let value: Option<String> = self
            .connection
            .query_row("SELECT value FROM edits WHERE song = ?1", [song], |row| {
                row.get(0)
            })
            .optional()?;
        Ok(value
            .and_then(|value| serde_json::from_str(&value).ok())
            .unwrap_or_default())
    }

    pub fn set_edits(&self, song: &str, edits: &Edits) -> DbResult<()> {
        if edits.is_empty() {
            self.connection
                .execute("DELETE FROM edits WHERE song = ?1", [song])?;
        } else {
            self.connection.execute(
                "INSERT OR REPLACE INTO edits (song, value) VALUES (?1, ?2)",
                params![song, to_json(edits)],
            )?;
        }
        Ok(())
    }

    pub fn set_kept_cover(&self, target: &str, cover: Option<&str>) -> DbResult<()> {
        match cover {
            Some(cover) => self.connection.execute(
                "INSERT OR REPLACE INTO kept_covers (target, cover) VALUES (?1, ?2)",
                params![target, cover],
            )?,
            None => self
                .connection
                .execute("DELETE FROM kept_covers WHERE target = ?1", [target])?,
        };
        Ok(())
    }

    pub fn album_settings(&self, key: &str) -> DbResult<AlbumSettings> {
        let transitions: Option<bool> = self
            .connection
            .query_row(
                "SELECT transitions FROM albums WHERE key = ?1",
                [key],
                |row| row.get(0),
            )
            .optional()?;
        Ok(AlbumSettings {
            transitions: transitions.unwrap_or(false),
        })
    }

    pub fn set_album_settings(&self, key: &str, settings: &AlbumSettings) -> DbResult<()> {
        self.connection.execute(
            "INSERT OR REPLACE INTO albums (key, transitions) VALUES (?1, ?2)",
            params![key, settings.transitions],
        )?;
        Ok(())
    }

    pub fn setting<T: DeserializeOwned>(&self, key: &str) -> DbResult<Option<T>> {
        let value: Option<String> = self
            .connection
            .query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
                row.get(0)
            })
            .optional()?;
        Ok(value.and_then(|value| serde_json::from_str(&value).ok()))
    }

    pub fn set_setting(&self, key: &str, value: &impl Serialize) -> DbResult<()> {
        self.connection.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, to_json(value)],
        )?;
        Ok(())
    }
}
