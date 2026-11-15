<script lang="ts">
  import { commands } from "../../lib/bindings";
  import { count, length } from "../../lib/format";
  import { library } from "../../lib/stores/library.svelte";
  import { player } from "../../lib/stores/player.svelte";
  import { ui } from "../../lib/stores/ui.svelte";
  import Hero from "../../components/media/Hero.svelte";
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
  <div bind:this={scroller} class="scroll h-full px-10 pb-32">
    <Hero cover={album.cover} label={album.title} oncover={() => (ui.coverTarget = { type: "album", key })} meta={[album.year, count(album.songs.length, "song", "songs"), length(album.duration)].filter(Boolean).join(" · ")}>
      {#snippet subtitle()}
        <button type="button" class="hover:text-paper" onclick={() => ui.go({ view: "artist", name: album.artist })}>{album.artist}</button>
      {/snippet}
      {#snippet actions()}
        <Button kind="primary" icon="play" onclick={() => player.play(album.songs.map((song) => song.id), 0, { type: "album", key })}>Play</Button>
        <label class="flex h-9 items-center gap-3 rounded-lg bg-ink-3 pr-2 pl-3.5 text-[13px] text-paper-2">
          {transitions ? "Transitions between tracks" : "Gapless"}
          <Toggle bind:checked={transitions} label="Transitions inside this album" onchange={(on) => commands.albumsSetSettings({ key, settings: { transitions: on } })} />
        </label>
      {/snippet}
    </Hero>
    <div class="mx-auto max-w-[880px]">
      <SongList songs={album.songs} context={{ type: "album", key }} numbered showAlbum={false} {scroller} />
    </div>
  </div>
{/if}
