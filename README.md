# Attacca

A music player for files you own, built around one idea: you decide how each song ends and how the next one begins.

- Plays a local library of songs, albums, artists and playlists, with a queue.
- Sets where each song starts and ends without touching the file.
- Moves between songs with one of six transitions: fade, rise, blend, wave, melt and slam. Albums play gapless.
- Shapes the sound with a ten-band equalizer, high-pass and low-pass filters and volume matching, globally or per song.
- Shows a visualizer driven by the mixed output.
- Edits covers for songs, albums and playlists.
- Shares a playlist as a small `.attacca` file. The other person's own copies of the songs fill it in.

Nothing is uploaded and there is no account. Your edits live beside the library and can always be reset.

## Building

You need Rust, Bun, the [Sabine](https://sabine.rs) CLI and the PipeWire development headers (`pipewire-devel` on Fedora).

```sh
sabine dev
```

starts the app against the interface's development server. To build a package:

```sh
sabine bundle .
```

## How it fits together

| Folder | What it does |
| --- | --- |
| `crates/media` | Opens and decodes files, trims encoder delay and padding so albums join without gaps, and identifies songs by their audio so moving or retagging a file keeps its edits. |
| `crates/engine` | The player. Two decks decode and resample on their own threads; a real-time mixer reads them, applies each song's sound and the transition curves, and plays through PipeWire. |
| `crates/analysis` | Listens to each song once: loudness, tempo, beats and bars, silence at both ends, closing fades, the waveform and an acoustic fingerprint. |
| `crates/library` | The library in SQLite: folders it watches, songs, analysis, edits, playlists, covers and shared playlists. |
| `app` | The desktop app: bridge commands, the queue and how it picks transitions, and the system's media controls. |
| `ui` | The interface, in Svelte. |

### Transitions

A transition starts at the outgoing song's End and brings in the next song from its Start. With beats known on both sides it lands on a downbeat and lasts whole bars. When the outgoing song has too little audio left, the transition moves earlier so it never runs into silence.

The transition between two songs is, in order: the one set for that pair in a playlist, the outgoing song's own Transition out, the playlist's default, and otherwise a fade. Songs in an album join gapless unless transitions are turned on for that album, and songs of the same sample rate continue one resampled stream, so the join is sample-continuous.

| Transition | What happens |
| --- | --- |
| Fade | Equal-power crossfade. |
| Rise | The next song climbs out of a closed low-pass while the current one holds, then the current one drops at the peak. |
| Blend | A long overlap with the bass handed over at a bar half-way, so the low end never doubles. Within 6% of each other, the outgoing song is time-stretched to the incoming tempo. |
| Wave | The two songs sway against each other, each swell leaning further to the next song. |
| Melt | The outgoing song slows like a tape stopping while its low-pass closes. |
| Slam | A cut on the beat that leaves a tempo-synced echo tail. |
| None | The next song starts when this one ends. |

A limiter at the output keeps overlaps and equalizer boosts from clipping.
