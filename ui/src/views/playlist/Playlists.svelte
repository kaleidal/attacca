<script lang="ts">
  import { cubicOut } from "svelte/easing";
  import { fly } from "svelte/transition";
  import { commands, type PlaylistSummary } from "../../lib/bindings";
  import { count } from "../../lib/format";
  import { motion } from "../../lib/motion";
  import { library } from "../../lib/stores/library.svelte";
  import { player } from "../../lib/stores/player.svelte";
  import { ui } from "../../lib/stores/ui.svelte";
  import Cover from "../../components/media/Cover.svelte";
  import Icon from "../../components/ui/Icon.svelte";

  async function create() {
    const id = await commands.playlistsCreate({ name: "New playlist", songs: [] });
    ui.go({ view: "playlist", id });
  }

  async function play(event: MouseEvent, summary: PlaylistSummary) {
    event.stopPropagation();
    const playlist = await commands.playlistsGet({ id: summary.id });
    const songs = playlist?.entries.flatMap((entry) => (entry.song ? [entry.song] : [])) ?? [];
    if (songs.length) void player.play(songs, 0, { type: "playlist", id: summary.id });
  }
</script>

<div class="scroll h-full px-10 pb-32">
  <header class="flex flex-col items-center gap-3 pt-7 pb-8">
    <h1 class="display text-[32px] leading-none font-semibold">Playlists</h1>
    <p class="text-[13px] text-paper-3">{count(library.playlists.length, "playlist", "playlists")}</p>
  </header>
  <div class="mx-auto grid max-w-[1180px] grid-cols-[repeat(auto-fit,176px)] justify-center gap-x-6 gap-y-8">
    <button type="button" class="group flex flex-col gap-2.5 text-left" onclick={create}>
      <div
        class="flex aspect-square w-full items-center justify-center rounded-2xl border border-dashed border-line-strong text-paper-3 transition duration-300 ease-out-soft group-hover:-translate-y-1 group-hover:border-paper-4 group-hover:bg-ink-2 group-hover:text-paper group-active:scale-[0.98]"
      >
        <Icon name="plus" size={28} class="transition-transform duration-300 ease-out-soft group-hover:rotate-90" />
      </div>
      <div class="text-center text-[14px] font-medium text-paper-2 transition-colors group-hover:text-paper">New playlist</div>
    </button>
    {#each library.playlists as playlist, index (playlist.id)}
      <div
        role="button"
        tabindex="0"
        in:fly={{ y: 14, duration: motion(320), delay: motion(Math.min(index, 18) * 22), easing: cubicOut }}
        class="group flex min-w-0 flex-col gap-2.5 text-left"
        onclick={() => ui.go({ view: "playlist", id: playlist.id })}
        onkeydown={(event) => event.key === "Enter" && ui.go({ view: "playlist", id: playlist.id })}
      >
        <div class="relative transition-transform duration-300 ease-out-soft group-hover:-translate-y-1 group-active:scale-[0.98]">
          {#if playlist.cover || playlist.covers.length < 4}
            <Cover cover={playlist.cover ?? playlist.covers[0]} label={playlist.name} size="fill" large class="rounded-2xl! transition-shadow duration-300 group-hover:shadow-[0_18px_44px_rgb(0_0_0/0.5)]" />
          {:else}
            <div class="grid aspect-square w-full grid-cols-2 overflow-hidden rounded-2xl transition-shadow duration-300 group-hover:shadow-[0_18px_44px_rgb(0_0_0/0.5)]">
              {#each playlist.covers.slice(0, 4) as cover (cover)}
                <Cover {cover} label={playlist.name} size="fill" class="rounded-none!" />
              {/each}
            </div>
          {/if}
          {#if playlist.count}
            <button
              type="button"
              aria-label="Play {playlist.name}"
              class="absolute right-2.5 bottom-2.5 flex size-10 translate-y-1 items-center justify-center rounded-full bg-accent text-on-accent opacity-0 shadow-[0_6px_20px_rgb(0_0_0/0.45)] transition duration-200 group-hover:translate-y-0 group-hover:opacity-100 hover:scale-105 active:scale-90"
              onclick={(event) => play(event, playlist)}
            >
              <Icon name="play" size={18} />
            </button>
          {/if}
        </div>
        <div class="min-w-0">
          <div class="truncate text-[14px] font-medium">{playlist.name}</div>
          <div class="text-[13px] text-paper-3">{count(playlist.count, "song", "songs")}</div>
        </div>
      </div>
    {/each}
  </div>
</div>
