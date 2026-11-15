<script lang="ts">
  import { fly } from "svelte/transition";
  import { ui } from "../../lib/stores/ui.svelte";
</script>

<div class="pointer-events-none fixed bottom-28 left-1/2 z-50 flex -translate-x-1/2 flex-col items-center gap-2" aria-live="polite">
  {#each ui.toasts as toast (toast.id)}
    <div transition:fly={{ y: 8, duration: 180 }} class="rounded-xl bg-ink-4 px-4 py-2.5 text-[13px] text-paper shadow-[0_10px_32px_rgb(0_0_0/0.5)]">
      {toast.message}
    </div>
  {/each}
</div>
