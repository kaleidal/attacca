<script lang="ts">
  import { cubicOut } from "svelte/easing";
  import { fade, scale } from "svelte/transition";
  import { artist, title } from "../../../lib/format";
  import { motion } from "../../../lib/motion";
  import { player } from "../../../lib/stores/player.svelte";
  import { ui } from "../../../lib/stores/ui.svelte";
  import { reducedMotion, visual } from "../../../lib/stores/visual.svelte";
  import Cover from "../../media/Cover.svelte";
  import Icon from "../../ui/Icon.svelte";
  import Segmented from "../../ui/Segmented.svelte";
  import { Levels } from "./levels";
  import { PALETTE_SIZE, accentPalette, coverPalette } from "./palette";
  import { Renderer, type Scene } from "./renderer";
  import { STYLES, type Style } from "./shaders";

  const KEY = "attacca.visualizer";
  const NAMES: Record<Style, string> = { hills: "Hills", ring: "Ring", fields: "Fields", bars: "Bars", wave: "Wave" };
  const IDLE_AFTER = 2600;
  const CALM_STEP_MS = 250;
  const BLEND_PER_SECOND = 2.4;
  const MAX_STEP_SECONDS = 0.1;

  const saved = localStorage.getItem(KEY);
  let style = $state<Style>(STYLES.includes(saved as Style) ? (saved as Style) : "hills");
  let canvas: HTMLCanvasElement;
  let coverBox = $state<HTMLDivElement>();
  let idle = $state(false);
  let resting: ReturnType<typeof setTimeout> | undefined;

  const levels = new Levels();
  const scene: Scene = {
    style: "hills",
    time: 0,
    center: [0, 0],
    cover: [0, 0],
    colors: new Float32Array(PALETTE_SIZE * 3),
    energy: levels.energy,
    bands: levels.bands,
    wave: levels.wave,
    scale: 1,
  };
  let target: Float32Array = new Float32Array(PALETTE_SIZE * 3);
  let primed = false;

  $effect(() => {
    scene.style = style;
    localStorage.setItem(KEY, style);
  });

  $effect(() => {
    const cover = player.song?.cover;
    let current = true;
    const palette = cover ? coverPalette(cover).catch(accentPalette) : Promise.resolve(accentPalette());
    void palette.then((colors) => {
      if (!current) return;
      if (!primed) scene.colors.set(colors);
      primed = true;
      target = colors;
    });
    return () => (current = false);
  });

  function measure() {
    const ratio = devicePixelRatio;
    const bounds = canvas.getBoundingClientRect();
    scene.scale = ratio;
    if (coverBox) {
      const box = coverBox.getBoundingClientRect();
      scene.center = [(box.left + box.width / 2 - bounds.left) * ratio, (box.top + box.height / 2 - bounds.top) * ratio];
      scene.cover = [(box.width / 2) * ratio, parseFloat(getComputedStyle(coverBox).borderTopLeftRadius) * ratio];
    } else {
      scene.center = [(bounds.width / 2) * ratio, (bounds.height / 2) * ratio];
      scene.cover = [0, 0];
    }
  }

  $effect(() => {
    void coverBox;
    measure();
  });

  function blend(seconds: number, calm: boolean) {
    const amount = calm ? 1 : 1 - Math.exp(-BLEND_PER_SECOND * seconds);
    scene.colors.forEach((value, index) => (scene.colors[index] = value + (target[index]! - value) * amount));
  }

  $effect(() => {
    const renderer = new Renderer(canvas);
    const stop = visual.watch();
    const sizes = new ResizeObserver(([entry]) => {
      const box = entry!.devicePixelContentBoxSize[0]!;
      renderer.resize(box.inlineSize, box.blockSize);
      measure();
    });
    sizes.observe(canvas, { box: "device-pixel-content-box" });
    let last = performance.now();
    let cancel = () => {};
    const frame = (now: number) => {
      const calm = reducedMotion.matches;
      const seconds = Math.min(MAX_STEP_SECONDS, (now - last) / 1000);
      last = now;
      const playing = player.state.playing;
      levels.update(seconds, playing, calm);
      blend(seconds, calm);
      if (playing) scene.time += seconds * (calm ? 0.04 : 0.35 + levels.energy[0]! * 0.9);
      if (primed && !document.hidden) renderer.draw(scene);
      schedule(calm);
    };
    const schedule = (calm: boolean) => {
      if (calm) {
        const timer = setTimeout(() => frame(performance.now()), CALM_STEP_MS);
        cancel = () => clearTimeout(timer);
      } else {
        const request = requestAnimationFrame(frame);
        cancel = () => cancelAnimationFrame(request);
      }
    };
    schedule(reducedMotion.matches);
    return () => {
      cancel();
      sizes.disconnect();
      stop();
      renderer.destroy();
    };
  });

  /** Shows the controls on movement and hides them again once the pointer rests. */
  function wake() {
    idle = false;
    clearTimeout(resting);
    resting = setTimeout(() => (idle = true), IDLE_AFTER);
  }

  $effect(() => {
    wake();
    return () => clearTimeout(resting);
  });
</script>

<svelte:window onkeydown={(event) => event.key === "Escape" && (ui.visualizer = false)} />

<div
  transition:fade={{ duration: motion(220), easing: cubicOut }}
  role="presentation"
  class="fixed inset-0 z-40 bg-ink {idle ? 'cursor-none' : ''}"
  onpointermove={wake}
  onpointerdown={wake}
>
  <canvas bind:this={canvas} class="pointer-events-none absolute inset-0 size-full" aria-hidden="true"></canvas>
  {#if style === "ring" && player.song}
    <div class="absolute inset-0 flex items-center justify-center pb-10">
      <div bind:this={coverBox} in:scale={{ start: 0.92, duration: motion(400), easing: cubicOut }} class="aspect-square h-[min(36vh,360px)] overflow-hidden rounded-[22px]">
        <Cover cover={player.song.cover} label={title(player.song)} size="fill" large class="rounded-[22px]!" />
      </div>
    </div>
  {/if}
  <div class="drag absolute inset-x-0 top-0 h-12"></div>
  <div class="absolute inset-x-10 bottom-9 flex items-end justify-between gap-6 transition-opacity duration-500 {idle ? 'opacity-0' : 'opacity-100'}">
    {#if player.song}
      <div class="min-w-0">
        <div class="display truncate text-[28px] font-semibold">{title(player.song)}</div>
        <div class="truncate text-[15px] text-paper-2">{artist(player.song)}</div>
      </div>
    {/if}
    <div class="flex shrink-0 items-center gap-2">
      <Segmented bind:value={style} label="Visualizer style" options={STYLES.map((value) => ({ value, label: NAMES[value] }))} />
      <button
        type="button"
        aria-label="Close visualizer"
        title="Close"
        class="flex size-[38px] items-center justify-center rounded-lg bg-ink-3 text-paper-2 transition duration-150 hover:bg-ink-4 hover:text-paper active:scale-95"
        onclick={() => (ui.visualizer = false)}
      >
        <Icon name="close" size={16} />
      </button>
    </div>
  </div>
</div>
