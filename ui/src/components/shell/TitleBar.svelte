<script lang="ts">
  import { appWindow } from "@lantharos/sabine";
  import { ui } from "../../lib/stores/ui.svelte";
  import Icon from "../ui/Icon.svelte";

  let maximized = $state(appWindow.maximized);
  $effect(() => appWindow.onStateChanged((state) => (maximized = state.maximized)));

  function search(value: string) {
    ui.search = value;
    if (value.trim() && ui.route.view !== "search") ui.go({ view: "search" });
  }
</script>

<header class="drag col-span-2 flex h-12 items-center gap-3 pl-[calc(var(--sabine-controls-left,0px)+16px)]">
  <div class="no-drag flex items-center gap-1">
    <button type="button" aria-label="Back" disabled={!ui.canGoBack} class="flex size-8 items-center justify-center rounded-lg text-paper-3 hover:bg-ink-3 hover:text-paper disabled:opacity-30" onclick={() => ui.back()}>
      <Icon name="back" size={18} />
    </button>
  </div>
  <div class="flex flex-1 justify-center">
    <label class="no-drag group flex h-8 w-[min(420px,46vw)] items-center gap-2 rounded-lg bg-ink-2 px-3 text-paper-3 focus-within:bg-ink-3 focus-within:text-paper-2">
      <Icon name="search" size={15} />
      <input
        value={ui.search}
        placeholder="Search songs, albums, artists"
        aria-label="Search"
        spellcheck="false"
        class="h-full w-full bg-transparent text-[13px] text-paper outline-none placeholder:text-paper-4"
        oninput={(event) => search(event.currentTarget.value)}
        onkeydown={(event) => event.key === "Escape" && (search(""), event.currentTarget.blur())}
      />
    </label>
  </div>
  <div class="window-controls flex h-12 self-start" aria-hidden="true">
    {#each [["minimize", "minimize"], ["maximize", maximized ? "restore" : "maximize"], ["close", "close"]] as [kind, icon] (kind)}
      <span class="flex h-12 w-[46px] items-center justify-center text-paper-3 transition-colors {kind === 'close' ? 'hover:bg-[#c42b1c] hover:text-white' : 'hover:bg-ink-3 hover:text-paper'}">
        <Icon name={icon as "minimize"} size={15} />
      </span>
    {/each}
  </div>
</header>

<style>
  :global(:root[data-controls="system"]) .window-controls {
    display: none;
  }
</style>
