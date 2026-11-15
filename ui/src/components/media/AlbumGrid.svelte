<script lang="ts">
  import type { Album } from "../../lib/stores/library.svelte";
  import { ui } from "../../lib/stores/ui.svelte";
  import { player } from "../../lib/stores/player.svelte";
  import Cover from "./Cover.svelte";
  import Icon from "../ui/Icon.svelte";

  let { albums }: { albums: Album[] } = $props();

  function play(album: Album, event: MouseEvent) {
    event.stopPropagation();
    void player.play(album.songs.map((song) => song.id), 0, { type: "album", key: album.key });
  }
</script>

<div class="grid grid-cols-[repeat(auto-fill,minmax(168px,1fr))] gap-x-5 gap-y-7">
  {#each albums as album (album.key)}
    <div
      role="button"
      tabindex="0"
      class="group flex min-w-0 flex-col gap-2.5 text-left"
      onclick={() => ui.go({ view: "album", key: album.key })}
      onkeydown={(event) => event.key === "Enter" && ui.go({ view: "album", key: album.key })}
    >
      <div class="relative">
        <Cover cover={album.cover} label={album.title} size="fill" large class="transition-transform duration-300 ease-out-soft group-hover:-translate-y-0.5" />
        <button
          type="button"
          aria-label="Play {album.title}"
          class="absolute right-2.5 bottom-2.5 flex size-10 items-center justify-center rounded-full bg-accent text-on-accent opacity-0 shadow-[0_6px_20px_rgb(0_0_0/0.45)] transition group-hover:opacity-100"
          onclick={(event) => play(album, event)}
        >
          <Icon name="play" size={18} />
        </button>
      </div>
      <div class="min-w-0">
        <div class="truncate text-[14px] font-medium">{album.title}</div>
        <div class="truncate text-[13px] text-paper-3">{album.artist}{album.year ? ` · ${album.year}` : ""}</div>
      </div>
    </div>
  {/each}
</div>
