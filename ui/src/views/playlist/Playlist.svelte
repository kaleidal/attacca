<script lang="ts">
  import { commands, type Playlist, type Song, type Transition } from "../../lib/bindings";
  import { count, length } from "../../lib/format";
  import { library } from "../../lib/stores/library.svelte";
  import { player } from "../../lib/stores/player.svelte";
  import { ui } from "../../lib/stores/ui.svelte";
  import { fade, info } from "../../lib/transitions";
  import Hero from "../../components/media/Hero.svelte";
  import TransitionGlyph from "../../components/media/TransitionGlyph.svelte";
  import TransitionPicker from "../../components/media/TransitionPicker.svelte";
  import Button from "../../components/ui/Button.svelte";
  import Popover from "../../components/ui/Popover.svelte";
  import Entries from "./Entries.svelte";
  import TransitionStrip from "./TransitionStrip.svelte";

  let { id }: { id: number } = $props();
  let playlist = $state<Playlist | null>(null);
  let naming = $state(false);
  let name = $state("");

  async function load() {
    playlist = await commands.playlistsGet({ id });
  }

  $effect(() => {
    void library.playlists;
    void library.songs;
    void load();
  });

  const songs = $derived(
    (playlist?.entries ?? []).flatMap((entry) => {
      const song = entry.song ? library.byId.get(entry.song) : undefined;
      return song ? [song] : [];
    }),
  );
  const missing = $derived((playlist?.entries ?? []).filter((entry) => !entry.song).length);
  const total = $derived(songs.reduce((sum, song) => sum + song.duration, 0));
  const fallback = $derived(playlist?.transition ?? fade);
  const context = $derived({ type: "playlist" as const, id });

  function play(index = 0) {
    void player.play(songs.map((song) => song.id), index, context);
  }

  async function rename() {
    naming = false;
    const next = name.trim();
    if (playlist && next && next !== playlist.name) await commands.playlistsRename({ id, name: next });
  }

  function remove() {
    void commands.playlistsDelete({ id }).then(() => ui.go({ view: "playlists" }));
  }

  async function share() {
    const path = await commands.playlistsShare({ id });
    if (path) ui.toast(`Saved ${path.slice(path.lastIndexOf("/") + 1)}`);
  }

  function setDefault(transition: Transition | null) {
    void commands.playlistsSetTransition({ id, transition });
  }

  function songAfter(song: Song) {
    const index = songs.findIndex((entry) => entry.id === song.id);
    return songs[index + 1] ?? null;
  }
</script>

{#if playlist}
  <div class="scroll h-full px-10 pb-32">
    <Hero cover={playlist.cover ?? songs[0]?.cover} label={playlist.name} oncover={() => (ui.coverTarget = { type: "playlist", id })} meta="{count(songs.length, 'song', 'songs')} · {length(total)}{missing ? ` · ${missing} not in your library yet` : ''}">
      {#snippet title()}
        {#if naming}
          <!-- svelte-ignore a11y_autofocus -->
          <input
            bind:value={name}
            aria-label="Playlist name"
            autofocus
            class="display w-full bg-transparent text-center text-[42px] leading-[1.05] font-semibold outline-none"
            onblur={rename}
            onkeydown={(event) => (event.key === "Enter" ? rename() : event.key === "Escape" && (naming = false))}
          />
        {:else}
          <button type="button" class="display max-w-full truncate text-[42px] leading-[1.05] font-semibold" onclick={() => ((name = playlist!.name), (naming = true))}>
            {playlist?.name}
          </button>
        {/if}
      {/snippet}
      {#snippet actions()}
        <Button kind="primary" icon="play" disabled={!songs.length} onclick={() => play()}>Play</Button>
        <Popover width={340}>
          {#snippet trigger(toggle)}
            <Button onclick={toggle}>
              <TransitionGlyph kind={fallback.kind} width={30} />
              {info(fallback.kind).label}{fallback.kind !== "none" ? ` · ${fallback.seconds.toFixed(fallback.seconds < 2 ? 1 : 0)} s` : ""}
            </Button>
          {/snippet}
          {#snippet children()}
            <div class="mb-3 text-[13px] text-paper-2">Between songs in this playlist</div>
            <TransitionPicker value={playlist!.transition} fallback={{ label: "Fade, as always", transition: fade }} onchange={setDefault} />
          {/snippet}
        </Popover>
        <Button icon="share" label="Share playlist" onclick={share} />
        <Button
          icon="more"
          label="More"
          onclick={(event) =>
            ui.openMenu(event, [
              { label: "Rename", action: () => ((name = playlist!.name), (naming = true)) },
              { label: "Change cover", action: () => (ui.coverTarget = { type: "playlist", id }) },
              { label: "Delete playlist", action: remove, danger: true },
            ])}
        />
      {/snippet}
    </Hero>

    {#if songs.length > 1}
      <TransitionStrip {playlist} {songs} {fallback} />
    {/if}

    <div class="mx-auto max-w-[880px]">
      <Entries {playlist} {context} {songAfter} onplay={(song) => play(songs.findIndex((entry) => entry.id === song.id))} />
    </div>
  </div>
{/if}
