<script lang="ts">
  import { commands, type Context, type Song } from "../../lib/bindings";
  import { precise, title } from "../../lib/format";
  import { waveform } from "../../lib/media";
  import { player } from "../../lib/stores/player.svelte";
  import { info } from "../../lib/transitions";
  import Segment, { type Handle } from "../media/Segment.svelte";
  import TransitionGlyph from "../media/TransitionGlyph.svelte";
  import TransitionPicker from "../media/TransitionPicker.svelte";
  import Button from "../ui/Button.svelte";
  import { SeamTrial } from "./seam.svelte";

  const MARGIN = 7;
  const SHORTEST = 18;
  const LOOP = 3;
  const AUDITION_EVERY_MS = 140;

  let { from, to, context, framed = true }: { from: Song; to: Song; context: Context; framed?: boolean } = $props();

  const seam = new SeamTrial(() => ({ from: from.id, to: to.id, context }));
  let peaks = $state.raw<{ from: Uint8Array | null; to: Uint8Array | null }>({ from: null, to: null });
  let frozen = $state<{ start: number; span: number; shift: number; cue: number } | null>(null);
  let auditioned = 0;
  let now = $state<{ song: string; seconds: number } | null>(null);

  $effect(() => {
    const pair = [from.id, to.id] as const;
    void seam.load();
    void Promise.all(pair.map(waveform)).then(([outgoing, incoming]) => {
      if (pair[0] === from.id && pair[1] === to.id) peaks = { from: outgoing!, to: incoming! };
    });
  });

  $effect(() => {
    const playing = player.song?.id;
    if (playing !== from.id && playing !== to.id) {
      now = null;
      return;
    }
    let frame = 0;
    const tick = () => {
      now = { song: playing, seconds: player.seconds() };
      frame = requestAnimationFrame(tick);
    };
    tick();
    return () => cancelAnimationFrame(frame);
  });

  const placed = $derived(seam.placed);
  const kind = $derived(placed?.kind ?? "fade");
  const overlap = $derived(placed?.overlap ?? 0);
  const view = $derived.by(() => {
    if (frozen) return frozen;
    if (!placed) return null;
    const span = Math.max(SHORTEST, overlap + MARGIN * 2);
    return { start: placed.cue - (span - overlap) / 2, span, shift: placed.start - placed.cue, cue: placed.cue };
  });
  const outgoingStart = $derived(seam.outgoing?.edits.start ?? seam.outgoing?.listening?.suggestedStart ?? 0);
  const lengthy = $derived(overlap > 0 && kind !== "slam");

  function audition(song: string, start: number) {
    const at = performance.now();
    if (at - auditioned < AUDITION_EVERY_MS) return;
    auditioned = at;
    void commands.playerAudition({ song, from: Math.max(0, start), to: start + LOOP });
  }

  function pickOutgoing(seconds: number, handle: string | null) {
    if (!view) return;
    frozen = view;
    if (handle === "length") {
      const [shortest, longest] = kind === "slam" ? [0.3, 2] : [1, 32];
      seam.try({ transition: { ...seam.transition, seconds: Math.min(longest, Math.max(shortest, seconds - view.cue)) } });
      return;
    }
    seam.try({ end: Math.max(outgoingStart + 1, Math.min(from.duration, seconds)) });
    audition(from.id, seconds - LOOP);
  }

  function pickIncoming(seconds: number) {
    if (!view) return;
    frozen = view;
    seam.try({ start: Math.max(0, Math.min(to.duration - 1, seconds)) });
    audition(to.id, seconds);
  }

  async function release() {
    void commands.playerStopAudition();
    await seam.save();
    frozen = null;
  }

  function hear() {
    void commands.playerPreview({ from: from.id, to: to.id, context });
  }

  const outgoingHandles = $derived<Handle[]>(
    placed ? [{ id: "end", at: placed.cue, side: "end" }, ...(lengthy ? [{ id: "length", at: placed.cue + overlap, side: "end" as const, color: kind }] : [])] : [],
  );
  const lengthLabel = $derived(kind === "none" ? "" : ` · ${overlap < 2 ? overlap.toFixed(1) : overlap.toFixed(overlap < 10 ? 1 : 0)} s`);
</script>

<div class="flex flex-col gap-5 {framed ? 'rounded-2xl bg-ink-2 p-4' : ''}">
  {#if view && placed}
    <div class="flex flex-col gap-1.5">
      <div class="truncate text-[12px] text-paper-3">{title(from)}</div>
      <Segment
        peaks={peaks.from}
        from={view.start}
        to={view.start + view.span}
        height={68}
        class="bg-ink"
        start={outgoingStart}
        end={placed.cue + overlap}
        beats={seam.outgoing?.listening?.rhythm?.beats ?? []}
        downbeat={seam.outgoing?.listening?.rhythm?.downbeat ?? 0}
        position={now?.song === from.id ? now.seconds : null}
        region={overlap > 0 ? { from: placed.cue, to: placed.cue + overlap, kind } : null}
        handles={outgoingHandles}
        onpick={pickOutgoing}
        onrelease={release}
      />
      <Segment
        peaks={peaks.to}
        from={view.start + view.shift}
        to={view.start + view.shift + view.span}
        height={68}
        class="bg-ink"
        start={placed.start}
        end={to.duration}
        beats={seam.incoming?.listening?.rhythm?.beats ?? []}
        downbeat={seam.incoming?.listening?.rhythm?.downbeat ?? 0}
        position={now?.song === to.id ? now.seconds : null}
        region={overlap > 0 ? { from: placed.start, to: placed.start + overlap, kind } : null}
        handles={[{ id: "start", at: placed.start, side: "start" }]}
        onpick={pickIncoming}
        onrelease={release}
      />
      <div class="truncate text-[12px] text-paper-3">{title(to)}</div>
    </div>
    <div class="grid grid-cols-[1fr_auto_1fr] items-center gap-3 text-[13px]">
      <span class="text-paper-3">Ends <span class="numeric text-paper">{precise(placed.cue)}</span></span>
      <span class="flex items-center gap-2 text-paper-2"><TransitionGlyph {kind} width={28} />{info(kind).label}{lengthLabel}</span>
      <span class="text-right text-paper-3">Starts <span class="numeric text-paper">{precise(placed.start)}</span></span>
    </div>
  {:else}
    <div class="h-[178px] w-full rounded-xl bg-ink"></div>
  {/if}
  <div class="flex items-start gap-6">
    <div class="w-[340px] shrink-0">
      <TransitionPicker value={seam.trial.transition === undefined ? seam.own : seam.trial.transition} fallback={seam.fallback} onchange={(transition) => seam.choose(transition)} />
    </div>
    <div class="flex flex-1 flex-col items-end gap-2">
      <Button kind="primary" icon="play" onclick={hear}>Hear it</Button>
    </div>
  </div>
</div>
