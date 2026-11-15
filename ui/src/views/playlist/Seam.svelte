<script lang="ts">
  import { commands, type Playlist, type Song, type Transition } from "../../lib/bindings";
  import { title } from "../../lib/format";
  import { info } from "../../lib/transitions";
  import TransitionGlyph from "../../components/media/TransitionGlyph.svelte";
  import TransitionPicker from "../../components/media/TransitionPicker.svelte";
  import Popover from "../../components/ui/Popover.svelte";

  let { playlist, from, to, fallback }: { playlist: Playlist; from: Song; to: Song; fallback: Transition } = $props();

  const own = $derived(playlist.pairs.find((pair) => pair.from === from.id && pair.to === to.id)?.transition ?? null);
  const inherited = $derived(from.transition ?? fallback);
  const shown = $derived(own ?? inherited);
  const length = $derived(shown.kind === "none" ? "" : ` · ${shown.seconds < 2 ? shown.seconds.toFixed(1) : Math.round(shown.seconds)} s`);

  function set(transition: Transition | null) {
    void commands.playlistsSetPair({ id: playlist.id, from: from.id, to: to.id, transition });
  }
</script>

<div class="relative flex h-8 items-center pl-[62px]">
  <span class="absolute top-0 bottom-0 left-[74px] w-px bg-line-strong" aria-hidden="true"></span>
  <Popover width={340}>
    {#snippet trigger(toggle)}
      <button
        type="button"
        aria-label="{info(shown.kind).label} into {title(to)}"
        class="relative flex h-7 items-center gap-2 rounded-full pr-3 pl-2 text-[12px] transition-[background-color,color,transform] duration-150 active:scale-95 {own
          ? 'bg-ink-3 text-paper-2 hover:bg-ink-4'
          : 'bg-ink text-paper-4 hover:bg-ink-3 hover:text-paper-3'}"
        onclick={toggle}
      >
        <TransitionGlyph kind={shown.kind} width={26} class={own ? "" : "opacity-50"} />
        {info(shown.kind).label}{length}
      </button>
    {/snippet}
    {#snippet children()}
      <div class="mb-3 truncate text-[13px] text-paper-2">Into {title(to)}</div>
      <TransitionPicker value={own} fallback={{ label: from.transition ? "The song's own" : "Default", transition: inherited }} onchange={set} />
    {/snippet}
  </Popover>
</div>
