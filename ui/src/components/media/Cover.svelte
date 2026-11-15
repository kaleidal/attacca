<script lang="ts">
  import { coverUrl } from "../../lib/media";

  let { cover, label, size = 48, large = false, class: className = "" }: {
    cover: string | null | undefined;
    label: string;
    size?: number | "fill";
    large?: boolean;
    class?: string;
  } = $props();

  const url = $derived(coverUrl(cover, !large));
  const letter = $derived(label.trim().charAt(0).toUpperCase() || "♪");
  const dimension = $derived(size === "fill" ? "width:100%;aspect-ratio:1" : `width:${size}px;height:${size}px`);
  const fontSize = $derived(size === "fill" ? "clamp(20px, 30cqw, 120px)" : `${Math.round(size * 0.42)}px`);
</script>

<div class="relative shrink-0 overflow-hidden rounded-(--radius-cover) bg-ink-4 [container-type:inline-size] {className}" style={dimension}>
  {#if url}
    <img src={url} alt="" draggable="false" loading="lazy" decoding="async" class="appear absolute inset-0 size-full object-cover" />
  {:else}
    <span class="display absolute inset-0 flex items-center justify-center font-semibold text-paper-4" style="font-size:{fontSize}">{letter}</span>
  {/if}
</div>
