use std::sync::OnceLock;

use attacca_engine::{Device, Engine, EngineEvent};
use attacca_library::{Library, LibraryEvent};
use parking_lot::Mutex;
use sabine::BridgeEventEmitter;
use sabine::prelude::*;

use crate::events::{self, DeviceInfo, Failure, LibraryChange, Scanning};
use crate::player::{Player, PlayerState};
use crate::settings::{self, SoundSettings};

pub const APP_ID: &str = "al.kaleid.attacca";

pub struct App {
    pub library: Library,
    pub engine: Engine,
    pub player: Mutex<Player>,
    pub sound: Mutex<SoundSettings>,
    pub devices: Mutex<Vec<Device>>,
    emitter: OnceLock<BridgeEventEmitter>,
}

static APP: OnceLock<App> = OnceLock::new();

pub fn app() -> &'static App {
    APP.wait()
}

/// Opens the library and starts the engine. Their events reach the app once it exists.
pub fn start() -> Result<(), String> {
    let folder = AppEnvironment::current().app_data_dir(APP_ID);
    let library = Library::open(&folder, |event| {
        if let Some(app) = APP.get() {
            app.on_library(event);
        }
    })
    .map_err(|error| error.to_string())?;
    let sound = settings::load(&library);
    let engine = Engine::start(
        sound.output.clone(),
        |event| {
            if let Some(app) = APP.get() {
                app.on_engine(event);
            }
        },
        |frame| {
            if let Some(app) = APP.get() {
                app.send_bytes(&events::VISUAL, frame);
            }
        },
    );
    engine.set_global(settings::global_sound(&sound));
    engine.set_volume(sound.volume);
    let app = App {
        library,
        engine,
        player: Mutex::new(Player::default()),
        sound: Mutex::new(sound),
        devices: Mutex::new(Vec::new()),
        emitter: OnceLock::new(),
    };
    let _ = APP.set(app);
    crate::state::app().library.start();
    Ok(())
}

impl App {
    pub fn attach(&self, emitter: BridgeEventEmitter) {
        let _ = self.emitter.set(emitter);
    }

    pub fn emit<T: serde::Serialize>(&self, event: &sabine::BridgeEvent<T>, payload: &T) {
        if let Some(emitter) = self.emitter.get() {
            emitter.send(event, payload);
        }
    }

    fn send_bytes(&self, event: &sabine::BridgeEvent<sabine::Bytes>, bytes: &[u8]) {
        if let Some(emitter) = self.emitter.get() {
            emitter.send_bytes(event, bytes);
        }
    }

    pub fn fail(&self, message: String) {
        self.emit(&events::FAILED, &Failure { message });
    }

    /// Sends the player's state to the page and the system's media controls.
    pub fn publish(&self, state: PlayerState) {
        crate::mpris::update(&state);
        self.emit(&events::PLAYER, &state);
    }

    fn on_engine(&self, event: EngineEvent) {
        match &event {
            EngineEvent::Devices(devices) => {
                *self.devices.lock() = devices.clone();
                let list = devices.iter().map(DeviceInfo::from).collect();
                self.emit(&events::DEVICES, &list);
            }
            _ => self.player.lock().on_engine(self, &event),
        }
    }

    fn on_library(&self, event: LibraryEvent) {
        match event {
            LibraryEvent::Songs { changed, removed } => {
                for song in &changed {
                    self.player.lock().refresh(self, song);
                }
                self.emit(&events::LIBRARY, &LibraryChange { changed, removed });
            }
            LibraryEvent::Analysed(song) => {
                self.player.lock().refresh(self, &song);
                self.emit(&events::ANALYSED, &song);
            }
            LibraryEvent::Scanning(busy) => self.emit(&events::SCANNING, &Scanning { busy }),
            LibraryEvent::Playlists => self.emit(&events::PLAYLISTS, &()),
        }
    }
}
