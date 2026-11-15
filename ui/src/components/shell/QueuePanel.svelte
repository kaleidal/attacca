<script lang="ts">
  import { fly } from "svelte/transition";
  import { commands } from "../../lib/bindings";
  import { artist, clock, title } from "../../lib/format";
  import { library } from "../../lib/stores/library.svelte";
  import { player } from "../../lib/stores/player.svelte";
  import { ui } from "../../lib/stores/ui.svelte";
  import Cover from "../media/Cover.svelte";
  import TransitionGlyph from "../media/TransitionGlyph.svelte";
  import Icon from "../ui/Icon.svelte";

  const offset = $derived(player.state.index === null ? 0 : player.state.index + 1);
  let dragging = $state<number | null>(null);
  let over = $state<number | null>(null);

  function drop(index: number) {
    if (dragging === null) return;
    const entry = player.upNext[dragging];
    if (entry) void commands.playerReorder({ token: entry.token, to: offset + index });
    dragging = null;
    over = null;
  }
</script>

<aside transition:fly={{ x: 24, duration: 200 }} class="absolute top-0 right-0 bottom-0 z-30 flex w-[360px] flex-col bg-ink-2 shadow-[-24px_0_48px_rgb(0_0_0/0.35)]">
  <div class="flex items-center justify-between px-5 pt-4 pb-3">
    <h2 class="display text-lg font-semibold">Up next</h2>
    <div class="flex items-center gap-1">
      {#if player.upNext.length}
        <button type="button" class="rounded-lg px-2.5 py-1 text-[13px] text-paper-3 hover:bg-ink-3 hover:text-paper" onclick={() => commands.playerClear()}>Clear</button>
      {/if}
      <button type="button" aria-label="Close queue" class="flex size-8 items-center justify-center rounded-lg text-paper-3 hover:bg-ink-3 hover:text-paper" onclick={() => (ui.queueOpen = false)}>
        <Icon name="close" size={16} />
      </button>
    </div>
  </div>
  <div class="scroll flex-1 px-3 pb-4">
    {#if player.song}
      <div class="mb-2 flex items-center gap-3 rounded-xl bg-ink-3 p-2.5">
        <Cover cover={player.song.cover} label={title(player.song)} size={40} />
        <div class="min-w-0 flex-1">
          <div class="truncate text-[14px] text-accent">{title(player.song)}</div>
          <div class="truncate text-[12px] text-paper-3">{artist(player.song)}</div>
        </div>
      </div>
    {/if}
    {#each player.upNext as entry, index (entry.token)}
      {@const song = library.byId.get(entry.song)}
      {#if index === 0 && player.state.upcoming}
        <div class="flex items-center gap-2 py-1 pl-14 text-[12px] text-paper-4">
          <TransitionGlyph kind={player.state.upcoming.kind} width={28} />
          {#if player.state.upcoming.overlap > 0}<span class="numeric">{player.state.upcoming.overlap.toFixed(1)} s</span>{/if}
        </div>
      {/if}
      {#if song}
        <div
          role="listitem"
          draggable="true"
          class="group flex items-center gap-3 rounded-xl p-2 hover:bg-ink-3 {over === index ? 'shadow-[inset_0_2px_0_var(--accent)]' : ''} {dragging === index ? 'opacity-40' : ''}"
          ondragstart={() => (dragging = index)}
          ondragover={(event) => (event.preventDefault(), (over = index))}
          ondragend={() => ((dragging = null), (over = null))}
          ondrop={() => drop(index)}
          ondblclick={() => commands.playerJump({ token: entry.token })}
        >
          <Icon name="grip" size={14} class="text-paper-4 opacity-0 group-hover:opacity-100" />
          <Cover cover={song.cover} label={title(song)} size={36} />
          <div class="min-w-0 flex-1">
            <div class="truncate text-[13px]">{title(song)}</div>
            <div class="truncate text-[12px] text-paper-3">{artist(song)}</div>
          </div>
          <span class="numeric text-[11px] text-paper-4 group-hover:hidden">{clock(song.duration)}</span>
          <button type="button" aria-label="Remove from queue" class="hidden size-7 items-center justify-center rounded-md text-paper-3 group-hover:flex hover:text-paper" onclick={() => commands.playerRemove({ tokens: [entry.token] })}>
            <Icon name="close" size={14} />
          </button>
        </div>
      {/if}
    {:else}
      <p class="px-2 py-6 text-[13px] text-paper-4">Nothing queued after this song.</p>
    {/each}
  </div>
</aside>
