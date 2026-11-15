<script lang="ts">
  import type { Snippet } from "svelte";
  import Cover from "./Cover.svelte";

  let { cover, label, round = false, oncover, subtitle, meta, actions, title }: {
    cover: string | null | undefined;
    label: string;
    round?: boolean;
    oncover?: () => void;
    subtitle?: Snippet;
    meta?: string;
    actions?: Snippet;
    title?: Snippet;
  } = $props();
</script>

<header class="flex flex-col items-center pt-6 pb-9 text-center">
  {#if oncover}
    <button type="button" aria-label="Change cover" class="group relative" onclick={oncover}>
      <Cover {cover} {label} size={220} large class="{round ? 'rounded-full!' : 'rounded-2xl!'} shadow-[0_28px_70px_rgb(0_0_0/0.55)] transition-transform duration-300 ease-out-soft group-hover:-translate-y-1" />
    </button>
  {:else}
    <Cover {cover} {label} size={220} large class="{round ? 'rounded-full!' : 'rounded-2xl!'} shadow-[0_28px_70px_rgb(0_0_0/0.55)]" />
  {/if}
  <div class="mt-7 w-full max-w-[760px]">
    {#if title}{@render title()}{:else}<h1 class="display truncate text-[42px] leading-[1.05] font-semibold">{label}</h1>{/if}
  </div>
  {#if subtitle}<div class="mt-2 text-[16px] text-paper-2">{@render subtitle()}</div>{/if}
  {#if meta}<p class="mt-1.5 text-[13px] text-paper-3">{meta}</p>{/if}
  {#if actions}<div class="mt-6 flex items-center justify-center gap-2">{@render actions()}</div>{/if}
</header>
