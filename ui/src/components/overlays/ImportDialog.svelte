<script lang="ts">
  import { commands, type ImportPreview } from "../../lib/bindings";
  import { ui } from "../../lib/stores/ui.svelte";
  import Button from "../ui/Button.svelte";
  import Dialog from "../ui/Dialog.svelte";
  import Segmented from "../ui/Segmented.svelte";

  let { path }: { path: string } = $props();
  let preview = $state<ImportPreview | null>(null);
  let failure = $state<string | null>(null);
  let keepOwn = $state(false);
  let busy = $state(false);

  $effect(() => {
    void commands
      .playlistsPreview({ path })
      .then((found) => (preview = found))
      .catch((error: Error) => (failure = error.message));
  });

  async function accept() {
    busy = true;
    try {
      const id = await commands.playlistsImport({ path, keepOwn });
      ui.importing = null;
      ui.go({ view: "playlist", id });
    } catch (error) {
      failure = (error as Error).message;
      busy = false;
    }
  }
</script>

<Dialog title={preview?.name ?? "Shared playlist"} onclose={() => (ui.importing = null)}>
  {#if failure}
    <p class="text-[14px] text-paper-2">{failure}</p>
    <div class="mt-6 flex justify-end"><Button onclick={() => (ui.importing = null)}>Close</Button></div>
  {:else if preview}
    <p class="text-[14px] text-paper-2">
      {preview.found} of {preview.songs} songs are in your library.
      {#if preview.missing}The other {preview.missing} join the playlist when you add them.{/if}
    </p>
    {#if preview.bothEdited}
      <div class="mt-5">
        <p class="mb-2.5 text-[13px] text-paper-3">You edited {preview.bothEdited === 1 ? "one of these songs" : `${preview.bothEdited} of these songs`} too.</p>
        <Segmented
          bind:value={keepOwn}
          label="Whose edits to keep"
          options={[
            { value: false as boolean, label: "Use theirs" },
            { value: true as boolean, label: "Keep mine" },
          ]}
        />
      </div>
    {/if}
    <div class="mt-7 flex justify-end gap-2">
      <Button kind="plain" onclick={() => (ui.importing = null)}>Cancel</Button>
      <Button kind="primary" disabled={busy} onclick={accept}>Add playlist</Button>
    </div>
  {:else}
    <p class="text-[14px] text-paper-3">Looking for these songs in your library…</p>
  {/if}
</Dialog>
