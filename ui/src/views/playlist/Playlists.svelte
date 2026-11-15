<script lang="ts">
  import { commands } from "../../lib/bindings";
  import { count } from "../../lib/format";
  import { library } from "../../lib/stores/library.svelte";
  import { ui } from "../../lib/stores/ui.svelte";
  import Cover from "../../components/media/Cover.svelte";
  import Icon from "../../components/ui/Icon.svelte";

  async function create() {
    const id = await commands.playlistsCreate({ name: "New playlist", songs: [] });
    ui.go({ view: "playlist", id });
  }
</script>

<div class="scroll h-full px-10 pb-32">
  <header class="flex flex-col items-center gap-3 pt-7 pb-8">
    <h1 class="display text-[32px] leading-none font-semibold">Playlists</h1>
    <p class="text-[13px] text-paper-3">{count(library.playlists.length, "playlist", "playlists")}</p>
  </header>
  <div class="mx-auto grid max-w-[1180px] grid-cols-[repeat(auto-fit,176px)] justify-center gap-x-6 gap-y-8">
    <button type="button" class="group flex flex-col gap-2.5 text-left" onclick={create}>
      <div class="flex aspect-square w-full items-center justify-center rounded-2xl bg-ink-2 text-paper-3 transition-colors group-hover:bg-ink-3 group-hover:text-paper">
        <Icon name="plus" size={28} />
      </div>
      <div class="text-center text-[14px] font-medium text-paper-2">New playlist</div>
    </button>
    {#each library.playlists as playlist (playlist.id)}
      <button type="button" class="group flex min-w-0 flex-col gap-2.5 text-left" onclick={() => ui.go({ view: "playlist", id: playlist.id })}>
        {#if playlist.cover || playlist.covers.length < 4}
          <Cover cover={playlist.cover ?? playlist.covers[0]} label={playlist.name} size="fill" large />
        {:else}
          <div class="grid aspect-square w-full grid-cols-2 overflow-hidden rounded-(--radius-cover)">
            {#each playlist.covers.slice(0, 4) as cover (cover)}
              <Cover {cover} label={playlist.name} size="fill" class="rounded-none!" />
            {/each}
          </div>
        {/if}
        <div class="min-w-0">
          <div class="truncate text-[14px] font-medium">{playlist.name}</div>
          <div class="text-[13px] text-paper-3">{count(playlist.count, "song", "songs")}</div>
        </div>
      </button>
    {/each}
  </div>
</div>
