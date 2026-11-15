<script lang="ts">
  import type { Snippet } from "svelte";
  import { cubicOut } from "svelte/easing";
  import { fade, scale } from "svelte/transition";
  import { motion } from "../../lib/motion";

  let { title, onclose, children, width = 440 }: { title: string; onclose: () => void; children: Snippet; width?: number } = $props();
</script>

<svelte:window onkeydown={(event) => event.key === "Escape" && onclose()} />

<div transition:fade={{ duration: motion(160) }} class="fixed inset-0 z-40 flex items-center justify-center bg-[rgb(8_7_6/0.72)] backdrop-blur-[2px]" role="presentation" onpointerdown={(event) => event.target === event.currentTarget && onclose()}>
  <div transition:scale={{ start: 0.96, duration: motion(200), easing: cubicOut }} role="dialog" aria-modal="true" aria-label={title} class="flex max-h-[86vh] flex-col rounded-2xl bg-ink-2 p-6 shadow-[0_24px_80px_rgb(0_0_0/0.6)]" style="width:{width}px">
    <h2 class="display mb-4 text-xl font-semibold">{title}</h2>
    {@render children()}
  </div>
</div>
