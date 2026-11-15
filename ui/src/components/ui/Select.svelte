<script lang="ts" generics="T">
  import { cubicOut } from "svelte/easing";
  import { scale } from "svelte/transition";
  import { motion } from "../../lib/motion";
  import Icon from "./Icon.svelte";

  let { value = $bindable(), options, label, onchange, class: className = "" }: {
    value: T;
    options: { value: T; label: string }[];
    label: string;
    onchange?: (value: T) => void;
    class?: string;
  } = $props();

  let open = $state(false);
  let root: HTMLDivElement;
  const current = $derived(options.find((option) => option.value === value)?.label ?? "");

  function choose(next: T) {
    value = next;
    open = false;
    onchange?.(next);
  }

  function outside(event: PointerEvent) {
    if (open && !root.contains(event.target as Node)) open = false;
  }
</script>

<svelte:window onpointerdown={outside} onkeydown={(event) => event.key === "Escape" && (open = false)} />

<div bind:this={root} class="relative {className}">
  <button
    type="button"
    aria-haspopup="listbox"
    aria-expanded={open}
    aria-label={label}
    class="flex h-9 w-full items-center justify-between gap-3 rounded-lg bg-ink-3 px-3 text-left text-[13px] text-paper transition-colors hover:bg-ink-4"
    onclick={() => (open = !open)}
  >
    <span class="truncate">{current}</span>
    <Icon name="forward" size={14} class="text-paper-3 transition-transform duration-200 {open ? '-rotate-90' : 'rotate-90'}" />
  </button>
  {#if open}
    <div role="listbox" transition:scale={{ start: 0.96, duration: motion(130), easing: cubicOut }} class="origin-top scroll absolute top-full right-0 left-0 z-40 mt-1 max-h-64 rounded-lg bg-ink-4 p-1 shadow-[0_12px_40px_rgb(0_0_0/0.55)]">
      {#each options as option, index (index)}
        <button
          type="button"
          role="option"
          aria-selected={option.value === value}
          class="flex w-full items-center justify-between gap-2 rounded-md px-2.5 py-1.5 text-left text-[13px] hover:bg-ink-5 {option.value === value ? 'text-paper' : 'text-paper-2'}"
          onclick={() => choose(option.value)}
        >
          <span class="truncate">{option.label}</span>
          {#if option.value === value}<Icon name="check" size={14} class="text-accent" />{/if}
        </button>
      {/each}
    </div>
  {/if}
</div>
