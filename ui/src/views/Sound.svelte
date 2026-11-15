<script lang="ts">
  import type { CoverWrites, Equalizer } from "../lib/bindings";
  import { library } from "../lib/stores/library.svelte";
  import { sound } from "../lib/stores/sound.svelte";
  import EqualizerPanel from "../components/sound/EqualizerPanel.svelte";
  import FiltersPanel from "../components/sound/FiltersPanel.svelte";
  import Button from "../components/ui/Button.svelte";
  import Icon from "../components/ui/Icon.svelte";
  import Segmented from "../components/ui/Segmented.svelte";
  import Select from "../components/ui/Select.svelte";
  import TextField from "../components/ui/TextField.svelte";
  import Toggle from "../components/ui/Toggle.svelte";

  const settings = $derived(sound.settings);
  let naming = $state(false);
  let presetName = $state("");

  const flat = (): Equalizer => ({ preamp: 0, gains: Array(10).fill(0) });
  const same = (a: Equalizer, b: Equalizer) => a.preamp === b.preamp && a.gains.every((gain, index) => gain === b.gains[index]);
  const preset = $derived(settings?.presets.find((entry) => same(entry.equalizer, settings.equalizer))?.name ?? (settings && same(settings.equalizer, flat()) ? "Flat" : "Custom"));

  function choosePreset(name: string) {
    sound.change((current) => {
      current.equalizer = name === "Flat" ? flat() : structuredClone($state.snapshot(current.presets.find((entry) => entry.name === name)!.equalizer));
    });
  }

  function savePreset() {
    const name = presetName.trim();
    naming = false;
    if (!name) return;
    sound.change((current) => {
      current.presets = [...current.presets.filter((entry) => entry.name !== name), { name, equalizer: structuredClone($state.snapshot(current.equalizer)) }];
    });
  }
</script>

{#if settings}
  <div class="scroll h-full px-8 pb-12">
    <header class="pt-6 pb-8">
      <h1 class="display text-[34px] leading-none font-semibold">Sound and folders</h1>
    </header>

    <div class="grid grid-cols-1 gap-x-14 gap-y-12 xl:grid-cols-[minmax(0,1.3fr)_minmax(0,1fr)]">
      <section>
        <div class="mb-5 flex items-center justify-between gap-4">
          <h2 class="display text-xl font-semibold">Equalizer</h2>
          <div class="flex items-center gap-2">
            {#if naming}
              <TextField bind:value={presetName} label="Preset name" placeholder="Name this curve" autofocus onsubmit={savePreset} class="w-44" />
              <Button kind="quiet" onclick={savePreset}>Save</Button>
            {:else}
              <Select
                value={preset}
                label="Equalizer preset"
                class="w-40"
                options={[{ value: "Flat", label: "Flat" }, ...settings.presets.map((entry) => ({ value: entry.name, label: entry.name })), ...(preset === "Custom" ? [{ value: "Custom", label: "Custom" }] : [])]}
                onchange={(name) => name !== "Custom" && choosePreset(name)}
              />
              <Button kind="plain" onclick={() => ((presetName = ""), (naming = true))}>Save as preset</Button>
            {/if}
          </div>
        </div>
        <EqualizerPanel equalizer={settings.equalizer} onchange={(equalizer) => sound.change((current) => (current.equalizer = equalizer))} />
        <p class="mt-4 text-[12px] text-paper-4">A song with its own curve uses it in place of this one.</p>
      </section>

      <section>
        <h2 class="display mb-5 text-xl font-semibold">Filters</h2>
        <FiltersPanel filters={settings.filters} onchange={(filters) => sound.change((current) => (current.filters = filters))} />
      </section>

      <section class="flex flex-col gap-7">
        <div class="flex items-start justify-between gap-6">
          <div>
            <h2 class="display text-xl font-semibold">Volume matching</h2>
            <p class="mt-1.5 max-w-md text-[13px] text-paper-3">Every song plays at the same loudness. Albums keep the differences between their tracks.</p>
          </div>
          <Toggle checked={settings.matching} label="Volume matching" onchange={(on) => sound.change((current) => (current.matching = on))} />
        </div>
        <div>
          <h2 class="display mb-3 text-xl font-semibold">Output</h2>
          <Select
            value={settings.output}
            label="Output device"
            class="max-w-md"
            options={[{ value: null, label: "System default" }, ...sound.devices.map((device) => ({ value: device.name as string | null, label: device.description }))]}
            onchange={(output) => sound.change((current) => (current.output = output))}
          />
        </div>
        <div>
          <h2 class="display mb-1.5 text-xl font-semibold">Song and album covers</h2>
          <p class="mb-3 max-w-md text-[13px] text-paper-3">Where a new cover goes. Playlist covers always stay with Attacca.</p>
          <Segmented
            value={settings.coverWrites}
            label="Where covers go"
            options={[
              { value: "ask" as CoverWrites, label: "Ask me" },
              { value: "files" as CoverWrites, label: "Into the files" },
              { value: "attacca" as CoverWrites, label: "Kept by Attacca" },
            ]}
            onchange={(choice) => sound.change((current) => (current.coverWrites = choice))}
          />
        </div>
      </section>

      <section>
        <div class="mb-4 flex items-center justify-between">
          <h2 class="display text-xl font-semibold">Folders</h2>
          <Button icon="plus" onclick={() => library.addFolder()}>Add a folder</Button>
        </div>
        <div class="flex flex-col gap-1">
          {#each library.folders as folder (folder)}
            <div class="group flex items-center gap-3 rounded-lg px-2 py-2 hover:bg-ink-2">
              <Icon name="folder" size={18} class="text-paper-3" />
              <span class="min-w-0 flex-1 truncate text-[13px]" title={folder}>{folder}</span>
              <button type="button" class="rounded-md px-2 py-1 text-[12px] text-paper-3 opacity-0 group-hover:opacity-100 hover:bg-ink-3 hover:text-paper" onclick={() => library.removeFolder(folder)}>Remove</button>
            </div>
          {:else}
            <p class="text-[13px] text-paper-4">No folders yet.</p>
          {/each}
        </div>
      </section>
    </div>
  </div>
{/if}
