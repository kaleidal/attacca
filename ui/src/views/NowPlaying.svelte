<script lang="ts">
  import { commands } from "../lib/bindings";
  import { artist, clock, title } from "../lib/format";
  import { library } from "../lib/stores/library.svelte";
  import { player } from "../lib/stores/player.svelte";
  import { ui } from "../lib/stores/ui.svelte";
  import { info } from "../lib/transitions";
  import Cover from "../components/media/Cover.svelte";
  import TransitionGlyph from "../components/media/TransitionGlyph.svelte";
  import Waveform from "../components/media/Waveform.svelte";
  import Button from "../components/ui/Button.svelte";

  let now = $state(0);
  $effect(() => {
    let frame = 0;
    const tick = () => {
      now = player.seconds();
      frame = requestAnimationFrame(tick);
    };
    tick();
    return () => cancelAnimationFrame(frame);
  });

  const song = $derived(player.song);
  const next = $derived(player.nextSong);
  const upcoming = $derived(player.state.upcoming);
  const overlap = $derived(upcoming && upcoming.overlap > 0 ? { from: upcoming.cue, to: upcoming.cue + upcoming.overlap } : null);
  const nextPlacement = $derived(
    song && next && upcoming
      ? { left: ((upcoming.cue - upcoming.start) / song.duration) * 100, width: (next.duration / song.duration) * 100 }
      : null,
  );
</script>

{#if song}
  <div class="flex h-full flex-col px-10 pb-8">
    <div class="grid min-h-0 flex-1 grid-cols-[minmax(0,1fr)_320px] gap-12 pt-8">
      <div class="flex min-w-0 items-end gap-9">
        <div class="w-[min(420px,40vh)] shrink-0">
          <Cover cover={song.cover} label={title(song)} size="fill" large class="shadow-[0_30px_80px_rgb(0_0_0/0.55)]" />
        </div>
        <div class="min-w-0 pb-2">
          <h1 class="display text-[clamp(32px,4.2vw,58px)] leading-[1.02] font-semibold">{title(song)}</h1>
          <button type="button" class="mt-3 block text-[17px] text-paper-2 hover:text-paper" onclick={() => ui.go({ view: "artist", name: artist(song) })}>{artist(song)}</button>
          {#if song.album}<div class="mt-1 text-[14px] text-paper-3">{song.album}</div>{/if}
          <div class="mt-6 flex gap-2">
            <Button icon="edit" onclick={() => ui.go({ view: "edit", song: song.id, next: next?.id ?? null, context: player.current?.context ?? { type: "queue" } })}>Edit song</Button>
            <Button icon="visual" onclick={() => (ui.visualizer = true)}>Visualizer</Button>
          </div>
        </div>
      </div>
      <aside class="scroll min-h-0 min-w-0">
        <h2 class="display mb-3 text-lg font-semibold">Up next</h2>
        {#each player.upNext.slice(0, 8) as entry, index (entry.token)}
          {@const queued = library.byId.get(entry.song)}
          {#if queued}
            <button type="button" class="flex w-full items-center gap-3 rounded-xl p-2 text-left hover:bg-ink-2" ondblclick={() => commands.playerJump({ token: entry.token })}>
              <Cover cover={queued.cover} label={title(queued)} size={40} />
              <div class="min-w-0 flex-1">
                <div class="truncate text-[13px]">{title(queued)}</div>
                <div class="truncate text-[12px] text-paper-3">{artist(queued)}</div>
              </div>
              {#if index === 0 && upcoming}<TransitionGlyph kind={upcoming.kind} width={28} />{/if}
            </button>
          {/if}
        {:else}
          <p class="text-[13px] text-paper-4">The queue ends with this song.</p>
        {/each}
      </aside>
    </div>

    <section class="mt-8 shrink-0" aria-label="Timeline">
      <div class="mb-2 flex items-baseline justify-between text-[12px]">
        <span class="numeric text-paper-2">{clock(now)}</span>
        {#if upcoming && next}
          <span class="flex items-center gap-2 text-paper-3">
            <TransitionGlyph kind={upcoming.kind} width={26} />
            {info(upcoming.kind).label} into {title(next)}{upcoming.overlap > 0 ? ` at ${clock(upcoming.cue)}` : ""}
          </span>
        {/if}
        <span class="numeric text-paper-3">{clock(song.duration)}</span>
      </div>
      <Waveform song={song.id} duration={song.duration} position={now} start={player.state.start} end={player.state.end} {overlap} overlapKind={upcoming?.kind} height={84} bar={3} onseek={(seconds) => player.seek(seconds)} />
      {#if next && nextPlacement}
        <div class="relative mt-1.5 h-10 overflow-hidden opacity-70" aria-hidden="true">
          <div class="absolute inset-y-0" style="left:{nextPlacement.left}%;width:{nextPlacement.width}%">
            <Waveform song={next.id} duration={next.duration} position={0} start={upcoming?.start ?? 0} height={40} bar={3} />
          </div>
        </div>
      {/if}
    </section>
  </div>
{:else}
  <div class="flex h-full items-center justify-center text-[14px] text-paper-4">Play something to see it here.</div>
{/if}
