<script lang="ts">
  import { invoke } from "@lantharos/sabine";
  import type { CoverWrites } from "../../lib/bindings";
  import { sound } from "../../lib/stores/sound.svelte";
  import { ui, type CoverTarget } from "../../lib/stores/ui.svelte";
  import Button from "../ui/Button.svelte";
  import Dialog from "../ui/Dialog.svelte";
  import Icon from "../ui/Icon.svelte";

  const STAGE = 380;

  let { target }: { target: CoverTarget } = $props();
  let file = $state<Blob | null>(null);
  let url = $state<string | null>(null);
  let natural = $state({ width: 1, height: 1 });
  let crop = $state({ x: 0, y: 0, size: 1 });
  let asking = $state(false);
  let saving = $state(false);
  let picker: HTMLInputElement;
  let dragging: { kind: "move" | "resize"; x: number; y: number; crop: typeof crop } | null = null;

  const fit = $derived(Math.min(STAGE / natural.width, STAGE / natural.height));
  const shown = $derived({ width: natural.width * fit, height: natural.height * fit });
  const side = $derived(Math.min(shown.width, shown.height));

  async function use(blob: Blob) {
    if (!blob.type.startsWith("image/")) return ui.toast("That isn't an image");
    if (url) URL.revokeObjectURL(url);
    file = blob;
    url = URL.createObjectURL(blob);
    const image = new Image();
    image.src = url;
    await image.decode();
    natural = { width: image.naturalWidth, height: image.naturalHeight };
    const short = Math.min(image.naturalWidth, image.naturalHeight);
    crop = { x: (image.naturalWidth - short) / 2 / image.naturalWidth, y: (image.naturalHeight - short) / 2 / image.naturalHeight, size: 1 };
  }

  function paste(event: ClipboardEvent) {
    const item = [...(event.clipboardData?.items ?? [])].find((entry) => entry.type.startsWith("image/"));
    const blob = item?.getAsFile();
    if (blob) void use(blob);
  }

  function drop(event: DragEvent) {
    event.preventDefault();
    const blob = event.dataTransfer?.files[0];
    if (blob) void use(blob);
  }

  function begin(event: PointerEvent, kind: "move" | "resize") {
    event.stopPropagation();
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    dragging = { kind, x: event.clientX, y: event.clientY, crop: { ...crop } };
  }

  function drag(event: PointerEvent) {
    if (!dragging) return;
    const dx = (event.clientX - dragging.x) / shown.width;
    const dy = (event.clientY - dragging.y) / shown.height;
    const sizeX = (dragging.crop.size * side) / shown.width;
    const sizeY = (dragging.crop.size * side) / shown.height;
    if (dragging.kind === "move") {
      crop = { ...crop, x: Math.min(1 - sizeX, Math.max(0, dragging.crop.x + dx)), y: Math.min(1 - sizeY, Math.max(0, dragging.crop.y + dy)) };
    } else {
      const grow = Math.max(dx * shown.width, dy * shown.height) / side;
      const room = Math.min((1 - dragging.crop.x) * shown.width, (1 - dragging.crop.y) * shown.height) / side;
      crop = { ...crop, size: Math.min(room, Math.max(0.15, dragging.crop.size + grow)) };
    }
  }

  async function save(choice?: CoverWrites) {
    if (!file || !sound.settings) return;
    const writes = choice ?? sound.settings.coverWrites;
    if (target.type !== "playlist" && writes === "ask") {
      asking = true;
      return;
    }
    if (choice) sound.change((settings) => (settings.coverWrites = choice));
    saving = true;
    try {
      await invoke("covers.set", { target, crop, intoFiles: writes === "files" }, { body: file, timeoutMs: 120_000 });
      ui.coverTarget = null;
    } catch (error) {
      ui.toast((error as Error).message);
      saving = false;
    }
  }
</script>

<svelte:window onpaste={paste} onpointermove={drag} onpointerup={() => (dragging = null)} />

<Dialog title={target.type === "playlist" ? "Playlist cover" : target.type === "album" ? "Album cover" : "Song cover"} onclose={() => (ui.coverTarget = null)} width={460}>
  {#if asking}
    <p class="text-[14px] text-paper-2">Write the cover into the song files, or keep it in Attacca and leave the files untouched?</p>
    <p class="mt-2 text-[12px] text-paper-4">Attacca remembers this. You can change it in Sound and folders.</p>
    <div class="mt-6 flex justify-end gap-2">
      <Button kind="quiet" disabled={saving} onclick={() => save("attacca")}>Keep in Attacca</Button>
      <Button kind="primary" disabled={saving} onclick={() => save("files")}>Write into files</Button>
    </div>
  {:else if url}
    <div class="relative mx-auto overflow-hidden rounded-lg bg-ink" style="width:{shown.width}px;height:{shown.height}px">
      <img src={url} alt="" class="absolute inset-0 size-full" draggable="false" />
      <div
        role="presentation"
        class="absolute cursor-move shadow-[0_0_0_9999px_rgb(14_13_12/0.68)] outline-[1.5px] outline-paper outline-solid"
        style="left:{crop.x * shown.width}px;top:{crop.y * shown.height}px;width:{crop.size * side}px;height:{crop.size * side}px"
        onpointerdown={(event) => begin(event, "move")}
      >
        <div role="presentation" class="absolute -right-1.5 -bottom-1.5 size-3.5 cursor-nwse-resize rounded-full bg-paper" onpointerdown={(event) => begin(event, "resize")}></div>
      </div>
    </div>
    <div class="mt-6 flex justify-between gap-2">
      <Button kind="plain" icon="image" onclick={() => picker.click()}>Another image</Button>
      <Button kind="primary" disabled={saving} onclick={() => save()}>Save cover</Button>
    </div>
  {:else}
    <button
      type="button"
      class="flex h-64 w-full flex-col items-center justify-center gap-3 rounded-xl border border-dashed border-line-strong text-paper-3 hover:border-paper-4 hover:text-paper-2"
      ondragover={(event) => event.preventDefault()}
      ondrop={drop}
      onclick={() => picker.click()}
    >
      <Icon name="image" size={28} />
      <span class="text-[14px]">Drop an image, paste one, or choose a file</span>
    </button>
  {/if}
  <input bind:this={picker} type="file" accept="image/*" class="hidden" onchange={(event) => event.currentTarget.files?.[0] && use(event.currentTarget.files[0])} />
</Dialog>
