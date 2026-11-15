use std::sync::Arc;

use attacca_engine::{Rhythm, TrackPlan, TrackSound, TransitionSpec};
use attacca_library::{Library, Transition, TransitionName};

use super::{Context, Item};
use crate::settings::{self, SoundSettings};

fn decibels(gain: f32) -> f32 {
    10f32.powf(gain / 20.0)
}

/// Albums play gapless unless their transitions are turned on.
fn is_gapless_album(library: &Library, item: &Item) -> bool {
    match &item.context {
        Context::Album { key } => !library.album_settings(key).transitions,
        _ => false,
    }
}

/// How the queue wants one song played: its edits, its level and its beats.
pub fn track(library: &Library, sound: &SoundSettings, item: &Item) -> Option<TrackPlan> {
    let path = library.path(&item.song)?;
    let song = library.song(&item.song)?;
    let edits = library.edits(&item.song);
    let listening = library.listening(&item.song);
    let gapless = is_gapless_album(library, item);
    let suggested = |value: Option<f32>| if gapless { None } else { value.map(f64::from) };
    let start = edits
        .start
        .or_else(|| {
            suggested(
                listening
                    .as_ref()
                    .map(|listening| listening.suggested_start),
            )
        })
        .unwrap_or(0.0);
    let end = edits
        .end
        .or_else(|| suggested(listening.as_ref().map(|listening| listening.suggested_end)))
        .filter(|end| *end > start && *end < song.duration - 0.05);
    let album = match &item.context {
        Context::Album { key } => Some(key.as_str()),
        _ => None,
    };
    let matched = if sound.matching {
        settings::matched_gain(library, &item.song, album)
    } else {
        0.0
    };
    let rhythm = listening
        .and_then(|listening| listening.rhythm)
        .map(|rhythm| {
            Arc::new(Rhythm {
                tempo: rhythm.tempo,
                beats: rhythm.beats,
                downbeat: rhythm.downbeat,
            })
        });
    Some(TrackPlan {
        token: item.token,
        path,
        start,
        end,
        duration: song.duration,
        sound: TrackSound {
            gain: decibels(matched + edits.volume),
            eq: edits.equalizer.as_ref().map(settings::curve),
            filters: edits.filters.as_ref().map(settings::filter_settings),
        },
        rhythm,
    })
}

/// Which transition leads from one queued song into the next: none inside a gapless album,
/// then the pair's own in a playlist, the song's Transition out, the playlist's default, and
/// finally a fade.
pub fn transition(library: &Library, from: &Item, to: &Item) -> TransitionSpec {
    let same_album = matches!((&from.context, &to.context), (Context::Album { key: a }, Context::Album { key: b }) if a == b);
    if same_album && is_gapless_album(library, from) {
        return settings::spec(&Transition {
            kind: TransitionName::None,
            seconds: 0.0,
            snap: false,
        });
    }
    let playlist = match (&from.context, &to.context) {
        (Context::Playlist { id: a }, Context::Playlist { id: b }) if a == b => {
            library.playlist(*a)
        }
        _ => None,
    };
    let pair = playlist.as_ref().and_then(|playlist| {
        playlist
            .pairs
            .iter()
            .find(|pair| pair.from == from.song && pair.to == to.song)
            .map(|pair| pair.transition)
    });
    let chosen = pair
        .or_else(|| library.edits(&from.song).transition)
        .or_else(|| playlist.and_then(|playlist| playlist.transition))
        .unwrap_or(Transition::FADE);
    settings::spec(&chosen)
}
