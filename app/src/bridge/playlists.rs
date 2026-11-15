use std::path::PathBuf;

use attacca_library::{ImportPreview, Playlist, PlaylistSummary, Transition};
use sabine::prelude::*;
use serde::Deserialize;
use specta::Type;

use super::Nothing;
use crate::state::app;

#[derive(Deserialize, Type)]
struct PlaylistId {
    id: i64,
}

#[derive(Deserialize, Type)]
struct Create {
    name: String,
    songs: Vec<String>,
}

#[derive(Deserialize, Type)]
struct Rename {
    id: i64,
    name: String,
}

#[derive(Deserialize, Type)]
struct Add {
    id: i64,
    songs: Vec<String>,
}

#[derive(Deserialize, Type)]
struct Order {
    id: i64,
    entries: Vec<i64>,
}

#[derive(Deserialize, Type)]
struct SetTransition {
    id: i64,
    transition: Option<Transition>,
}

#[derive(Deserialize, Type)]
struct SetPair {
    id: i64,
    from: String,
    to: String,
    transition: Option<Transition>,
}

#[derive(Deserialize, Type)]
struct SharedFile {
    path: String,
}

#[derive(Deserialize, Type)]
#[serde(rename_all = "camelCase")]
struct Import {
    path: String,
    keep_own: bool,
}

fn failed(error: impl std::fmt::Display) -> BridgeError {
    BridgeError::new(error.to_string())
}

fn changed<T>(result: Result<T, attacca_library::LibraryError>) -> Result<T, BridgeError> {
    let value = result.map_err(failed)?;
    app().player.lock().resound(app());
    Ok(value)
}

async fn share(request: PlaylistId) -> Result<Option<String>, BridgeError> {
    let name = app()
        .library
        .playlist(request.id)
        .map_or_else(|| "Playlist".to_string(), |playlist| playlist.name);
    let file = format!("{}.attacca", name.replace(['/', '\\'], "-"));
    let Some(path) =
        crate::portal::pick_save("Share playlist", &file, ("Attacca playlist", "*.attacca")).await
    else {
        return Ok(None);
    };
    app().library.export(request.id, &path).map_err(failed)?;
    Ok(Some(path.to_string_lossy().into_owned()))
}

pub fn register(window: SabineWindow) -> SabineWindow {
    window
        .bridge_typed(
            "playlists.list",
            |_: Nothing| -> Result<Vec<PlaylistSummary>, BridgeError> {
                app().library.playlists().map_err(failed)
            },
        )
        .bridge_typed(
            "playlists.get",
            |request: PlaylistId| -> Result<Option<Playlist>, BridgeError> {
                Ok(app().library.playlist(request.id))
            },
        )
        .bridge_typed("playlists.create", |request: Create| {
            app()
                .library
                .create_playlist(&request.name, &request.songs)
                .map_err(failed)
        })
        .bridge_typed("playlists.rename", |request: Rename| {
            app()
                .library
                .rename_playlist(request.id, &request.name)
                .map_err(failed)
        })
        .bridge_typed("playlists.delete", |request: PlaylistId| {
            app().library.delete_playlist(request.id).map_err(failed)
        })
        .bridge_typed("playlists.add", |request: Add| {
            changed(app().library.append_to_playlist(request.id, &request.songs))
        })
        .bridge_typed("playlists.order", |request: Order| {
            changed(app().library.order_playlist(request.id, &request.entries))
        })
        .bridge_typed("playlists.setTransition", |request: SetTransition| {
            changed(
                app()
                    .library
                    .set_playlist_transition(request.id, request.transition.as_ref()),
            )
        })
        .bridge_typed("playlists.setPair", |request: SetPair| {
            changed(app().library.set_pair(
                request.id,
                &request.from,
                &request.to,
                request.transition.as_ref(),
            ))
        })
        .bridge_typed_async("playlists.share", share)
        .bridge_typed(
            "playlists.preview",
            |request: SharedFile| -> Result<ImportPreview, BridgeError> {
                app()
                    .library
                    .preview_import(&PathBuf::from(request.path))
                    .map_err(failed)
            },
        )
        .bridge_typed("playlists.import", |request: Import| {
            app()
                .library
                .import(&PathBuf::from(request.path), request.keep_own)
                .map_err(failed)
        })
}
