<script lang="ts">
  import type { Context, Song } from "../../lib/bindings";
  import { artist, clock, title } from "../../lib/format";
  import { songMenu } from "../../lib/actions";
  import { player } from "../../lib/stores/player.svelte";
  import { ui, type MenuItem } from "../../lib/stores/ui.svelte";
  import Cover from "./Cover.svelte";
  import Icon from "../ui/Icon.svelte";

  const ROW = 52;
  const OVERSCAN = 8;

  let {
    songs,
    context,
    numbered = false,
    showAlbum = true,
    menuExtra,
    scroller,
  }: {
    songs: Song[];
    context: Context;
    numbered?: boolean;
    showAlbum?: boolean;
    menuExtra?: (song: Song, index: number) => MenuItem[];
    scroller: HTMLElement | undefined;
  } = $props();

  let list: HTMLDivElement;
  let top = $state(0);
  let viewport = $state(800);
  let selected = $state<string | null>(null);

  /** Scrolling only reads the scroll offset; the list's place is measured when sizes change. */
  $effect(() => {
    const area = scroller;
    if (!area) return;
    let offset = 0;
    const scrolled = () => {
      const row = Math.floor((area.scrollTop - offset) / ROW);
      if (row !== top) top = row;
    };
    const measure = () => {
      offset = list.getBoundingClientRect().top - area.getBoundingClientRect().top + area.scrollTop;
      viewport = area.clientHeight;
      scrolled();
    };
    measure();
    area.addEventListener("scroll", scrolled, { passive: true });
    const observer = new ResizeObserver(measure);
    observer.observe(area);
    observer.observe(list);
    return () => {
      area.removeEventListener("scroll", scrolled);
      observer.disconnect();
    };
  });

  const first = $derived(Math.max(0, top - OVERSCAN));
  const last = $derived(Math.min(songs.length, top + Math.ceil(viewport / ROW) + OVERSCAN));
  const visible = $derived(songs.slice(first, last));

  function play(index: number) {
    void player.play(songs.map((song) => song.id), index, context);
  }

  function menu(event: MouseEvent, song: Song, index: number) {
    selected = song.id;
    ui.openMenu(event, songMenu([song], context, songs[index + 1] ?? null, menuExtra?.(song, index) ?? []));
  }
</script>

<div bind:this={list} class="relative" style="height:{songs.length * ROW}px" role="listbox" aria-label="Songs">
  {#each visible as song, offset (song.id + (first + offset))}
    {@const index = first + offset}
    {@const playing = player.song?.id === song.id}
    <div
      role="option"
      tabindex="-1"
      aria-selected={selected === song.id}
      class="group absolute inset-x-0 grid items-center gap-4 rounded-xl px-3 transition-colors duration-150 {showAlbum ? 'grid-cols-[28px_minmax(0,2.2fr)_minmax(0,1.4fr)_64px_56px]' : 'grid-cols-[28px_minmax(0,1fr)_64px_56px]'} {selected === song.id ? 'bg-ink-3' : 'hover:bg-ink-2'}"
      style="top:{index * ROW}px;height:{ROW}px"
      onclick={() => (selected = song.id)}
      ondblclick={() => play(index)}
      onkeydown={(event) => event.key === "Enter" && play(index)}
      oncontextmenu={(event) => menu(event, song, index)}
    >
      <div class="relative flex h-full items-center justify-center text-[12px] text-paper-3">
        {#if playing}
          <Icon name={player.state.playing ? "play" : "pause"} size={14} class="text-accent" />
        {:else}
          <span class="numeric group-hover:invisible">{numbered ? (song.track ?? "") : index + 1}</span>
          <button type="button" aria-label="Play {title(song)}" class="absolute inset-0 hidden items-center justify-center text-paper group-hover:flex" onclick={() => play(index)}>
            <Icon name="play" size={14} />
          </button>
        {/if}
      </div>
      <div class="flex min-w-0 items-center gap-3">
        {#if !numbered}<Cover cover={song.cover} label={title(song)} size={36} />{/if}
        <div class="min-w-0">
          <div class="flex items-center gap-2 truncate text-[14px] {playing ? 'text-accent' : ''}">
            <span class="truncate">{title(song)}</span>
            {#if song.edited}<Icon name="edit" size={13} class="text-paper-4" />{/if}
          </div>
          <div class="truncate text-[13px] text-paper-3">{artist(song)}</div>
        </div>
      </div>
      {#if showAlbum}
        <div class="truncate text-[13px] text-paper-3">{song.album ?? song.folder}</div>
      {/if}
      <div class="numeric text-right text-[11px] text-paper-4">{song.tempo ? Math.round(song.tempo) : ""}</div>
      <div class="numeric text-right text-[12px] text-paper-3">{clock(song.duration)}</div>
    </div>
  {/each}
</div>
