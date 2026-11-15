import { commands, events } from "../bindings";

export const BANDS = 64;
export const WAVE = 256;

/** The mixed output's spectrum and waveform, flowing only while something shows them. */
class VisualFeed {
  frame: Uint8Array = new Uint8Array(BANDS + WAVE);
  #watchers = 0;
  #stop: (() => void) | null = null;
  #hidden = false;

  constructor() {
    document.addEventListener("visibilitychange", () => {
      this.#hidden = document.hidden;
      this.#apply();
    });
  }

  watch(): () => void {
    this.#watchers += 1;
    this.#apply();
    return () => {
      this.#watchers -= 1;
      this.#apply();
    };
  }

  #apply() {
    const active = this.#watchers > 0 && !this.#hidden;
    if (active && !this.#stop) {
      this.#stop = events.playerVisual((frame) => (this.frame = frame));
      void commands.playerVisual({ active: true });
    } else if (!active && this.#stop) {
      this.#stop();
      this.#stop = null;
      void commands.playerVisual({ active: false });
    }
  }

  band(index: number): number {
    return (this.frame[index] ?? 0) / 255;
  }

  wave(index: number): number {
    const byte = this.frame[BANDS + index] ?? 0;
    return (byte > 127 ? byte - 256 : byte) / 127;
  }
}

export const visual = new VisualFeed();

export const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");
