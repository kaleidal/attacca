import { commands, events, type Context, type PlayerState, type Position } from "../bindings";
import { library } from "./library.svelte";

class PlayerStore {
  state = $state<PlayerState>({ playing: false, index: null, queue: [], volume: 0.85, upcoming: null, start: 0, end: null });
  position = $state<Position & { at: number }>({ token: 0, seconds: 0, playing: false, crossing: null, at: 0 });
  current = $derived(this.state.index === null ? null : (this.state.queue[this.state.index] ?? null));
  song = $derived(this.current ? (library.byId.get(this.current.song) ?? null) : null);
  following = $derived(this.state.index === null ? null : (this.state.queue[this.state.index + 1] ?? null));
  nextSong = $derived(this.following ? (library.byId.get(this.following.song) ?? null) : null);
  upNext = $derived(this.state.index === null ? this.state.queue : this.state.queue.slice(this.state.index + 1));

  async start() {
    events.playerState((state) => (this.state = state));
    events.playerPosition((position) => (this.position = { ...position, at: performance.now() }));
    this.state = await commands.playerState();
  }

  /** The play position now, moving smoothly between the engine's updates. */
  seconds(now = performance.now()): number {
    const { seconds, playing, at, token } = this.position;
    if (!playing || token !== this.current?.token) return seconds;
    return seconds + Math.min(0.25, (now - at) / 1000);
  }

  play(songs: string[], index: number, context: Context) {
    return commands.playerPlay({ songs, index, context });
  }

  toggle() {
    return commands.playerToggle();
  }

  next() {
    return commands.playerNext();
  }

  previous() {
    return commands.playerPrevious();
  }

  seek(seconds: number) {
    this.position = { ...this.position, seconds, at: performance.now() };
    return commands.playerSeek({ seconds });
  }

  volume(volume: number) {
    this.state.volume = volume;
    return commands.soundSetVolume({ volume });
  }
}

export const player = new PlayerStore();
