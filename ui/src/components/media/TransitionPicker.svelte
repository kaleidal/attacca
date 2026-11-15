<script lang="ts">
  import type { Transition, TransitionName } from "../../lib/bindings";
  import { fresh, info, transitions } from "../../lib/transitions";
  import Slider from "../ui/Slider.svelte";
  import Toggle from "../ui/Toggle.svelte";
  import TransitionGlyph from "./TransitionGlyph.svelte";

  let { value, fallback, onchange }: {
    value: Transition | null;
    fallback?: { label: string; transition: Transition };
    onchange: (transition: Transition | null) => void;
  } = $props();

  const shown = $derived(value ?? fallback?.transition ?? fresh("fade"));
  let seconds = $state(0);
  $effect(() => {
    seconds = shown.seconds;
  });

  const range = (kind: TransitionName) => (kind === "slam" ? [0.3, 2] : [1, 32]);

  function choose(kind: TransitionName) {
    onchange(kind === shown.kind && value ? value : { ...fresh(kind), snap: shown.snap });
  }
</script>

<div class="flex flex-col gap-4">
  <div class="grid grid-cols-4 gap-1.5">
    {#each transitions as transition (transition.name)}
      {@const selected = shown.kind === transition.name && (value !== null || !fallback)}
      <button
        type="button"
        title={transition.hint}
        class="flex flex-col items-center gap-1.5 rounded-xl px-1 py-2.5 text-[12px] transition-colors {selected ? 'bg-ink-5 text-paper' : 'text-paper-3 hover:bg-ink-3 hover:text-paper-2'}"
        onclick={() => choose(transition.name)}
      >
        <TransitionGlyph kind={transition.name} width={34} />
        {transition.label}
      </button>
    {/each}
    {#if fallback}
      <button
        type="button"
        class="flex flex-col items-center justify-center gap-1 rounded-xl px-1 py-2.5 text-center text-[12px] leading-tight transition-colors {value === null ? 'bg-ink-5 text-paper' : 'text-paper-3 hover:bg-ink-3 hover:text-paper-2'}"
        onclick={() => onchange(null)}
      >
        {fallback.label}
      </button>
    {/if}
  </div>
  <p class="text-[12px] leading-snug text-paper-3">{info(shown.kind).hint}.</p>
  {#if shown.kind !== "none"}
    <div class="flex items-center gap-3">
      <span class="w-14 text-[13px] text-paper-2">Length</span>
      <Slider
        bind:value={seconds}
        min={range(shown.kind)[0]}
        max={range(shown.kind)[1]}
        step={shown.kind === "slam" ? 0.1 : 0.5}
        label="Transition length"
        oncommit={(next) => onchange({ ...shown, seconds: next })}
      />
      <span class="numeric w-14 shrink-0 text-right text-[12px] whitespace-nowrap text-paper-2">{seconds.toFixed(1)} s</span>
    </div>
    <label class="flex items-center justify-between text-[13px] text-paper-2">
      Snap to the beat
      <Toggle checked={shown.snap} label="Snap to the beat" onchange={(snap) => onchange({ ...shown, snap })} />
    </label>
  {/if}
</div>
