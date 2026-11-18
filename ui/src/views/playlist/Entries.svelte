<script lang="ts">
  import { flip } from "svelte/animate";
  import { fly } from "svelte/transition";
  import { type Context, type Playlist, type Song, type Transition } from "../../lib/bindings";
  import { songMenu } from "../../lib/actions";
  import { artist, clock, title } from "../../lib/format";
  import { library } from "../../lib/stores/library.svelte";
  import { player } from "../../lib/stores/player.svelte";
  import { ui, type MenuItem } from "../../lib/stores/ui.svelte";
  import Cover from "../../components/media/Cover.svelte";
  import Icon from "../../components/ui/Icon.svelte";
  import { motion } from "../../lib/motion";
  import { Reorder } from "./reorder.svelte";
  import Seam from "./Seam.svelte";

  let { playlist, context, fallback, songAfter, onplay, onorder }: {
    playlist: Playlist;
    context: Context;
    fallback: Transition;
    songAfter: (song: Song) => Song | null;
    onplay: (song: Song) => void;
    onorder: (entries: number[]) => void;
  } = $props();

  let list = $state<HTMLDivElement>();
  const rows: HTMLDivElement[] = [];
  const ids = () => playlist.entries.map((entry) => entry.id);

  function move(from: number, to: number) {
    const order = ids();
    const [moved] = order.splice(from, 1);
    order.splice(to, 0, moved!);
    onorder(order);
  }

  const reorder = new Reorder(
    () => list,
    () => rows.slice(0, playlist.entries.length),
    move,
  );

  function entryMenu(index: number): MenuItem[] {
    const id = playlist.entries[index]!.id;
    const last = playlist.entries.length - 1;
    return [
      ...(index > 0 ? [{ label: "Move to the top", action: () => move(index, 0) }] : []),
      ...(index < last ? [{ label: "Move to the bottom", action: () => move(index, last) }] : []),
      { label: "Remove from playlist", action: () => onorder(ids().filter((entry) => entry !== id)), danger: true },
    ];
  }

  function menu(event: MouseEvent, index: number, song: Song | undefined) {
    ui.openMenu(event, song ? songMenu([song], context, songAfter(song), entryMenu(index)) : entryMenu(index));
  }
</script>

<div bind:this={list} role="list" class="relative">
  {#each playlist.entries as entry, index (entry.id)}
    {@const song = entry.song ? library.byId.get(entry.song) : undefined}
    {@const following = playlist.entries[index + 1]}
    {@const next = following?.song ? library.byId.get(following.song) : undefined}
    {@const lifted = reorder.dragging === index}
    <div role="listitem" animate:flip={{ duration: motion(220) }} in:fly={{ y: 8, duration: motion(260), delay: motion(Math.min(index, 14) * 18) }}>
      <div
        bind:this={rows[index]}
        role="presentation"
        class="group relative grid h-[52px] cursor-grab grid-cols-[28px_minmax(0,2fr)_minmax(0,1.3fr)_56px_32px] items-center gap-4 rounded-xl px-3 {lifted
          ? 'z-10 cursor-grabbing bg-ink-3 shadow-[0_14px_40px_rgb(0_0_0/0.5)]'
          : 'transition-colors duration-150 hover:bg-ink-2'}"
        style:transform={lifted ? `translateY(${reorder.lift}px)` : undefined}
        onpointerdown={(event) => reorder.press(event, index)}
        ondblclick={() => song && onplay(song)}
        oncontextmenu={(event) => menu(event, index, song)}
      >
        <div class="flex justify-center text-paper-4">
          <span class="numeric text-[12px] group-hover:hidden {lifted ? 'hidden' : ''}">{index + 1}</span>
          <Icon name="grip" size={14} class="hidden group-hover:block {lifted ? 'block!' : ''}" />
        </div>
        {#if song}
          <div class="flex min-w-0 items-center gap-3">
            <button type="button" aria-label="Play {title(song)}" class="relative shrink-0" onclick={() => onplay(song)}>
              <Cover cover={song.cover} label={title(song)} size={36} />
              <span class="absolute inset-0 flex items-center justify-center rounded-(--radius-cover) bg-[rgb(14_13_12/0.6)] text-paper opacity-0 transition-opacity duration-150 group-hover:opacity-100">
                <Icon name="play" size={14} />
              </span>
            </button>
            <div class="min-w-0">
              <div class="truncate text-[14px] {player.song?.id === song.id ? 'text-accent' : ''}">{title(song)}</div>
              <div class="truncate text-[13px] text-paper-3">{artist(song)}</div>
            </div>
          </div>
          <div class="truncate text-[13px] text-paper-3">{song.album ?? song.folder}</div>
          <div class="numeric text-right text-[12px] text-paper-3">{clock(song.duration)}</div>
        {:else if entry.missing}
          <div class="flex min-w-0 items-center gap-3 text-paper-4">
            <div class="size-9 shrink-0 rounded-(--radius-cover) border border-dashed border-line-strong"></div>
            <div class="min-w-0">
              <div class="truncate text-[14px]">{entry.missing.title}</div>
              <div class="truncate text-[13px] italic">Not in your library yet</div>
            </div>
          </div>
          <div class="truncate text-[13px] text-paper-4">{entry.missing.artist ?? ""}</div>
          <div class="numeric text-right text-[12px] text-paper-4">{clock(entry.missing.duration)}</div>
        {/if}
        <button
          type="button"
          aria-label="More"
          class="flex size-8 items-center justify-center rounded-lg text-paper-3 opacity-0 transition-opacity group-hover:opacity-100 hover:bg-ink-4 hover:text-paper"
          onclick={(event) => menu(event, index, song)}
        >
          <Icon name="more" size={18} />
        </button>
      </div>
      {#if song && next}
        <div class="transition-opacity duration-150 {reorder.dragging !== null ? 'pointer-events-none opacity-25' : ''}">
          <Seam {playlist} from={song} to={next} {fallback} {context} />
        </div>
      {/if}
    </div>
  {:else}
    <p class="py-10 text-center text-[13px] text-paper-4">Add songs from their menu, with a right click.</p>
  {/each}
  {#if reorder.landing !== null}
    <div class="pointer-events-none absolute inset-x-3 h-[3px] -translate-y-1/2 rounded-full bg-accent" style="top:{reorder.marker}px" aria-hidden="true"></div>
  {/if}
</div>
