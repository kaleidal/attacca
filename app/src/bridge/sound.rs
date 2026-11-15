use sabine::prelude::*;
use serde::{Deserialize, Serialize};
use specta::Type;

use super::Nothing;
use crate::events::DeviceInfo;
use crate::settings::{self, SoundSettings};
use crate::state::app;

#[derive(Serialize, Type)]
struct SoundState {
    settings: SoundSettings,
    devices: Vec<DeviceInfo>,
}

#[derive(Deserialize, Type)]
struct SetSound {
    settings: SoundSettings,
}

#[derive(Deserialize, Type)]
struct SetVolume {
    volume: f32,
}

fn state() -> SoundState {
    SoundState {
        settings: app().sound.lock().clone(),
        devices: app().devices.lock().iter().map(DeviceInfo::from).collect(),
    }
}

fn save(settings: &SoundSettings) -> Result<(), BridgeError> {
    app()
        .library
        .set_setting("sound", settings)
        .map_err(|error| BridgeError::new(error.to_string()))
}

fn set(request: SetSound) -> Result<(), BridgeError> {
    let next = request.settings;
    let previous = std::mem::replace(&mut *app().sound.lock(), next.clone());
    app().engine.set_global(settings::global_sound(&next));
    app().engine.set_volume(next.volume);
    if previous.output != next.output {
        app().engine.set_output(next.output.clone());
    }
    if previous.matching != next.matching {
        app().player.lock().resound(app());
    }
    save(&next)
}

pub fn set_volume(volume: f32) {
    let volume = volume.clamp(0.0, 1.0);
    let settings = {
        let mut sound = app().sound.lock();
        sound.volume = volume;
        sound.clone()
    };
    app().engine.set_volume(volume);
    let _ = save(&settings);
}

pub fn register(window: SabineWindow) -> SabineWindow {
    window
        .bridge_typed(
            "sound.get",
            |_: Nothing| -> Result<SoundState, BridgeError> { Ok(state()) },
        )
        .bridge_typed("sound.set", set)
        .bridge_typed("sound.setVolume", |request: SetVolume| {
            set_volume(request.volume);
            Ok(())
        })
}
