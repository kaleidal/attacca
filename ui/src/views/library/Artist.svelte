<script lang="ts">
  import { count } from "../../lib/format";
  import { library } from "../../lib/stores/library.svelte";
  import { player } from "../../lib/stores/player.svelte";
  import AlbumGrid from "../../components/media/AlbumGrid.svelte";
  import SongList from "../../components/media/SongList.svelte";
  import Button from "../../components/ui/Button.svelte";

  let { name }: { name: string } = $props();
  let scroller = $state<HTMLElement>();
  const artist = $derived(library.artists.find((entry) => entry.name.toLowerCase() === name.toLowerCase()));
</script>

{#if artist}
  <div bind:this={scroller} class="scroll h-full px-8 pb-10">
    <header class="flex items-end justify-between gap-6 pt-6 pb-8">
      <div>
        <h1 class="display text-[40px] leading-none font-semibold">{artist.name}</h1>
        <p class="mt-2 text-[13px] text-paper-3">{count(artist.songs.length, "song", "songs")}</p>
      </div>
      <Button kind="primary" icon="play" onclick={() => player.play(artist.songs.map((song) => song.id), 0, { type: "songs" })}>Play</Button>
    </header>
    {#if artist.albums.length}
      <h2 class="display mb-4 text-xl font-semibold">Albums</h2>
      <div class="mb-10"><AlbumGrid albums={artist.albums} /></div>
    {/if}
    <h2 class="display mb-3 text-xl font-semibold">Songs</h2>
    <SongList songs={artist.songs} context={{ type: "songs" }} {scroller} />
  </div>
{/if}
