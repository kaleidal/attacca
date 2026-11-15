<script lang="ts">
  import { fade as appear } from "svelte/transition";
  import { artist, title } from "../../lib/format";
  import { coverUrl } from "../../lib/media";
  import { player } from "../../lib/stores/player.svelte";
  import { ui } from "../../lib/stores/ui.svelte";
  import { BANDS, WAVE, reducedMotion, visual } from "../../lib/stores/visual.svelte";
  import Icon from "../ui/Icon.svelte";
  import Segmented from "../ui/Segmented.svelte";

  type Style = "bars" | "wave" | "cover";
  let style = $state<Style>((localStorage.getItem("attacca.visualizer") as Style | null) ?? "bars");
  let canvas: HTMLCanvasElement;
  let still = $state(reducedMotion.matches);
  let palette = $state<string[]>(["#ff8f5e", "#5fc8b5", "#9d8cf0", "#f2b45a"]);

  $effect(() => {
    const listener = () => (still = reducedMotion.matches);
    reducedMotion.addEventListener("change", listener);
    return () => reducedMotion.removeEventListener("change", listener);
  });

  $effect(() => {
    localStorage.setItem("attacca.visualizer", style);
  });

  $effect(() => {
    const url = coverUrl(player.song?.cover, true);
    if (!url) return;
    const image = new Image();
    image.src = url;
    void image.decode().then(() => {
      const sample = new OffscreenCanvas(4, 4);
      const context = sample.getContext("2d")!;
      context.drawImage(image, 0, 0, 4, 4);
      const data = context.getImageData(0, 0, 4, 4).data;
      const colours: string[] = [];
      for (let i = 0; i < data.length; i += 16) colours.push(`rgb(${data[i]} ${data[i + 1]} ${data[i + 2]})`);
      palette = colours;
    });
  });

  $effect(() => {
    if (still) return;
    const stop = visual.watch();
    const context = canvas.getContext("2d")!;
    const peaks = new Float32Array(BANDS);
    const levels = new Float32Array(BANDS);
    let frame = 0;
    let phase = 0;
    const draw = () => {
      frame = requestAnimationFrame(draw);
      const scale = style === "cover" ? 0.25 : window.devicePixelRatio || 1;
      const width = Math.round(canvas.clientWidth * scale);
      const height = Math.round(canvas.clientHeight * scale);
      if (canvas.width !== width || canvas.height !== height) {
        canvas.width = width;
        canvas.height = height;
      }
      const accent = getComputedStyle(document.documentElement).getPropertyValue("--accent");
      for (let band = 0; band < BANDS; band++) {
        levels[band] = Math.max(visual.band(band), levels[band]! * 0.86);
        peaks[band] = Math.max(levels[band]!, peaks[band]! - 0.006);
      }
      if (style === "bars") {
        context.clearRect(0, 0, width, height);
        const gap = width * 0.004;
        const bar = (width * 0.8) / BANDS - gap;
        const left = width * 0.1;
        for (let band = 0; band < BANDS; band++) {
          const x = left + band * (bar + gap);
          const tall = levels[band]! * height * 0.62;
          context.fillStyle = accent;
          context.globalAlpha = 0.9;
          context.fillRect(x, height * 0.72 - tall, bar, tall);
          context.globalAlpha = 0.18;
          context.fillRect(x, height * 0.72 + 4 * scale, bar, tall * 0.35);
          context.globalAlpha = 1;
          context.fillStyle = "#efe9e1";
          context.fillRect(x, height * 0.72 - peaks[band]! * height * 0.62 - 3 * scale, bar, 2 * scale);
        }
      } else if (style === "wave") {
        context.fillStyle = "rgb(14 13 12 / 0.28)";
        context.fillRect(0, 0, width, height);
        context.strokeStyle = accent;
        context.lineWidth = 2.5 * scale;
        context.lineJoin = "round";
        context.beginPath();
        for (let point = 0; point < WAVE; point++) {
          const x = (point / (WAVE - 1)) * width;
          const y = height / 2 + visual.wave(point) * height * 0.32;
          if (point === 0) context.moveTo(x, y);
          else context.lineTo(x, y);
        }
        context.stroke();
      } else {
        phase += 0.004;
        const low = (levels[2]! + levels[4]! + levels[6]!) / 3;
        const mid = (levels[20]! + levels[28]!) / 2;
        const high = (levels[44]! + levels[52]!) / 2;
        context.globalCompositeOperation = "source-over";
        context.fillStyle = "rgb(14 13 12 / 0.2)";
        context.fillRect(0, 0, width, height);
        context.globalCompositeOperation = "lighter";
        const energies = [low, mid, high, (low + high) / 2];
        palette.forEach((colour, index) => {
          const energy = energies[index % energies.length]!;
          const angle = phase * (1 + index * 0.37) + index * 1.7;
          const x = width * (0.5 + Math.cos(angle) * 0.28);
          const y = height * (0.5 + Math.sin(angle * 1.3) * 0.24);
          const radius = Math.min(width, height) * (0.16 + energy * 0.34);
          const glow = context.createRadialGradient(x, y, 0, x, y, radius);
          glow.addColorStop(0, colour);
          glow.addColorStop(1, "transparent");
          context.globalAlpha = 0.18 + energy * 0.5;
          context.fillStyle = glow;
          context.fillRect(0, 0, width, height);
        });
        context.globalAlpha = 1;
        context.globalCompositeOperation = "source-over";
      }
    };
    draw();
    return () => {
      cancelAnimationFrame(frame);
      stop();
    };
  });
</script>

<svelte:window onkeydown={(event) => event.key === "Escape" && (ui.visualizer = false)} />

<div transition:appear={{ duration: 220 }} class="fixed inset-0 z-40 bg-ink">
  <canvas bind:this={canvas} class="absolute inset-0 size-full {style === 'cover' ? 'blur-2xl' : ''}"></canvas>
  {#if still}
    <div class="absolute inset-0 flex items-center justify-center text-[14px] text-paper-3">The visualizer rests while reduced motion is on.</div>
  {/if}
  <div class="drag absolute inset-x-0 top-0 flex h-12 items-center justify-end pr-[150px]">
    <button type="button" aria-label="Close visualizer" class="no-drag flex size-9 items-center justify-center rounded-lg text-paper-2 hover:bg-ink-3 hover:text-paper" onclick={() => (ui.visualizer = false)}>
      <Icon name="close" size={18} />
    </button>
  </div>
  <div class="absolute inset-x-10 bottom-9 flex items-end justify-between">
    {#if player.song}
      <div class="min-w-0">
        <div class="display truncate text-[28px] font-semibold">{title(player.song)}</div>
        <div class="truncate text-[15px] text-paper-2">{artist(player.song)}</div>
      </div>
    {/if}
    <Segmented
      bind:value={style}
      label="Visualizer style"
      options={[
        { value: "bars", label: "Spectrum" },
        { value: "wave", label: "Waveform" },
        { value: "cover", label: "Cover" },
      ]}
    />
  </div>
</div>
