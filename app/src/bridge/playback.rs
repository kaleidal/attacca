use sabine::prelude::*;
use serde::Deserialize;
use specta::Type;

use super::{Nothing, Songs};
use crate::player::seam::{self, Trial};
use crate::player::{Context, PlayerState, Upcoming};
use crate::state::app;

#[derive(Deserialize, Type)]
struct Play {
    songs: Vec<String>,
    index: u32,
    context: Context,
}

#[derive(Deserialize, Type)]
struct Seek {
    seconds: f64,
}

#[derive(Deserialize, Type)]
struct Token {
    token: u32,
}

#[derive(Deserialize, Type)]
struct Tokens {
    tokens: Vec<u32>,
}

#[derive(Deserialize, Type)]
struct Reorder {
    token: u32,
    to: u32,
}

#[derive(Deserialize, Type)]
struct Watching {
    active: bool,
}

#[derive(Deserialize, Type)]
struct Preview {
    from: String,
    to: String,
    context: Context,
}

#[derive(Deserialize, Type)]
struct PlaceSeam {
    from: String,
    to: String,
    context: Context,
    trial: Trial,
}

#[derive(Deserialize, Type)]
struct Audition {
    song: String,
    from: f64,
    to: f64,
}

fn with_player(action: impl FnOnce(&mut crate::player::Player)) -> Result<(), BridgeError> {
    action(&mut app().player.lock());
    Ok(())
}

pub fn register(window: SabineWindow) -> SabineWindow {
    window
        .bridge_typed(
            "player.state",
            |_: Nothing| -> Result<PlayerState, BridgeError> {
                let volume = app().sound.lock().volume;
                Ok(app().player.lock().state(volume))
            },
        )
        .bridge_typed("player.play", |request: Play| {
            with_player(|player| {
                player.play(
                    app(),
                    request.songs,
                    request.index as usize,
                    request.context,
                )
            })
        })
        .bridge_typed("player.playNext", |request: Songs| {
            with_player(|player| player.play_next(app(), request.songs))
        })
        .bridge_typed("player.enqueue", |request: Songs| {
            with_player(|player| player.enqueue(app(), request.songs))
        })
        .bridge_typed("player.toggle", |_: Nothing| {
            with_player(|player| player.toggle(app()))
        })
        .bridge_typed("player.next", |_: Nothing| {
            with_player(|player| player.next(app()))
        })
        .bridge_typed("player.previous", |_: Nothing| {
            with_player(|player| player.previous(app()))
        })
        .bridge_typed("player.seek", |request: Seek| {
            with_player(|player| player.seek(app(), request.seconds))
        })
        .bridge_typed("player.jump", |request: Token| {
            with_player(|player| player.jump(app(), u64::from(request.token)))
        })
        .bridge_typed("player.remove", |request: Tokens| {
            let tokens: Vec<u64> = request.tokens.into_iter().map(u64::from).collect();
            with_player(|player| player.remove(app(), &tokens))
        })
        .bridge_typed("player.reorder", |request: Reorder| {
            with_player(|player| {
                player.reorder(app(), u64::from(request.token), request.to as usize)
            })
        })
        .bridge_typed("player.clear", |_: Nothing| {
            with_player(|player| player.clear(app()))
        })
        .bridge_typed("player.visual", |request: Watching| {
            app().engine.set_visual(request.active);
            Ok(())
        })
        .bridge_typed("player.preview", |request: Preview| {
            with_player(|player| player.preview(app(), &request.from, &request.to, request.context))
        })
        .bridge_typed(
            "player.placeSeam",
            |request: PlaceSeam| -> Result<Option<Upcoming>, BridgeError> {
                Ok(seam::place(
                    app(),
                    &request.from,
                    &request.to,
                    request.context,
                    request.trial,
                ))
            },
        )
        .bridge_typed("player.audition", |request: Audition| {
            with_player(|player| player.audition(app(), &request.song, request.from, request.to))
        })
        .bridge_typed("player.stopAudition", |_: Nothing| {
            app().engine.audition(None);
            Ok(())
        })
}
