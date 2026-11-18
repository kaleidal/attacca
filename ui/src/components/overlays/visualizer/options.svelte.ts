import { STYLES, type Style } from "./shaders";

const KEY = "attacca.visualizer.look";

export type Colors = "cover" | "accent" | "mono";
export type Info = "always" | "controls" | "hidden";

interface Look {
  style: Style;
  colors: Colors;
  info: Info;
  intensity: number;
}

const DEFAULTS: Look = { style: "hills", colors: "cover", info: "always", intensity: 1 };

/** How the visualizer looks, kept on this computer. */
class VisualizerOptions {
  style = $state<Style>(DEFAULTS.style);
  colors = $state<Colors>(DEFAULTS.colors);
  info = $state<Info>(DEFAULTS.info);
  intensity = $state(DEFAULTS.intensity);

  constructor() {
    const saved: Partial<Look> = JSON.parse(localStorage.getItem(KEY) ?? "{}");
    if (saved.style && STYLES.includes(saved.style)) this.style = saved.style;
    if (saved.colors) this.colors = saved.colors;
    if (saved.info) this.info = saved.info;
    if (typeof saved.intensity === "number") this.intensity = saved.intensity;
  }

  save() {
    const look: Look = { style: this.style, colors: this.colors, info: this.info, intensity: this.intensity };
    localStorage.setItem(KEY, JSON.stringify(look));
  }
}

export const options = new VisualizerOptions();
