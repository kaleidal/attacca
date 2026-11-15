<script lang="ts">
  import { appWindow } from "@lantharos/sabine";
  import { Indicator } from "../../lib/indicator.svelte";
  import { library } from "../../lib/stores/library.svelte";
  import { ui, type Route, type Section } from "../../lib/stores/ui.svelte";
  import Icon from "../ui/Icon.svelte";

  let maximized = $state(appWindow.maximized);
  $effect(() => appWindow.onStateChanged((state) => (maximized = state.maximized)));

  const sections: { section: Section; label: string; route: Route }[] = [
    { section: "listen", label: "Listen", route: { view: "listen" } },
    { section: "library", label: "Library", route: { view: "songs" } },
    { section: "playlists", label: "Playlists", route: { view: "playlists" } },
    { section: "sound", label: "Sound", route: { view: "sound" } },
  ];

  const indicator = new Indicator(() => sections.findIndex((entry) => entry.section === ui.section));
  const deep = $derived(["album", "artist", "playlist", "edit"].includes(ui.route.view));
</script>

<header class="drag relative flex h-14 shrink-0 items-center">
  <div class="flex items-center gap-1 pl-[calc(var(--sabine-controls-left,0px)+14px)]">
    {#if deep}
      <button type="button" aria-label="Back" class="no-drag flex size-8 items-center justify-center rounded-full text-paper-2 hover:bg-ink-3 hover:text-paper" onclick={() => ui.back()}>
        <Icon name="back" size={18} />
      </button>
    {:else}
      <span class="display px-2 text-[19px] font-semibold tracking-tight">attacca</span>
    {/if}
    {#if library.scanning}<span class="ml-1 size-1.5 animate-pulse rounded-full bg-accent" title="Reading your folders"></span>{/if}
  </div>

  <nav aria-label="Sections" class="no-drag absolute top-1/2 left-1/2 flex -translate-x-1/2 -translate-y-1/2 items-center gap-0.5 rounded-full bg-ink-2 p-1">
    <span
      class="absolute top-1 bottom-1 rounded-full bg-ink-4 {indicator.ready ? 'transition-[left,width] duration-300 ease-out-soft' : ''}"
      style="left:{indicator.left}px;width:{indicator.width}px"
      aria-hidden="true"
    ></span>
    {#each sections as entry, index (entry.section)}
      <button
        bind:this={indicator.items[index]}
        type="button"
        aria-current={ui.section === entry.section ? "page" : undefined}
        class="relative rounded-full px-4 py-1.5 text-[13px] font-medium transition-colors duration-200 active:scale-95 {ui.section === entry.section ? 'text-paper' : 'text-paper-3 hover:text-paper-2'}"
        onclick={() => ui.go(entry.route)}
      >
        {entry.label}
      </button>
    {/each}
  </nav>

  <div class="ml-auto flex h-full items-center">
    <button type="button" aria-label="Search" class="no-drag flex h-full w-[46px] items-center justify-center text-paper-2 transition-colors duration-150 hover:bg-ink-3 hover:text-paper" onclick={() => (ui.searching = true)}>
      <Icon name="search" size={17} />
    </button>
    <div class="window-controls flex h-full" aria-hidden="true">
      {#each [["minimize", "minimize"], ["maximize", maximized ? "restore" : "maximize"], ["close", "close"]] as [kind, icon] (kind)}
        <span class="flex h-full w-[46px] items-center justify-center text-paper-2 transition-colors duration-150 {kind === 'close' ? 'hover:bg-[#c42b1c] hover:text-white' : 'hover:bg-ink-3 hover:text-paper'}">
          <Icon name={icon as "minimize"} size={17} />
        </span>
      {/each}
    </div>
  </div>
</header>

<style>
  :global(:root[data-controls="system"]) .window-controls {
    display: none;
  }
</style>
