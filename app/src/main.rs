#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod bridge;
mod events;
mod mpris;
mod player;
mod portal;
mod settings;
mod state;

use std::time::Duration;

use sabine::SingleInstancePolicy;
use sabine::prelude::*;

const POSITION_EVERY: Duration = Duration::from_millis(100);

fn declare_events(window: SabineWindow) -> SabineWindow {
    window
        .bridge_event(&events::LIBRARY)
        .bridge_event(&events::ANALYSED)
        .bridge_event(&events::SCANNING)
        .bridge_event(&events::PLAYLISTS)
        .bridge_event(&events::PLAYER)
        .bridge_event(&events::POSITION)
        .bridge_event(&events::VISUAL)
        .bridge_event(&events::DEVICES)
        .bridge_event(&events::FAILED)
}

/// Sends the play position to the page ten times a second.
fn follow_position() {
    std::thread::Builder::new()
        .name("attacca-position".into())
        .spawn(|| {
            let app = state::app();
            loop {
                std::thread::sleep(POSITION_EVERY);
                let snapshot = app.engine.snapshot();
                app.emit(
                    &events::POSITION,
                    &events::Position {
                        token: snapshot.token as u32,
                        seconds: snapshot.seconds,
                        playing: snapshot.playing,
                        crossing: snapshot.crossing,
                    },
                );
            }
        })
        .expect("the position thread starts");
}

fn main() {
    SabineWindow::main_with_process(
        |window| {
            let window = window
                .app()
                .title("Attacca")
                .app_chrome(AppChrome::new(48, 0))
                .size(1240, 800)
                .min_size(960, 620)
                .remember_size()
                .background_color(SabineColor::rgb8(0x0e, 0x0d, 0x0c))
                .single_instance(SingleInstancePolicy::FocusExisting);
            Ok(bridge::register(declare_events(window)))
        },
        |process| {
            if let Err(error) = state::start() {
                eprintln!("attacca: {error}");
                return;
            }
            state::app().attach(process.bridge_event_emitter());
            mpris::start();
            follow_position();
        },
    );
}
