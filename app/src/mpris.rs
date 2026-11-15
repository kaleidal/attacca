use std::sync::OnceLock;

use mpris_server::zbus::{Result, fdo};
use mpris_server::{
    LoopStatus, Metadata, PlaybackRate, PlaybackStatus, PlayerInterface, Property, RootInterface,
    Server, Time, TrackId, Volume,
};
use parking_lot::Mutex;

use crate::player::PlayerState;
use crate::state::app;

/// The system's media controls: media keys, the desktop's now-playing widgets.
pub struct Controls;

static SERVER: OnceLock<Server<Controls>> = OnceLock::new();
static LAST: Mutex<Option<(bool, Option<String>, bool)>> = Mutex::new(None);

pub fn start() {
    sabine::async_runtime().spawn(async {
        match Server::new("attacca", Controls).await {
            Ok(server) => {
                let _ = SERVER.set(server);
            }
            Err(error) => eprintln!("attacca: media controls are unavailable: {error}"),
        }
    });
}

fn track_id(token: u32) -> TrackId {
    TrackId::try_from(format!("/al/kaleid/attacca/track/{token}")).unwrap_or(TrackId::NO_TRACK)
}

fn metadata() -> Metadata {
    let player = app().player.lock();
    let state = player.state(0.0);
    drop(player);
    let Some(entry) = state
        .index
        .and_then(|index| state.queue.get(index as usize))
    else {
        return Metadata::new();
    };
    let Some(song) = app().library.song(&entry.song) else {
        return Metadata::new();
    };
    let mut metadata = Metadata::builder()
        .trackid(track_id(entry.token))
        .title(song.display_title())
        .length(Time::from_micros((song.duration * 1_000_000.0) as i64))
        .build();
    metadata.set_artist(song.artist.clone().map(|artist| [artist]));
    metadata.set_album(song.album.clone());
    metadata.set_album_artist(song.album_artist.clone().map(|artist| [artist]));
    metadata.set_track_number(song.track.map(|track| track as i32));
    if let Some(cover) = &song.cover {
        let path = app().library.covers_dir().join(format!("{cover}.jpg"));
        metadata.set_art_url(Some(format!("file://{}", path.display())));
    }
    metadata
}

/// Tells the system's media controls what changed.
pub fn update(state: &PlayerState) {
    let Some(server) = SERVER.get() else { return };
    let song = state
        .index
        .and_then(|index| state.queue.get(index as usize))
        .map(|entry| entry.song.clone());
    let has_next = state
        .index
        .is_some_and(|index| (index as usize) + 1 < state.queue.len());
    let key = (state.playing, song, has_next);
    let mut last = LAST.lock();
    if last.as_ref() == Some(&key) {
        return;
    }
    let song_changed = last.as_ref().is_none_or(|last| last.1 != key.1);
    *last = Some(key);
    drop(last);
    let status = if state.playing {
        PlaybackStatus::Playing
    } else {
        PlaybackStatus::Paused
    };
    sabine::async_runtime().spawn(async move {
        let mut properties = vec![
            Property::PlaybackStatus(status),
            Property::CanGoNext(has_next),
        ];
        if song_changed {
            properties.push(Property::Metadata(metadata()));
        }
        let _ = server.properties_changed(properties).await;
    });
}

impl RootInterface for Controls {
    async fn raise(&self) -> fdo::Result<()> {
        Ok(())
    }

    async fn quit(&self) -> fdo::Result<()> {
        Ok(())
    }

    async fn can_quit(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn fullscreen(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn set_fullscreen(&self, _fullscreen: bool) -> Result<()> {
        Ok(())
    }

    async fn can_set_fullscreen(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn can_raise(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn has_track_list(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn identity(&self) -> fdo::Result<String> {
        Ok("Attacca".into())
    }

    async fn desktop_entry(&self) -> fdo::Result<String> {
        Ok(crate::state::APP_ID.into())
    }

    async fn supported_uri_schemes(&self) -> fdo::Result<Vec<String>> {
        Ok(vec!["file".into()])
    }

    async fn supported_mime_types(&self) -> fdo::Result<Vec<String>> {
        Ok([
            "audio/flac",
            "audio/mpeg",
            "audio/mp4",
            "audio/ogg",
            "audio/opus",
            "audio/wav",
            "audio/aiff",
        ]
        .map(String::from)
        .to_vec())
    }
}

impl PlayerInterface for Controls {
    async fn next(&self) -> fdo::Result<()> {
        app().player.lock().next(app());
        Ok(())
    }

    async fn previous(&self) -> fdo::Result<()> {
        app().player.lock().previous(app());
        Ok(())
    }

    async fn pause(&self) -> fdo::Result<()> {
        app().player.lock().pause(app());
        Ok(())
    }

    async fn play_pause(&self) -> fdo::Result<()> {
        app().player.lock().toggle(app());
        Ok(())
    }

    async fn stop(&self) -> fdo::Result<()> {
        app().player.lock().pause(app());
        Ok(())
    }

    async fn play(&self) -> fdo::Result<()> {
        app().player.lock().resume(app());
        Ok(())
    }

    async fn seek(&self, offset: Time) -> fdo::Result<()> {
        let position = app().engine.snapshot().seconds;
        app()
            .player
            .lock()
            .seek(app(), position + offset.as_micros() as f64 / 1_000_000.0);
        Ok(())
    }

    async fn set_position(&self, _track_id: TrackId, position: Time) -> fdo::Result<()> {
        app()
            .player
            .lock()
            .seek(app(), position.as_micros() as f64 / 1_000_000.0);
        Ok(())
    }

    async fn open_uri(&self, _uri: String) -> fdo::Result<()> {
        Ok(())
    }

    async fn playback_status(&self) -> fdo::Result<PlaybackStatus> {
        Ok(if app().player.lock().is_playing() {
            PlaybackStatus::Playing
        } else {
            PlaybackStatus::Paused
        })
    }

    async fn loop_status(&self) -> fdo::Result<LoopStatus> {
        Ok(LoopStatus::None)
    }

    async fn set_loop_status(&self, _loop_status: LoopStatus) -> Result<()> {
        Ok(())
    }

    async fn rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(1.0)
    }

    async fn set_rate(&self, _rate: PlaybackRate) -> Result<()> {
        Ok(())
    }

    async fn shuffle(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn set_shuffle(&self, _shuffle: bool) -> Result<()> {
        Ok(())
    }

    async fn metadata(&self) -> fdo::Result<Metadata> {
        Ok(metadata())
    }

    async fn volume(&self) -> fdo::Result<Volume> {
        Ok(f64::from(app().sound.lock().volume))
    }

    async fn set_volume(&self, volume: Volume) -> Result<()> {
        crate::bridge::sound::set_volume(volume as f32);
        Ok(())
    }

    async fn position(&self) -> fdo::Result<Time> {
        Ok(Time::from_micros(
            (app().engine.snapshot().seconds * 1_000_000.0) as i64,
        ))
    }

    async fn minimum_rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(1.0)
    }

    async fn maximum_rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(1.0)
    }

    async fn can_go_next(&self) -> fdo::Result<bool> {
        Ok(app().player.lock().has_next())
    }

    async fn can_go_previous(&self) -> fdo::Result<bool> {
        Ok(app().player.lock().current_song().is_some())
    }

    async fn can_play(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn can_pause(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn can_seek(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn can_control(&self) -> fdo::Result<bool> {
        Ok(true)
    }
}
