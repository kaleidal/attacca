<script lang="ts">
  import type { Cut, Filters } from "../../lib/bindings";
  import { hertz } from "../../lib/format";
  import Segmented from "../ui/Segmented.svelte";
  import Slider from "../ui/Slider.svelte";
  import Toggle from "../ui/Toggle.svelte";

  let { filters, onchange }: { filters: Filters; onchange: (filters: Filters) => void } = $props();

  const toHz = (t: number) => 20 * Math.pow(1000, t);
  const toT = (hz: number) => Math.log(hz / 20) / Math.log(1000);
  const defaults: Record<"highpass" | "lowpass", Cut> = { highpass: { hz: 60, slope: 24 }, lowpass: { hz: 14000, slope: 24 } };
  const rows = [
    { key: "highpass", label: "High-pass" },
    { key: "lowpass", label: "Low-pass" },
  ] as const;

  let positions = $state({ highpass: 0, lowpass: 0 });
  $effect(() => {
    positions = {
      highpass: toT((filters.highpass ?? defaults.highpass).hz),
      lowpass: toT((filters.lowpass ?? defaults.lowpass).hz),
    };
  });

  function update(key: "highpass" | "lowpass", cut: Cut | null) {
    onchange({ ...filters, [key]: cut });
  }
</script>

<div class="flex flex-col gap-5">
  {#each rows as row (row.key)}
    {@const cut = filters[row.key]}
    <div class="flex flex-col gap-3">
      <div class="flex items-center justify-between">
        <span class="text-[14px]">{row.label}</span>
        <Toggle checked={cut !== null} label={row.label} onchange={(on) => update(row.key, on ? defaults[row.key] : null)} />
      </div>
      <div class="flex items-center gap-4 {cut ? '' : 'pointer-events-none opacity-35'}">
        <Slider
          bind:value={positions[row.key]}
          label="{row.label} cut-off"
          oncommit={(t) => update(row.key, { slope: cut?.slope ?? 24, hz: Math.round(toHz(t)) })}
        />
        <span class="numeric w-16 shrink-0 text-right text-[12px] text-paper-2">{hertz(toHz(positions[row.key]))}</span>
        <Segmented
          value={cut?.slope ?? 24}
          label="{row.label} slope"
          options={[12, 24, 36, 48].map((slope) => ({ value: slope, label: String(slope) }))}
          onchange={(slope) => update(row.key, { hz: cut?.hz ?? defaults[row.key].hz, slope })}
        />
      </div>
    </div>
  {/each}
  <p class="text-[12px] text-paper-4">Slopes are in decibels per octave.</p>
</div>
