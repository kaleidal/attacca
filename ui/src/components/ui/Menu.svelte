<script lang="ts">
  import { cubicOut } from "svelte/easing";
  import { scale } from "svelte/transition";
  import { motion } from "../../lib/motion";
  import { ui } from "../../lib/stores/ui.svelte";

  let element = $state<HTMLDivElement>();
  let place = $state({ x: 0, y: 0 });

  $effect(() => {
    const menu = ui.menu;
    if (!menu || !element) return;
    const { width, height } = element.getBoundingClientRect();
    place = {
      x: Math.min(menu.x, window.innerWidth - width - 8),
      y: Math.min(menu.y, window.innerHeight - height - 8),
    };
  });

  function close() {
    ui.menu = null;
  }
</script>

<svelte:window onkeydown={(event) => event.key === "Escape" && close()} onblur={close} />

{#if ui.menu}
  <div role="presentation" class="fixed inset-0 z-50" onpointerdown={close} oncontextmenu={(event) => (event.preventDefault(), close())}></div>
  <div
    bind:this={element}
    role="menu"
    transition:scale={{ start: 0.95, duration: motion(120), easing: cubicOut }}
    class="fixed z-50 min-w-48 rounded-xl bg-ink-4 p-1.5 shadow-[0_16px_48px_rgb(0_0_0/0.6)]"
    style="left:{place.x}px;top:{place.y}px"
  >
    {#each ui.menu.items as item (item.label)}
      <button
        type="button"
        role="menuitem"
        class="block w-full rounded-lg px-3 py-1.5 text-left text-[13px] transition-colors duration-100 hover:bg-ink-5 {item.danger ? 'text-[#ff8a75]' : 'text-paper'}"
        onclick={() => {
          close();
          item.action();
        }}
      >
        {item.label}
      </button>
    {/each}
  </div>
{/if}
