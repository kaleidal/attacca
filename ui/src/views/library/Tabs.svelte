<script lang="ts">
  import type { Snippet } from "svelte";
  import { Indicator } from "../../lib/indicator.svelte";
  import { ui, type Route } from "../../lib/stores/ui.svelte";

  let { meta, children }: { meta: string; children?: Snippet } = $props();

  const tabs: { label: string; route: Route }[] = [
    { label: "Songs", route: { view: "songs" } },
    { label: "Albums", route: { view: "albums" } },
    { label: "Artists", route: { view: "artists" } },
    { label: "Recent", route: { view: "recent" } },
  ];

  const indicator = new Indicator(() => tabs.findIndex((tab) => tab.route.view === ui.route.view), "library");
</script>

<header class="flex flex-col items-center gap-3 pt-7 pb-8">
  <nav aria-label="Library" class="relative flex items-baseline gap-8 pb-2">
    <span
      class="absolute bottom-0 h-[3px] rounded-full bg-accent {indicator.ready ? 'transition-[left,width] duration-300 ease-out-soft' : ''}"
      style="left:{indicator.left + indicator.width / 2 - 10}px;width:20px"
      aria-hidden="true"
    ></span>
    {#each tabs as tab, index (tab.label)}
      {@const active = ui.route.view === tab.route.view}
      <button
        bind:this={indicator.items[index]}
        type="button"
        aria-current={active ? "page" : undefined}
        class="display text-[32px] leading-none font-semibold transition-colors duration-200 {active ? 'text-paper' : 'text-paper-4 hover:text-paper-3'}"
        onclick={() => ui.go(tab.route)}
      >
        {tab.label}
      </button>
    {/each}
  </nav>
  <p class="text-[13px] text-paper-3">{meta}</p>
  {#if children}<div class="mt-1 flex items-center gap-2">{@render children()}</div>{/if}
</header>
