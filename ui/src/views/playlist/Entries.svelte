<script lang="ts">
  import { commands, type Context, type Playlist, type Song } from "../../lib/bindings";
  import { songMenu } from "../../lib/actions";
  import { artist, clock, title } from "../../lib/format";
  import { library } from "../../lib/stores/library.svelte";
  import { player } from "../../lib/stores/player.svelte";
  import { ui } from "../../lib/stores/ui.svelte";
  import Cover from "../../components/media/Cover.svelte";
  import Icon from "../../components/ui/Icon.svelte";

  let { playlist, context, songAfter, onplay }: {
    playlist: Playlist;
    context: Context;
    songAfter: (song: Song) => Song | null;
    onplay: (song: Song) => void;
  } = $props();

  let dragging = $state<number | null>(null);
  let over = $state<number | null>(null);

  function drop(target: number) {
    if (dragging === null || dragging === target) return finish();
    const order = playlist.entries.map((entry) => entry.id);
    const [moved] = order.splice(dragging, 1);
    order.splice(target, 0, moved!);
    void commands.playlistsOrder({ id: playlist.id, entries: order });
    finish();
  }

  function finish() {
    dragging = null;
    over = null;
  }

  function removeEntry(id: number) {
    void commands.playlistsOrder({ id: playlist.id, entries: playlist.entries.map((entry) => entry.id).filter((entry) => entry !== id) });
  }
</script>

<div role="list">
  {#each playlist.entries as entry, index (entry.id)}
    {@const song = entry.song ? library.byId.get(entry.song) : undefined}
    <div
      role="listitem"
      draggable="true"
      class="group grid h-[52px] grid-cols-[28px_minmax(0,2fr)_minmax(0,1.3fr)_56px] items-center gap-4 rounded-lg px-3 hover:bg-ink-2 {over === index ? 'shadow-[inset_0_2px_0_var(--accent)]' : ''} {dragging === index ? 'opacity-40' : ''}"
      ondragstart={() => (dragging = index)}
      ondragover={(event) => (event.preventDefault(), (over = index))}
      ondragend={finish}
      ondrop={() => drop(index)}
      ondblclick={() => song && onplay(song)}
      oncontextmenu={(event) =>
        ui.openMenu(
          event,
          song
            ? songMenu([song], context, songAfter(song), [{ label: "Remove from playlist", action: () => removeEntry(entry.id), danger: true }])
            : [{ label: "Remove from playlist", action: () => removeEntry(entry.id), danger: true }],
        )}
    >
      <div class="flex justify-center text-paper-4">
        <span class="numeric text-[12px] group-hover:hidden">{index + 1}</span>
        <Icon name="grip" size={14} class="hidden group-hover:block" />
      </div>
      {#if song}
        <div class="flex min-w-0 items-center gap-3">
          <Cover cover={song.cover} label={title(song)} size={36} />
          <div class="min-w-0">
            <div class="truncate text-[14px] {player.song?.id === song.id ? 'text-accent' : ''}">{title(song)}</div>
            <div class="truncate text-[13px] text-paper-3">{artist(song)}</div>
          </div>
        </div>
        <div class="truncate text-[13px] text-paper-3">{song.album ?? song.folder}</div>
        <div class="numeric text-right text-[12px] text-paper-3">{clock(song.duration)}</div>
      {:else if entry.missing}
        <div class="flex min-w-0 items-center gap-3 text-paper-4">
          <div class="size-9 shrink-0 rounded-(--radius-cover) border border-dashed border-line-strong"></div>
          <div class="min-w-0">
            <div class="truncate text-[14px]">{entry.missing.title}</div>
            <div class="truncate text-[13px] italic">Not in your library yet</div>
          </div>
        </div>
        <div class="truncate text-[13px] text-paper-4">{entry.missing.artist ?? ""}</div>
        <div class="numeric text-right text-[12px] text-paper-4">{clock(entry.missing.duration)}</div>
      {/if}
    </div>
  {:else}
    <p class="py-10 text-center text-[13px] text-paper-4">Add songs from their menu, with a right click.</p>
  {/each}
</div>
