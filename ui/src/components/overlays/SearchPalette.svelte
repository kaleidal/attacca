<script lang="ts">
  import { cubicOut } from "svelte/easing";
  import { motion } from "../../lib/motion";
  import { fade, fly } from "svelte/transition";
  import { artist, matches, title } from "../../lib/format";
  import { library } from "../../lib/stores/library.svelte";
  import { player } from "../../lib/stores/player.svelte";
  import { ui } from "../../lib/stores/ui.svelte";
  import Cover from "../media/Cover.svelte";
  import Icon from "../ui/Icon.svelte";

  interface Result {
    key: string;
    group: string;
    cover: string | null;
    round: boolean;
    primary: string;
    secondary: string;
    open: () => void;
  }

  let query = $state("");
  let selected = $state(0);
  let input: HTMLInputElement;

  $effect(() => input.focus());

  const results = $derived.by((): Result[] => {
    const text = query.trim().toLowerCase();
    if (!text) return [];
    const songs = library.songs.filter((song) => matches(title(song), text) || matches(song.artist, text) || matches(song.album, text)).slice(0, 6);
    const albums = library.albums.filter((album) => matches(album.title, text) || matches(album.artist, text)).slice(0, 4);
    const artists = library.artists.filter((entry) => matches(entry.name, text)).slice(0, 3);
    const playlists = library.playlists.filter((playlist) => matches(playlist.name, text)).slice(0, 3);
    const ids = songs.map((song) => song.id);
    return [
      ...songs.map((song, index) => ({
        key: `song:${song.id}`,
        group: "Songs",
        cover: song.cover,
        round: false,
        primary: title(song),
        secondary: artist(song),
        open: () => void player.play(ids, index, { type: "songs" }),
      })),
      ...albums.map((album) => ({
        key: `album:${album.key}`,
        group: "Albums",
        cover: album.cover,
        round: false,
        primary: album.title,
        secondary: album.artist,
        open: () => ui.go({ view: "album", key: album.key }),
      })),
      ...artists.map((entry) => ({
        key: `artist:${entry.name}`,
        group: "Artists",
        cover: entry.cover,
        round: true,
        primary: entry.name,
        secondary: `${entry.songs.length} songs`,
        open: () => ui.go({ view: "artist", name: entry.name }),
      })),
      ...playlists.map((playlist) => ({
        key: `playlist:${playlist.id}`,
        group: "Playlists",
        cover: playlist.cover ?? playlist.covers[0] ?? null,
        round: false,
        primary: playlist.name,
        secondary: `${playlist.count} songs`,
        open: () => ui.go({ view: "playlist", id: playlist.id }),
      })),
    ];
  });

  $effect(() => {
    void query;
    selected = 0;
  });

  function choose(result: Result | undefined) {
    if (!result) return;
    ui.searching = false;
    result.open();
  }

  function key(event: KeyboardEvent) {
    if (event.key === "Escape") ui.searching = false;
    else if (event.key === "ArrowDown") {
      event.preventDefault();
      selected = Math.min(results.length - 1, selected + 1);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      selected = Math.max(0, selected - 1);
    } else if (event.key === "Enter") choose(results[selected]);
  }
</script>

<div role="presentation" transition:fade={{ duration: motion(140) }} class="fixed inset-0 z-50 bg-[rgb(8_7_6/0.6)] backdrop-blur-[3px]" onpointerdown={(event) => event.target === event.currentTarget && (ui.searching = false)}>
  <div transition:fly={{ y: -10, duration: motion(180), easing: cubicOut }} class="mx-auto mt-[13vh] w-[min(640px,calc(100%-48px))] overflow-hidden rounded-3xl bg-ink-3 shadow-[0_30px_90px_rgb(0_0_0/0.65)]">
    <div class="flex h-16 items-center gap-3 px-5">
      <Icon name="search" size={20} class="text-paper-3" />
      <input
        bind:this={input}
        bind:value={query}
        aria-label="Search"
        placeholder="Songs, albums, artists, playlists"
        spellcheck="false"
        class="h-full flex-1 bg-transparent text-[18px] text-paper outline-none placeholder:text-paper-4"
        onkeydown={key}
      />
    </div>
    {#if results.length}
      <div class="scroll max-h-[52vh] px-2 pb-2">
        {#each results as result, index (result.key)}
          {#if index === 0 || results[index - 1]!.group !== result.group}
            <div class="px-3 pt-3 pb-1 text-[12px] text-paper-4">{result.group}</div>
          {/if}
          <button
            type="button"
            class="flex w-full items-center gap-3 rounded-2xl px-3 py-2 text-left {index === selected ? 'bg-ink-5' : 'hover:bg-ink-4'}"
            onpointermove={() => (selected = index)}
            onclick={() => choose(result)}
          >
            <Cover cover={result.cover} label={result.primary} size={36} class={result.round ? "rounded-full!" : "rounded-lg!"} />
            <div class="min-w-0">
              <div class="truncate text-[14px]">{result.primary}</div>
              <div class="truncate text-[12px] text-paper-3">{result.secondary}</div>
            </div>
          </button>
        {/each}
      </div>
    {:else if query.trim()}
      <p class="px-6 pb-6 text-[14px] text-paper-3">Nothing matches.</p>
    {/if}
  </div>
</div>
