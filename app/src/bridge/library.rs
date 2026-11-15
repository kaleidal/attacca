use std::path::PathBuf;

use attacca_library::{AlbumSettings, Edits, Song, SongDetail};
use sabine::prelude::*;
use serde::Deserialize;
use specta::Type;

use super::{Nothing, SongId};
use crate::player::Context;
use crate::state::app;

#[derive(Deserialize, Type)]
struct FolderPath {
    path: String,
}

#[derive(Deserialize, Type)]
struct Paths {
    paths: Vec<String>,
}

#[derive(Deserialize, Type)]
struct AlbumKey {
    key: String,
}

#[derive(Deserialize, Type)]
struct SetAlbum {
    key: String,
    settings: AlbumSettings,
}

#[derive(Deserialize, Type)]
struct SetEdits {
    song: String,
    edits: Edits,
}

fn failed(error: impl std::fmt::Display) -> BridgeError {
    BridgeError::new(error.to_string())
}

fn waveform(command: BridgeCommand) -> BridgeResult {
    let id = command.params["id"]
        .as_str()
        .ok_or_else(|| BridgeError::new("`id` names a song"))?;
    Ok(BridgeResponse::bytes(
        app().library.waveform(id).unwrap_or_default(),
    ))
}

async fn add_folder(_: Nothing) -> Result<Option<String>, BridgeError> {
    let Some(folder) = crate::portal::pick_folder("Add a music folder").await else {
        return Ok(None);
    };
    app().library.add_folder(&folder).map_err(failed)?;
    Ok(Some(folder.to_string_lossy().into_owned()))
}

/// Plays audio files handed to the app, adding them to the library on the way.
fn open_files(request: Paths) -> Result<(), BridgeError> {
    let songs: Vec<String> = request
        .paths
        .iter()
        .map(PathBuf::from)
        .filter(|path| attacca_library::is_audio(path))
        .filter_map(|path| app().library.include(&path).ok())
        .collect();
    if !songs.is_empty() {
        app().player.lock().play(app(), songs, 0, Context::Queue);
    }
    Ok(())
}

pub fn register(window: SabineWindow) -> SabineWindow {
    window
        .bridge_typed("library.songs", |_: Nothing| {
            app().library.songs().map_err(failed)
        })
        .bridge_typed(
            "library.song",
            |request: SongId| -> Result<Option<Song>, BridgeError> {
                Ok(app().library.song(&request.id))
            },
        )
        .bridge_typed(
            "library.detail",
            |request: SongId| -> Result<Option<SongDetail>, BridgeError> {
                Ok(app().library.detail(&request.id))
            },
        )
        .bridge_handler("library.waveform", waveform)
        .bridge_typed(
            "library.folders",
            |_: Nothing| -> Result<Vec<String>, BridgeError> { Ok(app().library.folders()) },
        )
        .bridge_typed_async("library.addFolder", add_folder)
        .bridge_typed("library.removeFolder", |request: FolderPath| {
            app()
                .library
                .remove_folder(&PathBuf::from(request.path))
                .map_err(failed)
        })
        .bridge_typed("library.openFiles", open_files)
        .bridge_typed(
            "library.coversFolder",
            |_: Nothing| -> Result<String, BridgeError> {
                Ok(app().library.covers_dir().to_string_lossy().into_owned())
            },
        )
        .bridge_typed(
            "albums.settings",
            |request: AlbumKey| -> Result<AlbumSettings, BridgeError> {
                Ok(app().library.album_settings(&request.key))
            },
        )
        .bridge_typed("albums.setSettings", |request: SetAlbum| {
            app()
                .library
                .set_album_settings(&request.key, &request.settings)
                .map_err(failed)?;
            app().player.lock().resound(app());
            Ok(())
        })
        .bridge_typed("edits.set", |request: SetEdits| {
            app()
                .library
                .set_edits(&request.song, &request.edits)
                .map_err(failed)
        })
        .bridge_typed("edits.reset", |request: SongId| {
            app()
                .library
                .set_edits(&request.id, &Edits::default())
                .map_err(failed)
        })
}
