use std::collections::VecDeque;
use std::sync::Arc;

use parking_lot::{Condvar, Mutex};

use crate::{Inner, LibraryEvent};

/// Songs waiting to be analysed. Songs someone is about to hear go first.
#[derive(Default)]
pub struct Queue {
    songs: Mutex<VecDeque<String>>,
    ready: Condvar,
}

impl Queue {
    pub fn push(&self, songs: Vec<String>) {
        if songs.is_empty() {
            return;
        }
        let mut queue = self.songs.lock();
        for song in songs {
            if !queue.contains(&song) {
                queue.push_back(song);
            }
        }
        self.ready.notify_all();
    }

    pub fn hurry(&self, song: &str) {
        let mut queue = self.songs.lock();
        queue.retain(|queued| queued != song);
        queue.push_front(song.to_string());
        self.ready.notify_one();
    }

    fn next(&self) -> String {
        let mut queue = self.songs.lock();
        loop {
            if let Some(song) = queue.pop_front() {
                return song;
            }
            self.ready.wait(&mut queue);
        }
    }
}

pub fn workers(inner: &Arc<Inner>) {
    let count =
        std::thread::available_parallelism().map_or(2, |cores| (cores.get() / 2).clamp(1, 6));
    for _ in 0..count {
        let inner = Arc::clone(inner);
        std::thread::Builder::new()
            .name("attacca-analysis".into())
            .spawn(move || {
                loop {
                    let song = inner.queue.next();
                    analyse(&inner, &song);
                }
            })
            .expect("analysis threads start");
    }
}

fn analyse(inner: &Inner, song: &str) {
    let path = {
        let db = inner.db.lock();
        if db.analysis_version(song).ok().flatten() == Some(attacca_analysis::VERSION) {
            return;
        }
        db.paths_of(song)
            .ok()
            .and_then(|paths| paths.into_iter().next())
    };
    let Some(path) = path else {
        return;
    };
    match attacca_analysis::analyse(std::path::Path::new(&path)) {
        Ok(analysis) => {
            if inner.db.lock().put_analysis(song, &analysis).is_ok() {
                inner.emit(LibraryEvent::Analysed(song.to_string()));
                crate::share::welcome(inner, song, &analysis.fingerprint);
            }
        }
        Err(error) => eprintln!("attacca: couldn't analyse {path}: {error}"),
    }
}
