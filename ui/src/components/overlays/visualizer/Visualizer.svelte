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
  import Popover from "../../ui/Popover.svelte";
  import Segmented from "../../ui/Segmented.svelte";
  import Slider from "../../ui/Slider.svelte";
  import { Levels } from "./levels";
  import { options, type Colors, type Info } from "./options.svelte";
  import { PALETTE_SIZE, accentPalette, coverPalette, monoPalette } from "./palette";
  import { Renderer, type Scene } from "./renderer";
  import { STYLES, type Style } from "./shaders";

  const NAMES: Record<Style, string> = { hills: "Hills", ring: "Ring", fields: "Fields", bars: "Bars", wave: "Wave" };
  const IDLE_AFTER = 2600;
  const CALM_STEP_MS = 250;
  const BLEND_PER_SECOND = 2.4;
  const MAX_STEP_SECONDS = 0.1;

  let canvas: HTMLCanvasElement;
  let coverBox = $state<HTMLDivElement>();
  let idle = $state(false);
  let tuning = $state(false);
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
    scene.style = options.style;
  });

  function paletteFor(colors: Colors, cover: string | null | undefined) {
    if (colors === "mono") return Promise.resolve(monoPalette());
    if (colors === "cover" && cover) return coverPalette(cover).catch(accentPalette);
    return Promise.resolve(accentPalette());
  }

  $effect(() => {
    const cover = player.song?.cover;
    let current = true;
    const palette = paletteFor(options.colors, cover);
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
      levels.update(seconds, playing, calm, options.intensity);
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
    resting = setTimeout(() => (idle = !tuning), IDLE_AFTER);
  }

  function change(update: () => void) {
    update();
    options.save();
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
  {#if options.style === "ring" && player.song}
    <div class="absolute inset-0 flex items-center justify-center pb-10">
      <div bind:this={coverBox} in:scale={{ start: 0.92, duration: motion(400), easing: cubicOut }} class="aspect-square h-[min(36vh,360px)] overflow-hidden rounded-[22px]">
        <Cover cover={player.song.cover} label={title(player.song)} size="fill" large class="rounded-[22px]!" />
      </div>
    </div>
  {/if}
  <div class="drag absolute inset-x-0 top-0 h-12"></div>
  {#if player.song && options.info !== "hidden"}
    <div class="absolute bottom-9 left-10 max-w-[min(640px,calc(100%-560px))] min-w-0 transition-opacity duration-500 {idle && options.info === 'controls' ? 'opacity-0' : 'opacity-100'}">
      <div class="display truncate text-[28px] font-semibold">{title(player.song)}</div>
      <div class="truncate text-[15px] text-paper-2">{artist(player.song)}</div>
    </div>
  {/if}
  <div class="absolute right-10 bottom-9 flex items-center gap-2 transition-opacity duration-500 {idle ? 'opacity-0' : 'opacity-100'}">
    <Segmented value={options.style} label="Visualizer style" options={STYLES.map((value) => ({ value, label: NAMES[value] }))} onchange={(style) => change(() => (options.style = style))} />
    <Popover bind:open={tuning} align="right" width={300}>
      {#snippet trigger(toggle)}
        <button
          type="button"
          aria-label="Look"
          title="Look"
          class="flex size-[38px] items-center justify-center rounded-lg transition duration-150 hover:bg-ink-4 hover:text-paper active:scale-95 {tuning ? 'bg-ink-4 text-paper' : 'bg-ink-3 text-paper-2'}"
          onclick={toggle}
        >
          <Icon name="options" size={18} />
        </button>
      {/snippet}
      {#snippet children()}
        <div class="flex flex-col gap-4">
          <div class="flex flex-col gap-2">
            <span class="text-[13px] text-paper-2">Colours</span>
            <Segmented
              value={options.colors}
              label="Colours"
              options={[
                { value: "cover" as Colors, label: "Cover" },
                { value: "accent" as Colors, label: "Accent" },
                { value: "mono" as Colors, label: "Mono" },
              ]}
              onchange={(colors) => change(() => (options.colors = colors))}
            />
          </div>
          <div class="flex flex-col gap-2">
            <span class="text-[13px] text-paper-2">Song</span>
            <Segmented
              value={options.info}
              label="Song"
              options={[
                { value: "always" as Info, label: "Always" },
                { value: "controls" as Info, label: "With controls" },
                { value: "hidden" as Info, label: "Hidden" },
              ]}
              onchange={(info) => change(() => (options.info = info))}
            />
          </div>
          <div class="flex items-center gap-3">
            <span class="w-16 shrink-0 text-[13px] text-paper-2">Intensity</span>
            <Slider bind:value={options.intensity} min={0.4} max={1.8} step={0.05} label="Intensity" oncommit={() => options.save()} />
            <span class="numeric w-10 shrink-0 text-right text-[12px] text-paper-2">{Math.round(options.intensity * 100)}%</span>
          </div>
        </div>
      {/snippet}
    </Popover>
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
