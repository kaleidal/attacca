import { BANDS, WAVE, visual } from "../../../lib/stores/visual.svelte";

const ATTACK_PER_SECOND = 22;
const RELEASE_PER_SECOND = 3.5;
const CALM = 0.12;
const AT_REST = 0.004;
const SPREAD = [1, 4, 6, 4, 1].map((weight) => weight / 16);
const RANGES = [
  [0, 10],
  [10, 36],
  [36, BANDS],
] as const;

/** Eases the mixed output's bands toward each new frame: quick to rise, slow to fall. */
export class Levels {
  readonly bands = new Uint8Array(BANDS);
  readonly wave = new Uint8Array(WAVE);
  readonly energy = new Float32Array(RANGES.length);
  #levels = new Float32Array(BANDS);
  #targets = new Float32Array(BANDS);
  #spread = new Float32Array(BANDS);

  update(seconds: number, live: boolean, calm: boolean, intensity: number): boolean {
    if (live) this.#hear(intensity);
    else this.#targets.fill(0);
    const pace = calm ? CALM : 1;
    const rise = 1 - Math.exp(-ATTACK_PER_SECOND * pace * seconds);
    const fall = 1 - Math.exp(-RELEASE_PER_SECOND * pace * seconds);
    let moving = false;
    for (let band = 0; band < BANDS; band++) {
      const target = this.#targets[band]!;
      const level = this.#levels[band]!;
      const next = level + (target - level) * (target > level ? rise : fall);
      this.#levels[band] = next;
      this.bands[band] = Math.round(next * 255);
      moving ||= next > AT_REST;
    }
    RANGES.forEach(([from, to], index) => {
      let sum = 0;
      for (let band = from; band < to; band++) sum += this.#levels[band]!;
      this.energy[index] = sum / (to - from);
    });
    return moving;
  }

  #hear(intensity: number) {
    for (let band = 0; band < BANDS; band++) {
      const tilt = 0.82 + (0.45 * band) / BANDS;
      this.#targets[band] = Math.min(1, visual.band(band) ** 1.35 * tilt * intensity);
    }
    for (let pass = 0; pass < 2; pass++) {
      for (let band = 0; band < BANDS; band++) {
        let sum = 0;
        SPREAD.forEach((weight, offset) => (sum += weight * this.#targets[Math.min(BANDS - 1, Math.max(0, band + offset - 2))]!));
        this.#spread[band] = sum;
      }
      this.#targets.set(this.#spread);
    }
    for (let point = 0; point < WAVE; point++) this.wave[point] = Math.round((Math.max(-1, Math.min(1, visual.wave(point) * intensity)) * 0.5 + 0.5) * 255);
  }
}
