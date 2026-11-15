use attacca_engine::Device;
use sabine::{BridgeEvent, Bytes};
use serde::Serialize;
use specta::Type;

use crate::player::PlayerState;

#[derive(Clone, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LibraryChange {
    pub changed: Vec<String>,
    pub removed: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Scanning {
    pub busy: bool,
}

#[derive(Clone, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub token: u32,
    pub seconds: f64,
    pub playing: bool,
    pub crossing: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Failure {
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub name: String,
    pub description: String,
}

impl From<&Device> for DeviceInfo {
    fn from(device: &Device) -> Self {
        Self {
            name: device.name.clone(),
            description: device.description.clone(),
        }
    }
}

pub const LIBRARY: BridgeEvent<LibraryChange> = BridgeEvent::new("library.changed");
pub const ANALYSED: BridgeEvent<String> = BridgeEvent::new("library.analysed");
pub const SCANNING: BridgeEvent<Scanning> = BridgeEvent::new("library.scanning");
pub const PLAYLISTS: BridgeEvent<()> = BridgeEvent::new("playlists.changed");
pub const PLAYER: BridgeEvent<PlayerState> = BridgeEvent::new("player.state");
pub const POSITION: BridgeEvent<Position> = BridgeEvent::new("player.position");
pub const VISUAL: BridgeEvent<Bytes> = BridgeEvent::new("player.visual");
pub const DEVICES: BridgeEvent<Vec<DeviceInfo>> = BridgeEvent::new("sound.devices");
pub const FAILED: BridgeEvent<Failure> = BridgeEvent::new("app.failed");
