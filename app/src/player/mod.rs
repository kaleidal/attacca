pub mod plans;
pub mod seam;

use attacca_engine::{Audition, EngineEvent, TrackPlan};
use attacca_library::TransitionName;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::settings;
use crate::state::App;

const RESTART_AFTER: f64 = 3.0;
const PREVIEW_LEAD: f64 = 8.0;
const PREVIEW_TAIL: f64 = 4.0;

/// Where a queued song came from, which decides how it meets its neighbours.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Context {
    Songs,
    Album { key: String },
    Playlist { id: i64 },
    Queue,
}

/// What plays once the queue or a song runs out.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Repeat {
    #[default]
    Off,
    All,
    One,
}

#[derive(Clone, Debug)]
pub struct Item {
    pub token: u64,
    pub song: String,
    pub context: Context,
}

#[derive(Clone, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct QueueEntry {
    pub token: u32,
    pub song: String,
    pub context: Context,
}

/// The transition into the next song, placed on the playing song's timeline.
#[derive(Clone, Copy, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Upcoming {
    pub kind: TransitionName,
    pub cue: f64,
    pub overlap: f64,
    pub start: f64,
}

#[derive(Clone, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PlayerState {
    pub playing: bool,
    pub index: Option<u32>,
    /// The queue entry that plays after this one, wrapping around when it repeats.
    pub following: Option<u32>,
    pub queue: Vec<QueueEntry>,
    pub repeat: Repeat,
    pub volume: f32,
    pub upcoming: Option<Upcoming>,
    pub start: f64,
    pub end: Option<f64>,
}

#[derive(Default)]
pub struct Player {
    items: Vec<Item>,
    index: Option<usize>,
    playing: bool,
    tokens: u64,
    current: Option<TrackPlan>,
    upcoming: Option<Upcoming>,
    repeat: Repeat,
    /// The token the playing song comes back under when it repeats.
    again: Option<u64>,
}

impl Player {
    pub fn new(repeat: Repeat) -> Self {
        Self {
            repeat,
            ..Self::default()
        }
    }

    fn token(&mut self) -> u64 {
        self.tokens += 1;
        self.tokens
    }

    fn items(&mut self, songs: Vec<String>, context: &Context) -> Vec<Item> {
        songs
            .into_iter()
            .map(|song| Item {
                token: self.token(),
                song,
                context: context.clone(),
            })
            .collect()
    }

    pub fn state(&self, volume: f32) -> PlayerState {
        PlayerState {
            playing: self.playing,
            index: self.index.map(|index| index as u32),
            following: self
                .index
                .and_then(|index| self.following(index))
                .map(|index| index as u32),
            repeat: self.repeat,
            queue: self
                .items
                .iter()
                .map(|item| QueueEntry {
                    token: item.token as u32,
                    song: item.song.clone(),
                    context: item.context.clone(),
                })
                .collect(),
            volume,
            upcoming: self.upcoming,
            start: self.current.as_ref().map_or(0.0, |current| current.start),
            end: self.current.as_ref().and_then(|current| current.end),
        }
    }

    fn announce(&self, app: &App) {
        let volume = app.sound.lock().volume;
        app.publish(self.state(volume));
    }

    pub fn current_song(&self) -> Option<&str> {
        self.index
            .and_then(|index| self.items.get(index))
            .map(|item| item.song.as_str())
    }

    pub fn has_next(&self) -> bool {
        self.index
            .is_some_and(|index| self.skip_to(index).is_some())
    }

    pub fn repeat(&self) -> Repeat {
        self.repeat
    }

    pub fn set_repeat(&mut self, app: &App, repeat: Repeat) {
        self.repeat = repeat;
        if let Err(error) = app.library.set_setting("repeat", &repeat) {
            app.fail(error.to_string());
        }
        self.recue(app);
    }

    /// The song that plays by itself after the one at `index`.
    fn following(&self, index: usize) -> Option<usize> {
        match self.repeat {
            Repeat::One => Some(index),
            Repeat::All => Some((index + 1) % self.items.len()),
            Repeat::Off => (index + 1 < self.items.len()).then_some(index + 1),
        }
    }

    /// The song Next goes to: the following one, back to the start when the queue repeats.
    fn skip_to(&self, index: usize) -> Option<usize> {
        if index + 1 < self.items.len() {
            Some(index + 1)
        } else {
            (self.repeat != Repeat::Off).then_some(0)
        }
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    pub fn play(&mut self, app: &App, songs: Vec<String>, index: usize, context: Context) {
        self.items = self.items(songs, &context);
        self.start(app, index, true);
    }

    fn start(&mut self, app: &App, index: usize, skip: bool) {
        let Some(item) = self.items.get(index).cloned() else {
            return;
        };
        app.library.hurry(&item.song);
        let sound = app.sound.lock().clone();
        let Some(plan) = plans::track(&app.library, &sound, &item) else {
            app.fail(format!(
                "{} can't be found",
                app.library.song(&item.song).map_or_else(
                    || "That song".into(),
                    |song| song.display_title().to_string()
                )
            ));
            return;
        };
        app.engine.play(plan.clone(), skip);
        self.index = Some(index);
        self.current = Some(plan);
        self.playing = true;
        self.recue(app);
    }

    /// Hands the engine the song after the playing one, and how to reach it. A song that repeats
    /// comes back under a new token, so the engine tells its two plays apart.
    pub fn recue(&mut self, app: &App) {
        self.upcoming = None;
        self.again = None;
        let Some((index, following)) = self
            .index
            .filter(|_| self.current.is_some())
            .and_then(|index| Some((index, self.following(index)?)))
        else {
            app.engine.cue(None);
            self.announce(app);
            return;
        };
        let mut next = self.items[following].clone();
        if following == index {
            next.token = self.token();
            self.again = Some(next.token);
        }
        app.library.hurry(&next.song);
        let sound = app.sound.lock().clone();
        let spec = plans::transition(&app.library, &self.items[index], &next);
        match plans::track(&app.library, &sound, &next) {
            Some(plan) => {
                let current = self.current.as_ref().expect("a song is playing");
                let placed = attacca_engine::place(current, &plan, &spec);
                self.upcoming = Some(Upcoming {
                    kind: settings::name(placed.kind),
                    cue: placed.cue,
                    overlap: placed.overlap,
                    start: placed.start,
                });
                app.engine.cue(Some((plan, spec)));
            }
            None => app.engine.cue(None),
        }
        self.announce(app);
    }

    pub fn next(&mut self, app: &App) {
        if let Some(index) = self.index.and_then(|index| self.skip_to(index)) {
            self.start(app, index, true);
        }
    }

    pub fn previous(&mut self, app: &App) {
        let Some(index) = self.index else { return };
        let position = app.engine.snapshot().seconds;
        let start = self.current.as_ref().map_or(0.0, |current| current.start);
        if position - start > RESTART_AFTER {
            self.seek(app, start);
        } else if index > 0 {
            self.start(app, index - 1, true);
        } else if self.repeat == Repeat::All {
            self.start(app, self.items.len() - 1, true);
        } else {
            self.seek(app, start);
        }
    }

    pub fn toggle(&mut self, app: &App) {
        if self.playing {
            self.pause(app);
        } else {
            self.resume(app);
        }
    }

    pub fn pause(&mut self, app: &App) {
        app.engine.pause();
        self.playing = false;
        self.announce(app);
    }

    pub fn resume(&mut self, app: &App) {
        match self.index {
            Some(index) if app.engine.snapshot().token == 0 => self.start(app, index, false),
            Some(_) => {
                app.engine.resume();
                self.playing = true;
                self.announce(app);
            }
            None if !self.items.is_empty() => self.start(app, 0, false),
            None => {}
        }
    }

    pub fn seek(&mut self, app: &App, seconds: f64) {
        if self.current.is_some() {
            app.engine.seek(seconds.max(0.0));
        }
    }

    pub fn jump(&mut self, app: &App, token: u64) {
        if let Some(index) = self.items.iter().position(|item| item.token == token) {
            self.start(app, index, true);
        }
    }

    pub fn play_next(&mut self, app: &App, songs: Vec<String>) {
        let items = self.items(songs, &Context::Queue);
        let at = self.index.map_or(self.items.len(), |index| index + 1);
        self.items.splice(at..at, items);
        self.recue(app);
    }

    pub fn enqueue(&mut self, app: &App, songs: Vec<String>) {
        let items = self.items(songs, &Context::Queue);
        self.items.extend(items);
        self.recue(app);
    }

    pub fn remove(&mut self, app: &App, tokens: &[u64]) {
        let current = self.index.map(|index| self.items[index].token);
        self.items
            .retain(|item| !tokens.contains(&item.token) || Some(item.token) == current);
        self.index =
            current.and_then(|token| self.items.iter().position(|item| item.token == token));
        self.recue(app);
    }

    pub fn reorder(&mut self, app: &App, token: u64, to: usize) {
        let Some(from) = self.items.iter().position(|item| item.token == token) else {
            return;
        };
        let current = self.index.map(|index| self.items[index].token);
        let item = self.items.remove(from);
        self.items.insert(to.min(self.items.len()), item);
        self.index =
            current.and_then(|token| self.items.iter().position(|item| item.token == token));
        self.recue(app);
    }

    pub fn clear(&mut self, app: &App) {
        let current = self.index.map(|index| self.items[index].clone());
        self.items = current.into_iter().collect();
        self.index = (!self.items.is_empty()).then_some(0);
        self.recue(app);
    }

    pub fn on_engine(&mut self, app: &App, event: &EngineEvent) {
        match event {
            EngineEvent::Crossing { to, .. } => {
                if let (Some(index), true) = (self.index, self.again == Some(*to)) {
                    self.items[index].token = *to;
                }
                if let Some(index) = self.items.iter().position(|item| item.token == *to) {
                    let sound = app.sound.lock().clone();
                    self.index = Some(index);
                    self.current = plans::track(&app.library, &sound, &self.items[index]);
                    self.recue(app);
                }
            }
            EngineEvent::Ended(token) => {
                if self
                    .current
                    .as_ref()
                    .is_some_and(|current| current.token == *token)
                    || !self.items.iter().any(|item| item.token == *token)
                {
                    self.playing = false;
                    self.announce(app);
                }
            }
            EngineEvent::Failed { token, message } => {
                app.fail(message.clone());
                if self
                    .current
                    .as_ref()
                    .is_some_and(|current| current.token == *token)
                {
                    self.next(app);
                }
            }
            EngineEvent::Began(_) | EngineEvent::Advanced(_) | EngineEvent::Devices(_) => {}
        }
    }

    /// Picks up a song's new edits or analysis while it plays or waits to play next.
    pub fn refresh(&mut self, app: &App, song: &str) {
        let Some(index) = self.index else { return };
        let sound = app.sound.lock().clone();
        if self.items[index].song == song {
            if let Some(plan) = plans::track(&app.library, &sound, &self.items[index]) {
                app.engine.set_track_sound(plan.token, plan.sound, plan.end);
                self.current = Some(plan);
            }
            self.recue(app);
        } else if self
            .following(index)
            .is_some_and(|next| self.items[next].song == song)
        {
            self.recue(app);
        }
    }

    /// Re-levels the playing songs after the sound settings changed.
    pub fn resound(&mut self, app: &App) {
        if let Some(index) = self.index {
            let song = self.items[index].song.clone();
            self.refresh(app, &song);
        }
    }

    /// Plays just the transition from one song into another: the last seconds before it, the
    /// overlap, and a few seconds after.
    pub fn preview(&mut self, app: &App, from: &str, to: &str, context: Context) {
        let sound = app.sound.lock().clone();
        let first = Item {
            token: self.token(),
            song: from.to_string(),
            context: context.clone(),
        };
        let second = Item {
            token: self.token(),
            song: to.to_string(),
            context,
        };
        let (Some(mut outgoing), Some(mut incoming)) = (
            plans::track(&app.library, &sound, &first),
            plans::track(&app.library, &sound, &second),
        ) else {
            return;
        };
        let spec = plans::transition(&app.library, &first, &second);
        let placed = attacca_engine::place(&outgoing, &incoming, &spec);
        outgoing.start = outgoing.start.max(placed.cue - PREVIEW_LEAD);
        incoming.end = Some(placed.start + placed.overlap + PREVIEW_TAIL);
        app.engine.play(outgoing, true);
        app.engine.cue(Some((incoming, spec)));
        self.playing = false;
        self.announce(app);
    }

    /// Loops a few seconds of a song while one of its edit handles moves.
    pub fn audition(&mut self, app: &App, song: &str, from: f64, to: f64) {
        let sound = app.sound.lock().clone();
        let item = Item {
            token: 0,
            song: song.to_string(),
            context: Context::Songs,
        };
        if let Some(track) = plans::track(&app.library, &sound, &item) {
            app.engine.audition(Some(Audition {
                track,
                from: from.max(0.0),
                to,
            }));
        }
    }
}
