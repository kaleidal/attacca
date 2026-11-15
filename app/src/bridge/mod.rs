mod covers;
mod library;
mod playback;
mod playlists;
pub mod sound;

use sabine::SabineWindow;
use serde::Deserialize;
use specta::Type;

#[derive(Deserialize, Type)]
pub struct Nothing {}

#[derive(Deserialize, Type)]
pub struct SongId {
    pub id: String,
}

#[derive(Deserialize, Type)]
pub struct Songs {
    pub songs: Vec<String>,
}

pub fn register(window: SabineWindow) -> SabineWindow {
    let window = library::register(window);
    let window = playback::register(window);
    let window = playlists::register(window);
    let window = sound::register(window);
    covers::register(window)
}
