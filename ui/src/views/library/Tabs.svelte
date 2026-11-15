<script lang="ts">
  import type { Snippet } from "svelte";
  import { ui, type Route } from "../../lib/stores/ui.svelte";

  let { meta, children }: { meta: string; children?: Snippet } = $props();

  const tabs: { label: string; route: Route }[] = [
    { label: "Songs", route: { view: "songs" } },
    { label: "Albums", route: { view: "albums" } },
    { label: "Artists", route: { view: "artists" } },
    { label: "Recent", route: { view: "recent" } },
  ];
</script>

<header class="flex flex-col items-center gap-3 pt-7 pb-8">
  <nav aria-label="Library" class="flex items-baseline gap-8">
    {#each tabs as tab (tab.label)}
      {@const active = ui.route.view === tab.route.view}
      <button
        type="button"
        aria-current={active ? "page" : undefined}
        class="display text-[32px] leading-none font-semibold transition-colors {active ? 'text-paper' : 'text-paper-4 hover:text-paper-3'}"
        onclick={() => ui.go(tab.route)}
      >
        {tab.label}
      </button>
    {/each}
  </nav>
  <p class="text-[13px] text-paper-3">{meta}</p>
  {#if children}<div class="mt-1 flex items-center gap-2">{@render children()}</div>{/if}
</header>
