import { reducedMotion } from "./stores/visual.svelte";

/** A duration in milliseconds, or none when the system asks for reduced motion. */
export function motion(milliseconds: number): number {
  return reducedMotion.matches ? 0 : milliseconds;
}
