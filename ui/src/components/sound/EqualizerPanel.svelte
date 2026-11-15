<script lang="ts">
  import type { Equalizer } from "../../lib/bindings";
  import { decibels } from "../../lib/format";
  import Slider from "../ui/Slider.svelte";

  const BANDS = ["31", "62", "125", "250", "500", "1k", "2k", "4k", "8k", "16k"];

  let { equalizer, onchange }: { equalizer: Equalizer; onchange: (equalizer: Equalizer) => void } = $props();

  let gains = $state<number[]>([]);
  let preamp = $state(0);
  $effect(() => {
    gains = [...equalizer.gains];
    preamp = equalizer.preamp;
  });

  const curve = $derived.by(() => {
    const points = gains.map((gain, index) => [((index + 0.5) / BANDS.length) * 100, 50 - (gain / 12) * 46] as const);
    if (points.length === 0) return "";
    let path = `M0 ${points[0]![1]} L${points[0]![0]} ${points[0]![1]}`;
    for (let i = 0; i < points.length - 1; i++) {
      const [x1, y1] = points[i]!;
      const [x2, y2] = points[i + 1]!;
      const middle = (x1 + x2) / 2;
      path += ` C${middle} ${y1} ${middle} ${y2} ${x2} ${y2}`;
    }
    return `${path} L100 ${points.at(-1)![1]}`;
  });

  function commit() {
    onchange({ preamp, gains: [...gains] });
  }
</script>

<div class="flex gap-6">
  <div class="flex flex-col items-center gap-2">
    <div class="h-40"><Slider bind:value={preamp} min={-12} max={12} step={0.5} centered vertical label="Preamp" oncommit={commit} /></div>
    <span class="text-[11px] text-paper-3">Pre</span>
    <span class="numeric text-[10px] text-paper-4">{decibels(preamp).replace(" dB", "")}</span>
  </div>
  <div class="relative flex-1">
    <svg class="pointer-events-none absolute inset-x-0 top-0 h-40 w-full" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
      <line x1="0" x2="100" y1="50" y2="50" stroke="var(--color-line-strong)" stroke-width="0.6" vector-effect="non-scaling-stroke" />
      <path d={curve} fill="none" stroke="var(--accent)" stroke-width="1.6" vector-effect="non-scaling-stroke" opacity="0.6" />
    </svg>
    <div class="relative grid grid-cols-10">
      {#each BANDS as band, index (band)}
        <div class="flex flex-col items-center gap-2">
          <div class="h-40"><Slider bind:value={gains[index]!} min={-12} max={12} step={0.5} centered vertical label="{band} hertz" oncommit={commit} /></div>
          <span class="text-[11px] text-paper-3">{band}</span>
          <span class="numeric text-[10px] text-paper-4">{(gains[index] ?? 0) > 0 ? "+" : ""}{(gains[index] ?? 0).toFixed(1)}</span>
        </div>
      {/each}
    </div>
  </div>
</div>
