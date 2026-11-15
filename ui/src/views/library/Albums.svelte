<script lang="ts">
  import { count } from "../../lib/format";
  import { library } from "../../lib/stores/library.svelte";
  import AlbumGrid from "../../components/media/AlbumGrid.svelte";
  import Empty from "./Empty.svelte";
  import Tabs from "./Tabs.svelte";
</script>

{#if library.ready && library.songs.length === 0}
  <Empty />
{:else}
  <div class="scroll h-full px-10 pb-32">
    <Tabs meta={count(library.albums.length, "album", "albums")} />
    <div class="mx-auto max-w-[1180px]"><AlbumGrid albums={library.albums} /></div>
  </div>
{/if}
