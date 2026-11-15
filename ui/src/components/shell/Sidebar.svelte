<script lang="ts">
  import { commands } from "../../lib/bindings";
  import { library } from "../../lib/stores/library.svelte";
  import { ui, type Route } from "../../lib/stores/ui.svelte";
  import Cover from "../media/Cover.svelte";
  import Icon, { type IconName } from "../ui/Icon.svelte";

  const places: { route: Route; label: string; icon: IconName }[] = [
    { route: { view: "songs" }, label: "Songs", icon: "songs" },
    { route: { view: "albums" }, label: "Albums", icon: "albums" },
    { route: { view: "artists" }, label: "Artists", icon: "artists" },
    { route: { view: "recent" }, label: "Recently added", icon: "recent" },
    { route: { view: "playlists" }, label: "Playlists", icon: "playlist" },
  ];

  const active = (route: Route) => ui.route.view === route.view;

  async function create() {
    const id = await commands.playlistsCreate({ name: "New playlist", songs: [] });
    ui.go({ view: "playlist", id });
  }
</script>

<nav class="flex min-h-0 flex-col pb-4" aria-label="Library">
  <div class="display px-5 pb-5 text-[22px] leading-none font-semibold tracking-tight">attacca</div>
  <div class="flex flex-col gap-0.5 px-3">
    {#each places as place (place.label)}
      <button
        type="button"
        class="flex h-9 items-center gap-3 rounded-lg px-2.5 text-[14px] transition-colors {active(place.route) ? 'bg-ink-3 text-paper' : 'text-paper-2 hover:text-paper'}"
        onclick={() => ui.go(place.route)}
      >
        <Icon name={place.icon} size={18} class={active(place.route) ? "text-accent" : ""} />
        {place.label}
      </button>
    {/each}
  </div>

  <div class="mt-6 flex items-center justify-between px-5 pb-2">
    <span class="text-[13px] text-paper-3">Playlists</span>
    <button type="button" aria-label="New playlist" class="flex size-6 items-center justify-center rounded-md text-paper-3 hover:bg-ink-3 hover:text-paper" onclick={create}>
      <Icon name="plus" size={15} />
    </button>
  </div>
  <div class="scroll min-h-0 flex-1 px-3">
    {#each library.playlists as playlist (playlist.id)}
      {@const current = ui.route.view === "playlist" && ui.route.id === playlist.id}
      <button
        type="button"
        class="flex w-full items-center gap-3 rounded-lg px-2 py-1.5 text-left text-[13px] {current ? 'bg-ink-3 text-paper' : 'text-paper-2 hover:text-paper'}"
        onclick={() => ui.go({ view: "playlist", id: playlist.id })}
      >
        <Cover cover={playlist.cover ?? playlist.covers[0]} label={playlist.name} size={28} />
        <span class="truncate">{playlist.name}</span>
      </button>
    {/each}
  </div>

  <div class="px-3 pt-3">
    <button
      type="button"
      class="flex h-9 w-full items-center gap-3 rounded-lg px-2.5 text-[14px] {ui.route.view === 'sound' ? 'bg-ink-3 text-paper' : 'text-paper-2 hover:text-paper'}"
      onclick={() => ui.go({ view: "sound" })}
    >
      <Icon name="sound" size={18} class={ui.route.view === "sound" ? "text-accent" : ""} />
      Sound and folders
      {#if library.scanning}<span class="ml-auto size-1.5 animate-pulse rounded-full bg-accent" title="Scanning folders"></span>{/if}
    </button>
  </div>
</nav>
