<script lang="ts">
  import { count } from "../../lib/format";
  import { library } from "../../lib/stores/library.svelte";
  import { ui } from "../../lib/stores/ui.svelte";
  import Cover from "../../components/media/Cover.svelte";
  import Empty from "./Empty.svelte";
</script>

{#if library.ready && library.songs.length === 0}
  <Empty />
{:else}
  <div class="scroll h-full px-8 pb-10">
    <header class="pt-6 pb-6">
      <h1 class="display text-[34px] leading-none font-semibold">Artists</h1>
      <p class="mt-2 text-[13px] text-paper-3">{count(library.artists.length, "artist", "artists")}</p>
    </header>
    <div class="grid grid-cols-[repeat(auto-fill,minmax(260px,1fr))] gap-1">
      {#each library.artists as artist (artist.name)}
        <button type="button" class="flex items-center gap-3.5 rounded-xl p-2 text-left hover:bg-ink-2" onclick={() => ui.go({ view: "artist", name: artist.name })}>
          <Cover cover={artist.cover} label={artist.name} size={52} class="rounded-full!" />
          <div class="min-w-0">
            <div class="truncate text-[14px] font-medium">{artist.name}</div>
            <div class="truncate text-[13px] text-paper-3">{count(artist.songs.length, "song", "songs")}{artist.albums.length ? ` · ${count(artist.albums.length, "album", "albums")}` : ""}</div>
          </div>
        </button>
      {/each}
    </div>
  </div>
{/if}
