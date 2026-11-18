<script lang="ts">
  import { commands, type Context, type Edits, type SongDetail } from "../../lib/bindings";
  import { artist, decibels, precise, title } from "../../lib/format";
  import { waveform } from "../../lib/media";
  import { library } from "../../lib/stores/library.svelte";
  import { player } from "../../lib/stores/player.svelte";
  import { sound } from "../../lib/stores/sound.svelte";
  import { ui } from "../../lib/stores/ui.svelte";
  import { fade } from "../../lib/transitions";
  import Cover from "../../components/media/Cover.svelte";
  import TransitionPicker from "../../components/media/TransitionPicker.svelte";
  import EqualizerPanel from "../../components/sound/EqualizerPanel.svelte";
  import FiltersPanel from "../../components/sound/FiltersPanel.svelte";
  import Button from "../../components/ui/Button.svelte";
  import Slider from "../../components/ui/Slider.svelte";
  import Toggle from "../../components/ui/Toggle.svelte";
  import Segment from "../../components/media/Segment.svelte";

  const LOOP = 3;
  const DETAIL = 7;

  let { song: id, next, context }: { song: string; next: string | null; context: Context } = $props();

  let detail = $state<SongDetail | null>(null);
  let edits = $state<Edits>({ start: null, end: null, volume: 0, equalizer: null, filters: null, transition: null });
  let peaks = $state.raw<Uint8Array | null>(null);
  let volume = $state(0);
  let saving: ReturnType<typeof setTimeout> | null = null;
  let auditioned = 0;
  let held = $state<{ start: number; end: number } | null>(null);

  $effect(() => {
    const song = id;
    void commands.libraryDetail({ id: song }).then((found) => {
      detail = found;
      if (found) {
        edits = found.edits;
        volume = found.edits.volume;
      }
    });
    void waveform(song).then((bytes) => (peaks = bytes));
  });

  const song = $derived(detail?.song ?? library.byId.get(id) ?? null);
  const duration = $derived(song?.duration ?? 0);
  const listening = $derived(detail?.listening ?? null);
  const start = $derived(edits.start ?? listening?.suggestedStart ?? 0);
  const end = $derived(edits.end ?? listening?.suggestedEnd ?? duration);
  const beats = $derived(listening?.rhythm?.beats ?? []);
  const nextSong = $derived(next ? library.byId.get(next) : player.song?.id === id ? player.nextSong : null);
  const playing = $derived(player.song?.id === id);

  let now = $state<number | null>(null);
  $effect(() => {
    if (!playing) {
      now = null;
      return;
    }
    let frame = 0;
    const tick = () => {
      now = player.seconds();
      frame = requestAnimationFrame(tick);
    };
    tick();
    return () => cancelAnimationFrame(frame);
  });

  function save() {
    if (saving) clearTimeout(saving);
    const snapshot = $state.snapshot(edits);
    saving = setTimeout(() => void commands.editsSet({ song: id, edits: snapshot }), 250);
  }

  function update(change: Partial<Edits>) {
    edits = { ...edits, ...change };
    save();
  }

  function move(seconds: number, handle: string | null) {
    if (handle !== "start" && handle !== "end") return;
    held ??= { start, end };
    if (handle === "start") edits.start = Math.min(Math.max(0, seconds), end - 1);
    else edits.end = Math.max(Math.min(duration, seconds), start + 1);
    const at = performance.now();
    if (at - auditioned > 140) {
      auditioned = at;
      const from = handle === "start" ? edits.start! : edits.end! - LOOP;
      void commands.playerAudition({ song: id, from, to: from + LOOP });
    }
  }

  function release() {
    held = null;
    void commands.playerStopAudition();
    save();
  }

  function preview() {
    if (nextSong) void commands.playerPreview({ from: id, to: nextSong.id, context });
  }

  async function reset() {
    await commands.editsReset({ id });
    edits = { start: null, end: null, volume: 0, equalizer: null, filters: null, transition: null };
    volume = 0;
  }

  const ownSound = $derived(edits.equalizer !== null || edits.filters !== null);
</script>

{#if song}
  <div class="scroll h-full px-10 pb-32">
    <div class="mx-auto max-w-[1180px]">
    <header class="flex flex-col items-center pt-4 pb-8 text-center">
      <Cover cover={song.cover} label={title(song)} size={88} class="rounded-xl! shadow-[0_18px_50px_rgb(0_0_0/0.5)]" />
      <h1 class="display mt-5 max-w-[760px] truncate text-[34px] leading-tight font-semibold">{title(song)}</h1>
      <p class="mt-1 text-[14px] text-paper-3">{artist(song)}{listening?.rhythm ? ` · ${Math.round(listening.rhythm.tempo)} BPM` : ""}{listening ? ` · ${listening.loudness.toFixed(1)} LUFS` : ""}</p>
      <div class="mt-5 flex items-center gap-2">
        <Button kind="primary" icon="play" disabled={!nextSong} onclick={preview}>{nextSong ? `Hear the transition into ${title(nextSong)}` : "Hear the transition"}</Button>
        <Button kind="quiet" icon="reset" onclick={reset} disabled={!song.edited && JSON.stringify(edits) === JSON.stringify({ start: null, end: null, volume: 0, equalizer: null, filters: null, transition: null })}>Reset</Button>
      </div>
    </header>

    <Segment {peaks} from={0} to={duration} height={132} {start} {end} position={now} {beats} downbeat={listening?.rhythm?.downbeat ?? 0} handles={[{ id: "start", at: start, side: "start" }, { id: "end", at: end, side: "end" }]} onpick={move} onrelease={release} />

    <div class="mt-4 grid grid-cols-2 gap-6">
      {#each ["start", "end"] as const as handle (handle)}
        {@const at = handle === "start" ? start : end}
        {@const centre = held ? held[handle] : at}
        {@const suggested = handle === "start" ? listening?.suggestedStart : listening?.suggestedEnd}
        <div>
          <div class="mb-2 flex items-baseline justify-between">
            <span class="text-[14px]">{handle === "start" ? "Start" : "End"}</span>
            <span class="flex items-baseline gap-3">
              {#if (handle === "start" ? edits.start : edits.end) !== null && suggested !== undefined}
                <button type="button" class="text-[12px] text-paper-3 hover:text-paper" onclick={() => update(handle === "start" ? { start: null } : { end: null })}>Use suggestion</button>
              {/if}
              <span class="numeric text-[13px] text-paper-2">{precise(at)}</span>
            </span>
          </div>
          <Segment
            {peaks}
            from={Math.max(0, centre - DETAIL)}
            to={Math.min(duration, centre + DETAIL)}
            {start}
            {end}
            position={now}
            {beats}
            downbeat={listening?.rhythm?.downbeat ?? 0}
            ghosts={suggested === undefined ? [] : [suggested]}
            handles={[{ id: handle, at, side: handle }]}
            onpick={move}
            onrelease={release}
          />
        </div>
      {/each}
    </div>
    <p class="mt-3 text-center text-[12px] text-paper-4">Drag a handle to hear the moment around it. Dashed lines mark where Attacca suggests.</p>

    <div class="mt-10 grid grid-cols-1 gap-x-14 gap-y-12 xl:grid-cols-[minmax(0,1fr)_minmax(0,1.4fr)]">
      <section class="flex flex-col gap-10">
        <div>
          <h2 class="display mb-4 text-xl font-semibold">Transition out</h2>
          <div class="rounded-2xl bg-ink-2 p-4">
            <TransitionPicker value={edits.transition} fallback={{ label: "Default", transition: fade }} onchange={(transition) => update({ transition })} />
          </div>
        </div>
        <div>
          <h2 class="display mb-4 text-xl font-semibold">Volume</h2>
          <div class="flex items-center gap-4">
            <Slider bind:value={volume} min={-12} max={12} step={0.5} centered label="Volume offset" oncommit={(value) => update({ volume: value })} />
            <span class="numeric w-16 text-right text-[12px] text-paper-2">{decibels(volume)}</span>
          </div>
          <p class="mt-2 text-[12px] text-paper-4">On top of volume matching.</p>
        </div>
      </section>

      <section>
        <div class="mb-5 flex items-center justify-between">
          <h2 class="display text-xl font-semibold">Its own sound</h2>
          <Toggle
            checked={ownSound}
            label="Its own sound"
            onchange={(on) =>
              update(on ? { equalizer: structuredClone($state.snapshot(sound.settings?.equalizer ?? { preamp: 0, gains: Array(10).fill(0) })), filters: structuredClone($state.snapshot(sound.settings?.filters ?? { highpass: null, lowpass: null })) } : { equalizer: null, filters: null })}
          />
        </div>
        {#if ownSound}
          <EqualizerPanel equalizer={edits.equalizer ?? { preamp: 0, gains: Array(10).fill(0) }} onchange={(equalizer) => update({ equalizer })} />
          <div class="mt-8"><FiltersPanel filters={edits.filters ?? { highpass: null, lowpass: null }} onchange={(filters) => update({ filters })} /></div>
        {:else}
          <p class="text-[13px] text-paper-3">This song uses the equalizer and filters from Sound.</p>
        {/if}
      </section>
    </div>
    </div>
  </div>
{/if}
