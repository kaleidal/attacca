<script lang="ts">
  import { commands } from "../../lib/bindings";
  import { count, length } from "../../lib/format";
  import { library } from "../../lib/stores/library.svelte";
  import { player } from "../../lib/stores/player.svelte";
  import { ui } from "../../lib/stores/ui.svelte";
  import Cover from "../../components/media/Cover.svelte";
  import SongList from "../../components/media/SongList.svelte";
  import Button from "../../components/ui/Button.svelte";
  import Toggle from "../../components/ui/Toggle.svelte";

  let { key }: { key: string } = $props();
  let scroller = $state<HTMLElement>();
  let transitions = $state(false);

  const album = $derived(library.albumsByKey.get(key));

  $effect(() => {
    void commands.albumsSettings({ key }).then((settings) => (transitions = settings.transitions));
  });
</script>

{#if album}
  <div bind:this={scroller} class="scroll h-full px-8 pb-10">
    <header class="flex items-end gap-7 pt-6 pb-8">
      <button type="button" aria-label="Change album cover" class="shrink-0" onclick={() => (ui.coverTarget = { type: "album", key })}>
        <Cover cover={album.cover} label={album.title} size={208} large />
      </button>
      <div class="min-w-0 flex-1 pb-1">
        <h1 class="display truncate text-[40px] leading-[1.05] font-semibold">{album.title}</h1>
        <button type="button" class="mt-2 text-[15px] text-paper-2 hover:text-paper" onclick={() => ui.go({ view: "artist", name: album.artist })}>{album.artist}</button>
        <p class="mt-1 text-[13px] text-paper-3">{[album.year, count(album.songs.length, "song", "songs"), length(album.duration)].filter(Boolean).join(" · ")}</p>
        <div class="mt-5 flex items-center gap-5">
          <Button kind="primary" icon="play" onclick={() => player.play(album.songs.map((song) => song.id), 0, { type: "album", key })}>Play</Button>
          <label class="flex items-center gap-3 text-[13px] text-paper-2">
            <Toggle bind:checked={transitions} label="Transitions inside this album" onchange={(on) => commands.albumsSetSettings({ key, settings: { transitions: on } })} />
            {transitions ? "Transitions between tracks" : "Gapless"}
          </label>
        </div>
      </div>
    </header>
    <SongList songs={album.songs} context={{ type: "album", key }} numbered showAlbum={false} {scroller} />
  </div>
{/if}
