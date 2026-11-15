<script lang="ts">
  import { count } from "../../lib/format";
  import { library } from "../../lib/stores/library.svelte";
  import { player } from "../../lib/stores/player.svelte";
  import AlbumGrid from "../../components/media/AlbumGrid.svelte";
  import Hero from "../../components/media/Hero.svelte";
  import SongList from "../../components/media/SongList.svelte";
  import Button from "../../components/ui/Button.svelte";

  let { name }: { name: string } = $props();
  let scroller = $state<HTMLElement>();
  const artist = $derived(library.artists.find((entry) => entry.name.toLowerCase() === name.toLowerCase()));
</script>

{#if artist}
  <div bind:this={scroller} class="scroll h-full px-10 pb-32">
    <Hero cover={artist.cover} label={artist.name} round meta="{count(artist.songs.length, 'song', 'songs')}{artist.albums.length ? ` · ${count(artist.albums.length, 'album', 'albums')}` : ''}">
      {#snippet actions()}
        <Button kind="primary" icon="play" onclick={() => player.play(artist.songs.map((song) => song.id), 0, { type: "songs" })}>Play</Button>
      {/snippet}
    </Hero>
    {#if artist.albums.length > 1}
      <div class="mx-auto mb-12 max-w-[1180px]"><AlbumGrid albums={artist.albums} /></div>
    {/if}
    <div class="mx-auto max-w-[880px]">
      <SongList songs={artist.songs} context={{ type: "songs" }} showAlbum={artist.albums.length > 1} {scroller} />
    </div>
  </div>
{/if}
