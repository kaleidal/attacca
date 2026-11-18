<script lang="ts">
  import type { Context } from "../../lib/bindings";
  import { library } from "../../lib/stores/library.svelte";
  import { ui } from "../../lib/stores/ui.svelte";
  import SeamEditor from "../seam/SeamEditor.svelte";
  import Dialog from "../ui/Dialog.svelte";

  let { from, to, context }: { from: string; to: string; context: Context } = $props();

  const outgoing = $derived(library.byId.get(from));
  const incoming = $derived(library.byId.get(to));
</script>

<Dialog title="Transition" onclose={() => (ui.seam = null)} width={860}>
  {#if outgoing && incoming}
    <SeamEditor from={outgoing} to={incoming} {context} framed={false} />
  {/if}
</Dialog>
