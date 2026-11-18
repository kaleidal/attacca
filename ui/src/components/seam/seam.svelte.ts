import { commands, type Context, type Edits, type Playlist, type SongDetail, type Transition, type Trial, type Upcoming } from "../../lib/bindings";
import { fade } from "../../lib/transitions";

export interface Ends {
  from: string;
  to: string;
  context: Context;
}

/** The transition between two songs as it plays, tried out live while it's dragged and saved on release. */
export class SeamTrial {
  outgoing = $state<SongDetail | null>(null);
  incoming = $state<SongDetail | null>(null);
  playlist = $state<Playlist | null>(null);
  placed = $state<Upcoming | null>(null);
  trial = $state<Trial>({});
  #ends: () => Ends;
  #asking = false;
  #again = false;

  constructor(ends: () => Ends) {
    this.#ends = ends;
  }

  get inPlaylist() {
    return this.#ends().context.type === "playlist" && this.playlist !== null;
  }

  /** The transition saved for these two songs, or null when they follow one set elsewhere. */
  get own(): Transition | null {
    const { from, to } = this.#ends();
    if (this.inPlaylist) return this.playlist!.pairs.find((pair) => pair.from === from && pair.to === to)?.transition ?? null;
    return this.outgoing?.edits.transition ?? null;
  }

  get fallback(): { label: string; transition: Transition } {
    const songs = this.outgoing?.edits.transition ?? null;
    if (this.inPlaylist && songs) return { label: "The song's own", transition: songs };
    return { label: "Default", transition: this.playlist?.transition ?? fade };
  }

  get transition(): Transition {
    return this.trial.transition ?? this.own ?? this.fallback.transition;
  }

  async load() {
    const { from, to, context } = this.#ends();
    const [outgoing, incoming, playlist] = await Promise.all([
      commands.libraryDetail({ id: from }),
      commands.libraryDetail({ id: to }),
      context.type === "playlist" ? commands.playlistsGet({ id: context.id }) : Promise.resolve(null),
    ]);
    this.outgoing = outgoing;
    this.incoming = incoming;
    this.playlist = playlist;
    await this.place();
  }

  /** Asks where the transition lands, at most one question at a time, always ending on the latest trial. */
  async place() {
    if (this.#asking) {
      this.#again = true;
      return;
    }
    this.#asking = true;
    const { from, to, context } = this.#ends();
    this.placed = await commands.playerPlaceSeam({ from, to, context, trial: $state.snapshot(this.trial) });
    this.#asking = false;
    if (this.#again) {
      this.#again = false;
      await this.place();
    }
  }

  try(change: Trial) {
    this.trial = { ...this.trial, ...change };
    void this.place();
  }

  async save() {
    const { from, to, context } = this.#ends();
    const { end, start, transition } = $state.snapshot(this.trial);
    const saves: Promise<unknown>[] = [];
    if (end !== undefined && this.outgoing) saves.push(this.#edit(this.outgoing, { end }));
    if (start !== undefined && this.incoming) saves.push(this.#edit(this.incoming, { start }));
    if (transition !== undefined) {
      if (this.inPlaylist && context.type === "playlist") {
        saves.push(commands.playlistsSetPair({ id: context.id, from, to, transition }).then(async () => (this.playlist = await commands.playlistsGet({ id: context.id }))));
      } else if (this.outgoing) {
        saves.push(this.#edit(this.outgoing, { transition }));
      }
    }
    await Promise.all(saves);
    this.trial = {};
    await this.place();
  }

  choose(transition: Transition | null) {
    this.trial = { ...this.trial, transition };
    void this.save();
  }

  #edit(detail: SongDetail, change: Partial<Edits>) {
    detail.edits = { ...detail.edits, ...change };
    return commands.editsSet({ song: detail.song.id, edits: $state.snapshot(detail.edits) });
  }
}
