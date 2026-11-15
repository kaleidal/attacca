<script lang="ts">
  import { WAVEFORM_RATE, waveform } from "../../lib/media";

  let {
    song,
    duration,
    position = 0,
    start = 0,
    end = null,
    overlap = null,
    overlapKind = "fade",
    height = 48,
    bar = 2,
    gap = 1,
    onseek,
    class: className = "",
  }: {
    song: string;
    duration: number;
    position?: number;
    start?: number;
    end?: number | null;
    overlap?: { from: number; to: number } | null;
    overlapKind?: string;
    height?: number;
    bar?: number;
    gap?: number;
    onseek?: (seconds: number) => void;
    class?: string;
  } = $props();

  let canvas: HTMLCanvasElement;
  let width = $state(0);
  let peaks = $state<Uint8Array | null>(null);
  let shape = $state.raw<OffscreenCanvas | null>(null);
  let hover = $state<number | null>(null);

  $effect(() => {
    const id = song;
    peaks = null;
    void waveform(id).then((bytes) => {
      if (id === song) peaks = bytes;
    });
  });

  function colour(name: string): string {
    return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  }

  /** Draws the bars once per size and song; playing only repaints the overlay. */
  function bake(cssWidth: number, ratio: number): OffscreenCanvas | null {
    if (!peaks || cssWidth <= 0) return null;
    const off = new OffscreenCanvas(Math.round(cssWidth * ratio), Math.round(height * ratio));
    const context = off.getContext("2d")!;
    context.scale(ratio, ratio);
    const step = bar + gap;
    const count = Math.floor(cssWidth / step);
    const buckets = peaks.length / 2;
    const middle = height / 2;
    context.fillStyle = "#fff";
    for (let i = 0; i < count; i++) {
      const from = Math.floor((i / count) * buckets);
      const to = Math.max(from + 1, Math.floor(((i + 1) / count) * buckets));
      let peak = 0;
      let rms = 0;
      for (let b = from; b < to && b < buckets; b++) {
        peak = Math.max(peak, peaks[b * 2]!);
        rms = Math.max(rms, peaks[b * 2 + 1]!);
      }
      const level = Math.max(peak * 0.55, rms * 1.4) / 255;
      const half = Math.max(0.75, Math.min(1, level) * middle);
      context.fillRect(i * step, middle - half, bar, half * 2);
    }
    return off;
  }

  $effect(() => {
    const ratio = window.devicePixelRatio || 1;
    shape = bake(width, ratio);
    canvas.width = Math.round(width * ratio);
    canvas.height = Math.round(height * ratio);
  });

  $effect(() => {
    const context = canvas.getContext("2d");
    if (!context) return;
    const ratio = window.devicePixelRatio || 1;
    const length = duration || (peaks ? peaks.length / 2 / WAVEFORM_RATE : 1);
    const x = (seconds: number) => Math.round((Math.max(0, Math.min(seconds, length)) / length) * canvas.width);
    context.clearRect(0, 0, canvas.width, canvas.height);
    if (!shape) {
      context.fillStyle = colour("--color-ink-4");
      context.fillRect(0, canvas.height / 2 - ratio, canvas.width, ratio * 2);
      return;
    }
    const paint = (from: number, to: number, fill: string) => {
      if (to <= from) return;
      context.save();
      context.beginPath();
      context.rect(from, 0, to - from, canvas.height);
      context.clip();
      context.drawImage(shape!, 0, 0);
      context.globalCompositeOperation = "source-in";
      context.fillStyle = fill;
      context.fillRect(from, 0, to - from, canvas.height);
      context.restore();
    };
    const played = x(position);
    const first = x(start);
    const last = end === null ? canvas.width : x(end);
    paint(0, first, colour("--color-ink-4"));
    paint(first, Math.min(played, last), colour("--color-paper"));
    paint(Math.max(played, first), last, colour("--color-ink-5"));
    paint(last, canvas.width, colour("--color-ink-4"));
    if (overlap) {
      const from = x(overlap.from);
      const to = x(overlap.to);
      context.fillStyle = colour(`--color-${overlapKind}`);
      context.globalAlpha = 0.16;
      context.fillRect(from, 0, to - from, canvas.height);
      context.globalAlpha = 1;
      context.fillRect(from, 0, Math.max(1, ratio), canvas.height);
    }
    if (hover !== null) {
      context.fillStyle = colour("--color-paper");
      context.globalAlpha = 0.5;
      context.fillRect(hover * ratio, 0, ratio, canvas.height);
      context.globalAlpha = 1;
    }
  });

  function seconds(event: PointerEvent): number {
    const rect = canvas.getBoundingClientRect();
    return ((event.clientX - rect.left) / rect.width) * duration;
  }
</script>

<div class="relative w-full {className}" style="height:{height}px" bind:clientWidth={width}>
  <canvas
    bind:this={canvas}
    class="absolute inset-0 size-full"
    style="height:{height}px"
    onpointermove={(event) => onseek && (hover = event.offsetX)}
    onpointerleave={() => (hover = null)}
    onpointerdown={(event) => onseek?.(seconds(event))}
  ></canvas>
</div>
