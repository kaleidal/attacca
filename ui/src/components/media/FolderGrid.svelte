<script lang="ts">
  import { cubicOut } from "svelte/easing";
  import { fly } from "svelte/transition";
  import type { Folder } from "../../lib/folders";
  import { count } from "../../lib/format";
  import { motion } from "../../lib/motion";
  import { player } from "../../lib/stores/player.svelte";
  import { ui } from "../../lib/stores/ui.svelte";
  import Icon from "../ui/Icon.svelte";
  import Cover from "./Cover.svelte";

  let { folders }: { folders: Folder[] } = $props();

  function play(folder: Folder, event: MouseEvent) {
    event.stopPropagation();
    void player.play(folder.all.map((song) => song.id), 0, { type: "songs" });
  }

  const open = (folder: Folder) => ui.go({ view: "folders", path: folder.path });
</script>

<div class="grid grid-cols-[repeat(auto-fit,152px)] justify-center gap-x-6 gap-y-7">
  {#each folders as folder, index (folder.path)}
    <div
      in:fly={{ y: 14, duration: motion(320), delay: motion(Math.min(index, 18) * 22), easing: cubicOut }}
      role="button"
      tabindex="0"
      class="group flex min-w-0 flex-col gap-2.5 text-left"
      onclick={() => open(folder)}
      onkeydown={(event) => event.key === "Enter" && open(folder)}
    >
      <div class="relative">
        {#if folder.cover}
          <Cover cover={folder.cover} label={folder.name} size="fill" large class="rounded-2xl! transition duration-300 ease-out-soft group-hover:-translate-y-1 group-active:scale-[0.98]" />
        {:else}
          <div class="flex aspect-square w-full items-center justify-center rounded-2xl bg-ink-2 text-paper-4 transition duration-300 ease-out-soft group-hover:-translate-y-1 group-active:scale-[0.98]">
            <Icon name="folder" size={36} />
          </div>
        {/if}
        <button
          type="button"
          aria-label="Play {folder.name}"
          class="absolute right-2.5 bottom-2.5 flex size-10 translate-y-1 items-center justify-center rounded-full bg-accent text-on-accent opacity-0 shadow-[0_6px_20px_rgb(0_0_0/0.45)] transition duration-200 group-hover:translate-y-0 group-hover:opacity-100 active:scale-90"
          onclick={(event) => play(folder, event)}
        >
          <Icon name="play" size={18} />
        </button>
      </div>
      <div class="min-w-0">
        <div class="truncate text-[14px] font-medium">{folder.name}</div>
        <div class="truncate text-[13px] text-paper-3">{count(folder.all.length, "song", "songs")}</div>
      </div>
    </div>
  {/each}
</div>
