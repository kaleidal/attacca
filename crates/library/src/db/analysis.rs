use attacca_analysis::Analysis;
use rusqlite::{OptionalExtension, params};

use super::{Db, DbResult};
use crate::model::{Listening, Rhythm};

fn floats_to_blob(values: impl IntoIterator<Item = f32>) -> Vec<u8> {
    values.into_iter().flat_map(f32::to_le_bytes).collect()
}

fn blob_to_floats(blob: &[u8]) -> Vec<f32> {
    blob.as_chunks::<4>()
        .0
        .iter()
        .map(|bytes| f32::from_le_bytes(*bytes))
        .collect()
}

fn blob_to_words(blob: &[u8]) -> Vec<u32> {
    blob.as_chunks::<4>()
        .0
        .iter()
        .map(|bytes| u32::from_le_bytes(*bytes))
        .collect()
}

/// Analysis as kept in the library.
pub struct Stored {
    pub listening: Listening,
    pub fingerprint: Vec<u32>,
}

impl Db {
    pub fn put_analysis(&self, song: &str, analysis: &Analysis) -> DbResult<()> {
        self.connection.execute(
            "INSERT OR REPLACE INTO analysis
             (song, version, loudness, peak, tempo, beats, downbeat, sound_start, sound_end, fade_start, waveform, fingerprint)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                song,
                attacca_analysis::VERSION,
                analysis.loudness,
                analysis.peak,
                analysis.tempo,
                floats_to_blob(analysis.beats.iter().copied()),
                analysis.downbeat,
                analysis.sound_start,
                analysis.sound_end,
                analysis.fade_start,
                analysis.waveform,
                analysis.fingerprint.iter().flat_map(|word| word.to_le_bytes()).collect::<Vec<u8>>(),
            ],
        )?;
        Ok(())
    }

    pub fn analysis(&self, song: &str) -> DbResult<Option<Stored>> {
        self.connection
            .query_row(
                "SELECT loudness, peak, tempo, beats, downbeat, sound_start, sound_end, fade_start, fingerprint
                 FROM analysis WHERE song = ?1",
                [song],
                |row| {
                    let tempo: Option<f32> = row.get(2)?;
                    let beats: Vec<u8> = row.get(3)?;
                    let sound_start: f32 = row.get(5)?;
                    let sound_end: f32 = row.get(6)?;
                    let fade_start: Option<f32> = row.get(7)?;
                    let fingerprint: Vec<u8> = row.get(8)?;
                    Ok(Stored {
                        listening: Listening {
                            loudness: row.get(0)?,
                            peak: row.get(1)?,
                            rhythm: tempo.map(|tempo| Rhythm {
                                tempo,
                                beats: blob_to_floats(&beats),
                                downbeat: row.get(4).unwrap_or(0),
                            }),
                            suggested_start: (sound_start - 0.01).max(0.0),
                            suggested_end: fade_start.unwrap_or(sound_end),
                        },
                        fingerprint: blob_to_words(&fingerprint),
                    })
                },
            )
            .optional()
    }

    pub fn analysis_version(&self, song: &str) -> DbResult<Option<u32>> {
        self.connection
            .query_row(
                "SELECT version FROM analysis WHERE song = ?1",
                [song],
                |row| row.get(0),
            )
            .optional()
    }

    pub fn waveform(&self, song: &str) -> DbResult<Option<Vec<u8>>> {
        self.connection
            .query_row(
                "SELECT waveform FROM analysis WHERE song = ?1",
                [song],
                |row| row.get(0),
            )
            .optional()
    }

    /// Songs that were never analysed, or analysed by an older version.
    pub fn unanalysed(&self) -> DbResult<Vec<String>> {
        let mut statement = self.connection.prepare(
            "SELECT s.id FROM songs s LEFT JOIN analysis a ON a.song = s.id
             WHERE a.version IS NULL OR a.version < ?1 ORDER BY s.added DESC",
        )?;
        statement
            .query_map([attacca_analysis::VERSION], |row| row.get(0))?
            .collect()
    }

    pub fn loudness(&self, song: &str) -> DbResult<Option<(f32, f64)>> {
        self.connection
            .query_row(
                "SELECT a.loudness, s.duration FROM analysis a JOIN songs s ON s.id = a.song WHERE a.song = ?1",
                [song],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
    }

    /// Fingerprints of analysed songs lasting about `seconds`.
    pub fn fingerprints_near(
        &self,
        seconds: f64,
        tolerance: f64,
    ) -> DbResult<Vec<(String, Vec<u32>)>> {
        let mut statement = self.connection.prepare(
            "SELECT a.song, a.fingerprint FROM analysis a JOIN songs s ON s.id = a.song
             WHERE s.duration BETWEEN ?1 AND ?2",
        )?;
        statement
            .query_map(params![seconds - tolerance, seconds + tolerance], |row| {
                let blob: Vec<u8> = row.get(1)?;
                Ok((row.get(0)?, blob_to_words(&blob)))
            })?
            .collect()
    }
}
