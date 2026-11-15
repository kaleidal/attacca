<script lang="ts">
  import { matches } from "../../lib/format";
  import { library } from "../../lib/stores/library.svelte";
  import { ui } from "../../lib/stores/ui.svelte";
  import AlbumGrid from "../../components/media/AlbumGrid.svelte";
  import Cover from "../../components/media/Cover.svelte";
  import SongList from "../../components/media/SongList.svelte";

  let scroller = $state<HTMLElement>();
  const query = $derived(ui.search.trim().toLowerCase());
  const songs = $derived(
    query ? library.songs.filter((song) => matches(song.title ?? song.name, query) || matches(song.artist, query) || matches(song.album, query) || matches(song.genre, query)).slice(0, 200) : [],
  );
  const albums = $derived(query ? library.albums.filter((album) => matches(album.title, query) || matches(album.artist, query)).slice(0, 12) : []);
  const artists = $derived(query ? library.artists.filter((artist) => matches(artist.name, query)).slice(0, 8) : []);
  const playlists = $derived(query ? library.playlists.filter((playlist) => matches(playlist.name, query)) : []);
</script>

<div bind:this={scroller} class="scroll h-full px-8 pb-10">
  <header class="pt-6 pb-6">
    <h1 class="display truncate text-[34px] leading-none font-semibold">{ui.search.trim() || "Search"}</h1>
  </header>
  {#if query && !songs.length && !albums.length && !artists.length && !playlists.length}
    <p class="text-[14px] text-paper-3">Nothing matches.</p>
  {/if}
  {#if artists.length || playlists.length}
    <div class="mb-8 flex flex-wrap gap-2">
      {#each artists as artist (artist.name)}
        <button type="button" class="flex items-center gap-2.5 rounded-full bg-ink-2 py-1 pr-4 pl-1 text-[13px] hover:bg-ink-3" onclick={() => ui.go({ view: "artist", name: artist.name })}>
          <Cover cover={artist.cover} label={artist.name} size={30} class="rounded-full!" />{artist.name}
        </button>
      {/each}
      {#each playlists as playlist (playlist.id)}
        <button type="button" class="flex items-center gap-2.5 rounded-full bg-ink-2 py-1 pr-4 pl-1 text-[13px] hover:bg-ink-3" onclick={() => ui.go({ view: "playlist", id: playlist.id })}>
          <Cover cover={playlist.cover ?? playlist.covers[0]} label={playlist.name} size={30} class="rounded-full!" />{playlist.name}
        </button>
      {/each}
    </div>
  {/if}
  {#if albums.length}
    <h2 class="display mb-4 text-xl font-semibold">Albums</h2>
    <div class="mb-10"><AlbumGrid {albums} /></div>
  {/if}
  {#if songs.length}
    <h2 class="display mb-3 text-xl font-semibold">Songs</h2>
    <SongList {songs} context={{ type: "songs" }} {scroller} />
  {/if}
</div>
