<script lang="ts">
  import { artist, clock, title } from "../../lib/format";
  import { player } from "../../lib/stores/player.svelte";
  import { ui } from "../../lib/stores/ui.svelte";
  import { info } from "../../lib/transitions";
  import Cover from "../media/Cover.svelte";
  import Spectrum from "../media/Spectrum.svelte";
  import TransitionGlyph from "../media/TransitionGlyph.svelte";
  import Waveform from "../media/Waveform.svelte";
  import Icon from "../ui/Icon.svelte";
  import Slider from "../ui/Slider.svelte";

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
  const upcoming = $derived(player.state.upcoming);
  const overlap = $derived(upcoming && upcoming.overlap > 0 ? { from: upcoming.cue, to: upcoming.cue + upcoming.overlap } : null);
  let volume = $state(player.state.volume);
  $effect(() => {
    volume = player.state.volume;
  });
</script>

<footer class="col-span-2 grid h-[88px] grid-cols-[minmax(0,1fr)_minmax(0,1.5fr)_minmax(0,1fr)] items-center gap-6 bg-ink-2 px-4">
  <div class="flex min-w-0 items-center gap-3">
    {#if song}
      <button type="button" class="flex min-w-0 items-center gap-3 text-left" onclick={() => ui.go({ view: "now" })}>
        <Cover cover={song.cover} label={title(song)} size={56} />
        <div class="min-w-0">
          <div class="truncate text-[14px] font-medium">{title(song)}</div>
          <div class="truncate text-[13px] text-paper-3">{artist(song)}</div>
        </div>
      </button>
    {:else}
      <div class="text-[13px] text-paper-4">Nothing playing</div>
    {/if}
  </div>

  <div class="flex min-w-0 flex-col items-center gap-1.5">
    <div class="flex items-center gap-2">
      <button type="button" aria-label="Previous" class="flex size-9 items-center justify-center rounded-full text-paper-2 hover:text-paper" onclick={() => player.previous()}>
        <Icon name="previous" size={18} />
      </button>
      <button type="button" aria-label={player.state.playing ? "Pause" : "Play"} class="flex size-10 items-center justify-center rounded-full bg-paper text-ink transition-transform active:scale-95" onclick={() => player.toggle()}>
        <Icon name={player.state.playing ? "pause" : "play"} size={18} />
      </button>
      <button type="button" aria-label="Next" class="flex size-9 items-center justify-center rounded-full text-paper-2 hover:text-paper disabled:opacity-30" disabled={!player.following} onclick={() => player.next()}>
        <Icon name="next" size={18} />
      </button>
    </div>
    <div class="flex w-full items-center gap-3">
      <span class="numeric w-11 text-right text-[11px] text-paper-3">{clock(now)}</span>
      {#if song}
        <Waveform
          song={song.id}
          duration={song.duration}
          position={now}
          start={player.state.start}
          end={player.state.end}
          {overlap}
          overlapKind={upcoming?.kind}
          height={26}
          onseek={(seconds) => player.seek(seconds)}
          class="flex-1"
        />
      {:else}
        <div class="h-[26px] flex-1"></div>
      {/if}
      <span class="numeric w-11 text-[11px] text-paper-3">{song ? clock(song.duration) : ""}</span>
    </div>
  </div>

  <div class="flex items-center justify-end gap-2">
    {#if upcoming && player.nextSong}
      <div class="mr-2 hidden items-center gap-2 text-[12px] text-paper-3 xl:flex" title="{info(upcoming.kind).label} into {title(player.nextSong)}">
        <TransitionGlyph kind={upcoming.kind} width={30} />
        <span class="max-w-32 truncate">{title(player.nextSong)}</span>
      </div>
    {/if}
    <button type="button" aria-label="Visualizer" class="flex h-9 items-center rounded-lg px-2 hover:bg-ink-3" onclick={() => (ui.visualizer = true)}>
      <Spectrum bars={10} height={18} />
    </button>
    <button type="button" aria-label="Queue" class="flex size-9 items-center justify-center rounded-lg hover:bg-ink-3 {ui.queueOpen ? 'text-accent' : 'text-paper-2'}" onclick={() => (ui.queueOpen = !ui.queueOpen)}>
      <Icon name="queue" size={18} />
    </button>
    <div class="flex w-32 items-center gap-2 text-paper-2">
      <Icon name={volume === 0 ? "mute" : "volume"} size={18} />
      <Slider bind:value={volume} label="Volume" onchange={(value) => player.volume(value)} />
    </div>
  </div>
</footer>
