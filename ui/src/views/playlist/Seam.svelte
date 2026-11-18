<script lang="ts">
  import { slide } from "svelte/transition";
  import type { Context, Playlist, Song, Transition } from "../../lib/bindings";
  import { title } from "../../lib/format";
  import { motion } from "../../lib/motion";
  import { info } from "../../lib/transitions";
  import TransitionGlyph from "../../components/media/TransitionGlyph.svelte";
  import SeamEditor from "../../components/seam/SeamEditor.svelte";

  let { playlist, from, to, fallback, context }: { playlist: Playlist; from: Song; to: Song; fallback: Transition; context: Context } = $props();

  let open = $state(false);
  const own = $derived(playlist.pairs.find((pair) => pair.from === from.id && pair.to === to.id)?.transition ?? null);
  const shown = $derived(own ?? from.transition ?? fallback);
  const length = $derived(shown.kind === "none" ? "" : ` · ${shown.seconds < 2 ? shown.seconds.toFixed(1) : Math.round(shown.seconds)} s`);
</script>

<div class="relative flex h-8 items-center pl-[62px]">
  <span class="absolute top-0 bottom-0 left-[74px] w-px bg-line-strong" aria-hidden="true"></span>
  <button
    type="button"
    aria-label="{info(shown.kind).label} into {title(to)}"
    aria-expanded={open}
    class="relative flex h-7 items-center gap-2 rounded-full pr-3 pl-2 text-[12px] transition-[background-color,color,transform] duration-150 active:scale-95 {open || own
      ? 'bg-ink-3 text-paper-2 hover:bg-ink-4'
      : 'bg-ink text-paper-4 hover:bg-ink-3 hover:text-paper-3'}"
    onclick={() => (open = !open)}
  >
    <TransitionGlyph kind={shown.kind} width={26} class={own || open ? "" : "opacity-50"} />
    {info(shown.kind).label}{length}
  </button>
</div>
{#if open}
  <div transition:slide={{ duration: motion(220) }} class="pt-1 pb-3 pl-[62px]">
    <SeamEditor {from} {to} {context} />
  </div>
{/if}
