<script lang="ts">
  import { count } from "../../lib/format";
  import { library } from "../../lib/stores/library.svelte";
  import { ui } from "../../lib/stores/ui.svelte";
  import Cover from "../../components/media/Cover.svelte";
  import Empty from "./Empty.svelte";
  import Tabs from "./Tabs.svelte";
</script>

{#if library.ready && library.songs.length === 0}
  <Empty />
{:else}
  <div class="scroll h-full px-10 pb-32">
    <Tabs meta={count(library.artists.length, "artist", "artists")} />
    <div class="mx-auto grid max-w-[1180px] grid-cols-[repeat(auto-fit,148px)] justify-center gap-x-6 gap-y-8">
      {#each library.artists as artist (artist.name)}
        <button type="button" class="group flex flex-col items-center gap-3 text-center" onclick={() => ui.go({ view: "artist", name: artist.name })}>
          <Cover cover={artist.cover} label={artist.name} size={148} class="rounded-full! transition-transform duration-300 ease-out-soft group-hover:-translate-y-1" />
          <div class="w-full min-w-0">
            <div class="truncate text-[14px] font-medium">{artist.name}</div>
            <div class="truncate text-[12px] text-paper-3">{count(artist.songs.length, "song", "songs")}</div>
          </div>
        </button>
      {/each}
    </div>
  </div>
{/if}
