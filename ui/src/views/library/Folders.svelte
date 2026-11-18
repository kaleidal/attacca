<script lang="ts">
  import type { Folder } from "../../lib/folders";
  import { count, length } from "../../lib/format";
  import { library } from "../../lib/stores/library.svelte";
  import { player } from "../../lib/stores/player.svelte";
  import { ui } from "../../lib/stores/ui.svelte";
  import FolderGrid from "../../components/media/FolderGrid.svelte";
  import SongList from "../../components/media/SongList.svelte";
  import Button from "../../components/ui/Button.svelte";
  import Empty from "./Empty.svelte";
  import Tabs from "./Tabs.svelte";

  let { path }: { path: string | null } = $props();
  let scroller = $state<HTMLElement>();

  const roots = $derived(library.tree.roots);
  const folder = $derived(path ? library.tree.byPath.get(path) : roots.length === 1 ? roots[0] : undefined);
  const shown = $derived(folder?.folders ?? roots);
  const songs = $derived(folder?.all ?? library.songs);

  /** The folders from the watched one down to this one. */
  const trail = $derived.by(() => {
    const steps: Folder[] = [];
    for (let at = folder; at; at = roots.includes(at) ? undefined : library.tree.byPath.get(at.path.slice(0, at.path.lastIndexOf("/")))) steps.unshift(at);
    return steps;
  });
  const meta = $derived(
    folder ? `${count(songs.length, "song", "songs")} · ${length(songs.reduce((sum, song) => sum + song.duration, 0))}` : count(roots.length, "folder", "folders"),
  );
</script>

{#if library.ready && library.songs.length === 0}
  <Empty />
{:else}
  <div bind:this={scroller} class="scroll h-full px-10 pb-32">
    <Tabs {meta}>
      {#if folder}
        <Button kind="primary" icon="play" disabled={!songs.length} onclick={() => player.play(songs.map((song) => song.id), 0, { type: "songs" })}>Play</Button>
      {/if}
    </Tabs>
    {#if trail.length > (roots.length === 1 ? 1 : 0)}
      <nav aria-label="Folder" class="mx-auto -mt-3 mb-8 flex max-w-[1040px] flex-wrap items-center justify-center gap-1 text-[13px]">
        {#if roots.length > 1}
          <button type="button" class="rounded-md px-1.5 py-0.5 text-paper-3 hover:text-paper" onclick={() => ui.go({ view: "folders", path: null })}>All folders</button>
        {/if}
        {#each trail as step, index (step.path)}
          {#if index > 0 || roots.length > 1}<span class="text-paper-4">/</span>{/if}
          <button
            type="button"
            class="max-w-56 truncate rounded-md px-1.5 py-0.5 {index === trail.length - 1 ? 'text-paper' : 'text-paper-3 hover:text-paper'}"
            onclick={() => ui.go({ view: "folders", path: step.path })}
          >
            {step.name}
          </button>
        {/each}
      </nav>
    {/if}
    {#if shown.length}
      <div class="mx-auto mb-12 max-w-[1180px]"><FolderGrid folders={shown} /></div>
    {/if}
    {#if folder?.songs.length}
      <div class="mx-auto max-w-[1040px]">
        <SongList songs={folder.songs} context={{ type: "songs" }} {scroller} />
      </div>
    {/if}
  </div>
{/if}
