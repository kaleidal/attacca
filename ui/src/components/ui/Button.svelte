<script lang="ts">
  import type { Snippet } from "svelte";
  import Icon, { type IconName } from "./Icon.svelte";

  let {
    onclick,
    icon,
    kind = "quiet",
    label,
    disabled = false,
    children,
    class: className = "",
  }: {
    onclick?: (event: MouseEvent) => void;
    icon?: IconName;
    kind?: "primary" | "quiet" | "plain" | "danger";
    label?: string;
    disabled?: boolean;
    children?: Snippet;
    class?: string;
  } = $props();

  const kinds = {
    primary: "bg-accent text-on-accent hover:brightness-110",
    quiet: "bg-ink-3 text-paper hover:bg-ink-4",
    plain: "text-paper-2 hover:text-paper hover:bg-ink-3",
    danger: "bg-ink-3 text-[#ff8a75] hover:bg-ink-4",
  };
</script>

<button
  type="button"
  {disabled}
  aria-label={label}
  title={children ? undefined : label}
  class="inline-flex h-9 shrink-0 items-center justify-center gap-2 rounded-lg text-[13px] font-medium transition disabled:pointer-events-none disabled:opacity-40 {children ? 'px-3.5' : 'w-9'} {kinds[kind]} {className}"
  {onclick}
>
  {#if icon}<Icon name={icon} size={18} />{/if}
  {@render children?.()}
</button>
