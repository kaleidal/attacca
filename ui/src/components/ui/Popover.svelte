<script lang="ts">
  import type { Snippet } from "svelte";
  import { cubicOut } from "svelte/easing";
  import { scale } from "svelte/transition";
  import { motion } from "../../lib/motion";

  const GAP = 8;
  const MARGIN = 12;

  let { open = $bindable(false), trigger, children, align = "left", width = 320 }: {
    open?: boolean;
    trigger: Snippet<[() => void]>;
    children: Snippet<[() => void]>;
    align?: "left" | "right";
    width?: number;
  } = $props();

  let root: HTMLDivElement;
  let panel = $state<HTMLDivElement>();
  let place = $state({ left: 0, top: 0 });

  const toggle = () => (open = !open);
  const close = () => (open = false);

  /** Placed against the window rather than its container, so scrolling parents never clip it. */
  function position() {
    if (!panel) return;
    const anchor = root.getBoundingClientRect();
    const height = panel.offsetHeight;
    const below = anchor.bottom + GAP + height + MARGIN <= window.innerHeight || anchor.top < height + GAP + MARGIN;
    const left = align === "right" ? anchor.right - width : anchor.left;
    place = {
      left: Math.min(Math.max(MARGIN, left), window.innerWidth - width - MARGIN),
      top: below ? anchor.bottom + GAP : anchor.top - GAP - height,
    };
  }

  $effect(() => {
    if (!open || !panel) return;
    position();
    const observer = new ResizeObserver(position);
    observer.observe(panel);
    const scrolled = (event: Event) => !panel?.contains(event.target as Node) && close();
    window.addEventListener("scroll", scrolled, true);
    return () => {
      observer.disconnect();
      window.removeEventListener("scroll", scrolled, true);
    };
  });

  function outside(event: PointerEvent) {
    if (open && !root.contains(event.target as Node) && !panel?.contains(event.target as Node)) close();
  }
</script>

<svelte:window onpointerdown={outside} onkeydown={(event) => event.key === "Escape" && close()} onresize={close} />

<div bind:this={root} class="relative inline-flex">
  {@render trigger(toggle)}
</div>
{#if open}
  <div
    bind:this={panel}
    transition:scale={{ start: 0.96, duration: motion(140), easing: cubicOut }}
    class="fixed z-40 rounded-2xl bg-ink-4 p-4 shadow-[0_18px_56px_rgb(0_0_0/0.6)]"
    style="left:{place.left}px;top:{place.top}px;width:{width}px"
  >
    {@render children(close)}
  </div>
{/if}
