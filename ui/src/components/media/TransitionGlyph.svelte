<script lang="ts">
  import type { TransitionName } from "../../lib/bindings";
  import { glyphs } from "../../lib/transitions";

  let { kind, width = 40, class: className = "" }: { kind: TransitionName; width?: number; class?: string } = $props();
  const [outgoing, incoming] = $derived(glyphs[kind]);
</script>

<svg {width} height={width * 0.4} viewBox="0 0 40 16" fill="none" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" class="shrink-0 {className}" aria-hidden="true">
  <path d={outgoing} stroke="currentColor" opacity="0.45" />
  <path d={incoming} style="stroke: var(--color-{kind})" />
</svg>
