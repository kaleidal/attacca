<script lang="ts">
  import { count } from "../../lib/format";
  import { library } from "../../lib/stores/library.svelte";
  import AlbumGrid from "../../components/media/AlbumGrid.svelte";
  import Empty from "./Empty.svelte";
</script>

{#if library.ready && library.songs.length === 0}
  <Empty />
{:else}
  <div class="scroll h-full px-8 pb-10">
    <header class="pt-6 pb-6">
      <h1 class="display text-[34px] leading-none font-semibold">Albums</h1>
      <p class="mt-2 text-[13px] text-paper-3">{count(library.albums.length, "album", "albums")}</p>
    </header>
    <AlbumGrid albums={library.albums} />
  </div>
{/if}
