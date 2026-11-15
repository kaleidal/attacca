<script lang="ts">
  import type { Song } from "../../lib/bindings";
  import { artist, compare, count, length, title } from "../../lib/format";
  import { library } from "../../lib/stores/library.svelte";
  import { player } from "../../lib/stores/player.svelte";
  import SongList from "../../components/media/SongList.svelte";
  import Button from "../../components/ui/Button.svelte";
  import Select from "../../components/ui/Select.svelte";
  import Empty from "./Empty.svelte";

  let { recent = false }: { recent?: boolean } = $props();

  type Order = "title" | "artist" | "album" | "added" | "length";
  let order = $state<Order>("title");
  let scroller = $state<HTMLElement>();

  const sorters: Record<Order, (a: Song, b: Song) => number> = {
    title: (a, b) => compare(title(a), title(b)),
    artist: (a, b) => compare(artist(a), artist(b)) || compare(a.album ?? "", b.album ?? "") || (a.track ?? 0) - (b.track ?? 0),
    album: (a, b) => compare(a.album ?? a.folder, b.album ?? b.folder) || (a.disc ?? 1) - (b.disc ?? 1) || (a.track ?? 0) - (b.track ?? 0),
    added: (a, b) => b.added - a.added,
    length: (a, b) => a.duration - b.duration,
  };

  const songs = $derived(recent ? [...library.songs].sort(sorters.added).slice(0, 300) : [...library.songs].sort(sorters[order]));
  const total = $derived(songs.reduce((sum, song) => sum + song.duration, 0));
</script>

{#if library.ready && library.folders.length === 0 && library.songs.length === 0}
  <Empty />
{:else}
  <div bind:this={scroller} class="scroll h-full px-8 pb-10">
    <header class="flex items-end justify-between gap-6 pt-6 pb-6">
      <div>
        <h1 class="display text-[34px] leading-none font-semibold">{recent ? "Recently added" : "Songs"}</h1>
        <p class="mt-2 text-[13px] text-paper-3">{count(songs.length, "song", "songs")} · {length(total)}</p>
      </div>
      <div class="flex items-center gap-2">
        {#if !recent}
          <Select
            bind:value={order}
            label="Sort songs"
            class="w-40"
            options={[
              { value: "title", label: "By title" },
              { value: "artist", label: "By artist" },
              { value: "album", label: "By album" },
              { value: "added", label: "Newest first" },
              { value: "length", label: "By length" },
            ]}
          />
        {/if}
        <Button kind="primary" icon="play" disabled={!songs.length} onclick={() => player.play(songs.map((song) => song.id), 0, { type: "songs" })}>Play</Button>
      </div>
    </header>
    <SongList {songs} context={{ type: "songs" }} {scroller} />
  </div>
{/if}
