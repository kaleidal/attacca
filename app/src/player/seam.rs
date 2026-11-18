use attacca_library::Transition;
use serde::Deserialize;
use specta::Type;

use super::{Context, Item, Upcoming, plans};
use crate::settings;
use crate::state::App;

/// What the person is trying out between two songs before it's saved: where the first ends,
/// where the second starts and the transition between them.
#[derive(Clone, Debug, Default, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Trial {
    pub end: Option<f64>,
    pub start: Option<f64>,
    pub transition: Option<Transition>,
}

/// Where the transition from one song into another lands, as it would play, with `trial` in
/// place of what's saved.
pub fn place(app: &App, from: &str, to: &str, context: Context, trial: Trial) -> Option<Upcoming> {
    let sound = app.sound.lock().clone();
    let item = |song: &str| Item {
        token: 0,
        song: song.to_string(),
        context: context.clone(),
    };
    let (first, second) = (item(from), item(to));
    let mut outgoing = plans::track(&app.library, &sound, &first)?;
    let mut incoming = plans::track(&app.library, &sound, &second)?;
    if let Some(end) = trial.end {
        outgoing.end = Some(end.clamp(outgoing.start, outgoing.duration))
            .filter(|end| *end < outgoing.duration - 0.05);
    }
    if let Some(start) = trial.start {
        incoming.start = start.clamp(0.0, incoming.duration);
    }
    let spec = trial.transition.as_ref().map_or_else(
        || plans::transition(&app.library, &first, &second),
        settings::spec,
    );
    let placed = attacca_engine::place(&outgoing, &incoming, &spec);
    Some(Upcoming {
        kind: settings::name(placed.kind),
        cue: placed.cue,
        overlap: placed.overlap,
        start: placed.start,
    })
}
