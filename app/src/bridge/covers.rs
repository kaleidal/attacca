use attacca_library::{CoverTarget, Crop};
use sabine::prelude::*;
use serde::Deserialize;

use crate::state::app;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
enum Target {
    Song { id: String },
    Album { key: String },
    Playlist { id: i64 },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetCover {
    target: Target,
    crop: Crop,
    into_files: bool,
}

/// Sets a cover from the image bytes the page sends, cropped to a square.
fn set_cover(command: BridgeCommand) -> BridgeResult {
    let request: SetCover = serde_json::from_value(command.params)
        .map_err(|error| BridgeError::new(format!("invalid cover request: {error}")))?;
    let image = command
        .body
        .ok_or_else(|| BridgeError::new("The call carries no image"))?;
    let target = match request.target {
        Target::Song { id } => CoverTarget::Song(id),
        Target::Album { key } => CoverTarget::Album(key),
        Target::Playlist { id } => CoverTarget::Playlist(id),
    };
    let name = app()
        .library
        .set_cover(&target, &image, request.crop, request.into_files)
        .map_err(|error| BridgeError::new(error.to_string()))?;
    Ok(BridgeResponse::json(serde_json::Value::String(name)))
}

pub fn register(window: SabineWindow) -> SabineWindow {
    window.bridge_handler("covers.set", set_cover)
}
