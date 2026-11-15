import type { Transition, TransitionName } from "./bindings";

export interface TransitionInfo {
  name: TransitionName;
  label: string;
  hint: string;
  seconds: number;
}

export const transitions: TransitionInfo[] = [
  { name: "fade", label: "Fade", hint: "One song gives way to the next at an even level", seconds: 6 },
  { name: "rise", label: "Rise", hint: "The next song swells up from underneath", seconds: 8 },
  { name: "blend", label: "Blend", hint: "Both play as one, the bass handed over half-way", seconds: 16 },
  { name: "wave", label: "Wave", hint: "The songs trade places in slow swells", seconds: 10 },
  { name: "melt", label: "Melt", hint: "The song slows and sinks away like a tape stopping", seconds: 5 },
  { name: "slam", label: "Slam", hint: "A hard cut on the beat with an echo tail", seconds: 0.8 },
  { name: "none", label: "None", hint: "The next song starts when this one ends", seconds: 0 },
];

export function info(name: TransitionName): TransitionInfo {
  return transitions.find((transition) => transition.name === name) ?? transitions[0]!;
}

export function fresh(name: TransitionName): Transition {
  return { kind: name, seconds: info(name).seconds, snap: true };
}

export const fade: Transition = fresh("fade");

/** Two curves for each transition, drawn left to right in a 40 by 16 box: outgoing, incoming. */
export const glyphs: Record<TransitionName, [string, string]> = {
  fade: ["M2 3 C14 3 26 13 38 13", "M2 13 C14 13 26 3 38 3"],
  rise: ["M2 3 L30 3 C32 3 33 13 35 13 L38 13", "M2 13 C12 13 22 12 30 3 L38 3"],
  blend: ["M2 3 L19 3 L21 13 L38 13", "M2 13 L19 13 L21 3 L38 3"],
  wave: ["M2 3 C8 3 10 9 14 9 S20 5 24 8 S32 13 38 13", "M2 13 C8 13 10 7 14 7 S20 11 24 8 S32 3 38 3"],
  melt: ["M2 3 C12 3 18 5 24 9 C28 12 32 13 38 13", "M2 13 C10 13 14 4 22 3 L38 3"],
  slam: ["M2 3 L20 3 L20 13", "M20 3 L38 3 M23 9 L25 9 M28 11 L29 11"],
  none: ["M2 8 L19 8", "M21 8 L38 8"],
};
