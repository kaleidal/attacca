<script lang="ts">
  import { player } from "../../lib/stores/player.svelte";
  import Icon from "../ui/Icon.svelte";

  let { size = 19, class: className = "" }: { size?: number; class?: string } = $props();

  const labels = { off: "Repeat", all: "Repeating the queue", one: "Repeating this song" } as const;
  const repeat = $derived(player.state.repeat);
</script>

<button
  type="button"
  aria-label={labels[repeat]}
  aria-pressed={repeat !== "off"}
  title={labels[repeat]}
  class="flex items-center justify-center rounded-full transition-colors {repeat === 'off' ? 'text-paper-3 hover:text-paper' : 'text-accent'} {className}"
  onclick={() => player.cycleRepeat()}
>
  <Icon name={repeat === "one" ? "repeat-one" : "repeat"} {size} />
</button>
