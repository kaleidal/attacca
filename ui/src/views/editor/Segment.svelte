<script lang="ts">
  import { WAVEFORM_RATE } from "../../lib/media";

  let {
    peaks,
    from,
    to,
    height = 96,
    start,
    end,
    beats = [],
    downbeat = 0,
    position = null,
    ghosts = [],
    onpick,
    onrelease,
    handles,
  }: {
    peaks: Uint8Array | null;
    from: number;
    to: number;
    height?: number;
    start: number;
    end: number;
    beats?: number[];
    downbeat?: number;
    position?: number | null;
    ghosts?: number[];
    onpick?: (seconds: number, handle: "start" | "end" | null) => void;
    onrelease?: () => void;
    handles: ("start" | "end")[];
  } = $props();

  let canvas: HTMLCanvasElement;
  let width = $state(0);
  let dragging = $state<"start" | "end" | null>(null);

  const span = $derived(Math.max(0.001, to - from));
  const left = (seconds: number) => ((seconds - from) / span) * 100;

  function css(name: string) {
    return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  }

  $effect(() => {
    const ratio = window.devicePixelRatio || 1;
    canvas.width = Math.round(width * ratio);
    canvas.height = Math.round(height * ratio);
    const context = canvas.getContext("2d")!;
    context.setTransform(ratio, 0, 0, ratio, 0, 0);
    context.clearRect(0, 0, width, height);
    const middle = height / 2;
    for (const [index, beat] of beats.entries()) {
      if (beat < from || beat > to) continue;
      const x = ((beat - from) / span) * width;
      context.fillStyle = index % 4 === downbeat ? css("--color-line-strong") : css("--color-line");
      context.fillRect(Math.round(x), 0, 1, height);
    }
    if (!peaks) return;
    const buckets = peaks.length / 2;
    const columns = Math.floor(width / 2);
    for (let column = 0; column < columns; column++) {
      const t0 = from + (column / columns) * span;
      const t1 = from + ((column + 1) / columns) * span;
      const b0 = Math.max(0, Math.floor(t0 * WAVEFORM_RATE));
      const b1 = Math.min(buckets, Math.max(b0 + 1, Math.ceil(t1 * WAVEFORM_RATE)));
      let peak = 0;
      for (let b = b0; b < b1; b++) peak = Math.max(peak, peaks[b * 2]!);
      const half = Math.max(0.5, (peak / 255) * middle * 0.94);
      const centre = (t0 + t1) / 2;
      context.fillStyle = centre < start || centre > end ? css("--color-ink-4") : css("--color-paper-2");
      context.fillRect(column * 2, middle - half, 1.4, half * 2);
    }
  });

  function seconds(event: PointerEvent): number {
    const rect = canvas.getBoundingClientRect();
    return from + Math.min(1, Math.max(0, (event.clientX - rect.left) / rect.width)) * span;
  }

  function down(event: PointerEvent) {
    const at = seconds(event);
    const near = handles.toSorted((a, b) => Math.abs((a === "start" ? start : end) - at) - Math.abs((b === "start" ? start : end) - at))[0] ?? null;
    dragging = near;
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    onpick?.(at, near);
  }

  function up() {
    if (dragging) onrelease?.();
    dragging = null;
  }
</script>

<div
  role="presentation"
  class="relative w-full touch-none overflow-hidden rounded-xl bg-ink-2"
  style="height:{height}px"
  bind:clientWidth={width}
  onpointerdown={down}
  onpointermove={(event) => dragging && onpick?.(seconds(event), dragging)}
  onpointerup={up}
  onpointercancel={up}
>
  <canvas bind:this={canvas} class="absolute inset-0 size-full"></canvas>
  {#if start > from}
    <div class="pointer-events-none absolute inset-y-0 left-0 bg-ink/55" style="width:{Math.min(100, left(start))}%"></div>
  {/if}
  {#if end < to}
    <div class="pointer-events-none absolute inset-y-0 right-0 bg-ink/55" style="width:{Math.min(100, 100 - left(end))}%"></div>
  {/if}
  {#each ghosts as ghost, index (index)}
    {#if ghost >= from && ghost <= to}
      <div class="pointer-events-none absolute inset-y-2 w-px border-l border-dashed border-paper-3/60" style="left:{left(ghost)}%"></div>
    {/if}
  {/each}
  {#if position !== null && position >= from && position <= to}
    <div class="pointer-events-none absolute inset-y-0 w-px bg-paper/70" style="left:{left(position)}%"></div>
  {/if}
  {#each handles as handle (handle)}
    {@const at = handle === "start" ? start : end}
    {#if at >= from && at <= to}
      <div class="pointer-events-none absolute inset-y-0 w-[2px] -translate-x-1/2 bg-accent" style="left:{left(at)}%">
        <div class="absolute top-0 size-2.5 -translate-x-[4px] rounded-b-sm bg-accent {handle === 'end' ? 'right-0 translate-x-[4px]' : 'left-0'}"></div>
      </div>
    {/if}
  {/each}
</div>
