<script lang="ts">
  import { appWindow } from "@lantharos/sabine";
  import { accentFrom } from "./lib/accent";
  import { events } from "./lib/bindings";
  import { coverUrl } from "./lib/media";
  import { followOpenedFiles, open } from "./lib/opening";
  import { library } from "./lib/stores/library.svelte";
  import { player } from "./lib/stores/player.svelte";
  import { sound } from "./lib/stores/sound.svelte";
  import { ui } from "./lib/stores/ui.svelte";
  import CoverEditor from "./components/overlays/CoverEditor.svelte";
  import ImportDialog from "./components/overlays/ImportDialog.svelte";
  import Visualizer from "./components/overlays/Visualizer.svelte";
  import SearchPalette from "./components/overlays/SearchPalette.svelte";
  import Capsule from "./components/shell/Capsule.svelte";
  import QueueSheet from "./components/shell/QueueSheet.svelte";
  import TitleBar from "./components/shell/TitleBar.svelte";
  import Toasts from "./components/shell/Toasts.svelte";
  import Menu from "./components/ui/Menu.svelte";
  import Album from "./views/library/Album.svelte";
  import Albums from "./views/library/Albums.svelte";
  import Artist from "./views/library/Artist.svelte";
  import Artists from "./views/library/Artists.svelte";
  import SongEditor from "./views/editor/SongEditor.svelte";
  import Listen from "./views/Listen.svelte";
  import Playlist from "./views/playlist/Playlist.svelte";
  import Playlists from "./views/playlist/Playlists.svelte";
  import Songs from "./views/library/Songs.svelte";
  import Sound from "./views/Sound.svelte";

  const ready = Promise.all([
    library.start(),
    player.start(),
    sound.start(),
    appWindow.controlsOverlay().then(({ left, right }) => {
      document.documentElement.dataset.controls = left || right ? "system" : "page";
    }),
  ]);
  appWindow.revealWhen(ready);
  void ready.then(followOpenedFiles);
  events.appFailed(({ message }) => ui.toast(message));
  appWindow.onControlsOverlayChanged(({ left, right }) => {
    document.documentElement.dataset.controls = left || right ? "system" : "page";
  });

  $effect(() => {
    const url = coverUrl(player.song?.cover, true);
    const root = document.documentElement.style;
    if (!url) {
      root.removeProperty("--accent");
      root.removeProperty("--on-accent");
      return;
    }
    void accentFrom(url).then((colours) => {
      if (!colours) {
        root.removeProperty("--accent");
        root.removeProperty("--on-accent");
        return;
      }
      root.setProperty("--accent", colours.accent);
      root.setProperty("--on-accent", colours.onAccent);
    });
  });

  function typing(target: EventTarget | null) {
    return target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement;
  }

  function key(event: KeyboardEvent) {
    if (typing(event.target) || ui.coverTarget || ui.importing || ui.searching) return;
    if ((event.key === "k" && event.ctrlKey) || event.key === "/") {
      event.preventDefault();
      ui.searching = true;
    } else if (event.key === " ") {
      event.preventDefault();
      void player.toggle();
    } else if (event.key === "ArrowRight" && event.altKey) {
      void player.next();
    } else if (event.key === "ArrowLeft" && event.altKey) {
      void player.previous();
    } else if (event.key === "ArrowRight" || event.key === "ArrowLeft") {
      if (player.song) void player.seek(player.seconds() + (event.key === "ArrowRight" ? 5 : -5));
    }
  }

  function dropped(event: DragEvent) {
    const files = [...(event.dataTransfer?.files ?? [])] as (File & { path?: string })[];
    const paths = files.map((file) => file.path).filter((path): path is string => !!path);
    if (!paths.length || ui.coverTarget) return;
    event.preventDefault();
    open(paths);
  }
</script>

<svelte:window onkeydown={key} ondragover={(event) => event.preventDefault()} ondrop={dropped} />

<div class="flex h-full flex-col">
  <TitleBar />
  <main class="relative min-h-0 flex-1 overflow-hidden">
    {#key JSON.stringify(ui.route)}
      {#if ui.route.view === "listen"}<Listen />
      {:else if ui.route.view === "songs"}<Songs />
      {:else if ui.route.view === "recent"}<Songs recent />
      {:else if ui.route.view === "albums"}<Albums />
      {:else if ui.route.view === "album"}<Album key={ui.route.key} />
      {:else if ui.route.view === "artists"}<Artists />
      {:else if ui.route.view === "artist"}<Artist name={ui.route.name} />
      {:else if ui.route.view === "playlists"}<Playlists />
      {:else if ui.route.view === "playlist"}<Playlist id={ui.route.id} />
      {:else if ui.route.view === "sound"}<Sound />
      {:else if ui.route.view === "edit"}<SongEditor song={ui.route.song} next={ui.route.next} context={ui.route.context} />
      {/if}
    {/key}
  </main>
</div>

{#if ui.route.view !== "listen"}<Capsule />{/if}
{#if ui.queueOpen}<QueueSheet />{/if}
{#if ui.searching}<SearchPalette />{/if}
{#if ui.visualizer}<Visualizer />{/if}
{#if ui.coverTarget}<CoverEditor target={ui.coverTarget} />{/if}
{#if ui.importing}<ImportDialog path={ui.importing} />{/if}
<Menu />
<Toasts />
