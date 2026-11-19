<script lang="ts">
  import { cubicOut } from "svelte/easing";
  import { fly, scale } from "svelte/transition";
  import { artist, clock, title } from "../lib/format";
  import { motion } from "../lib/motion";
  import { library } from "../lib/stores/library.svelte";
  import { player } from "../lib/stores/player.svelte";
  import { ui } from "../lib/stores/ui.svelte";
  import { info } from "../lib/transitions";
  import Cover from "../components/media/Cover.svelte";
  import TransitionGlyph from "../components/media/TransitionGlyph.svelte";
  import Waveform from "../components/media/Waveform.svelte";
  import Button from "../components/ui/Button.svelte";
  import Icon from "../components/ui/Icon.svelte";
  import Slider from "../components/ui/Slider.svelte";
  import RepeatButton from "../components/shell/RepeatButton.svelte";
  import Empty from "./library/Empty.svelte";

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

  let volume = $state(player.state.volume);
  $effect(() => {
    volume = player.state.volume;
  });

  const song = $derived(player.song);
  const next = $derived(player.nextSong);
  const upcoming = $derived(player.state.upcoming);
  const overlap = $derived(upcoming && upcoming.overlap > 0 ? { from: upcoming.cue, to: upcoming.cue + upcoming.overlap } : null);
  const lane = $derived(
    song && next && upcoming ? { left: ((upcoming.cue - upcoming.start) / song.duration) * 100, width: (next.duration / song.duration) * 100 } : null,
  );
  const recent = $derived(library.albums.toSorted((a, b) => Math.max(...b.songs.map((s) => s.added)) - Math.max(...a.songs.map((s) => s.added))).slice(0, 5));

  function edit() {
    if (song) ui.go({ view: "edit", song: song.id, next: next?.id ?? null, context: player.current?.context ?? { type: "queue" } });
  }
</script>

{#if library.ready && library.songs.length === 0}
  <Empty />
{:else if !song}
  <div class="flex h-full flex-col items-center justify-center gap-8 pb-16">
    <h1 class="display text-[40px] leading-none font-semibold">Nothing playing</h1>
    {#if recent.length}
      <div class="flex gap-5">
        {#each recent as album (album.key)}
          <button type="button" class="group flex w-[112px] flex-col items-center gap-2" title={album.title} onclick={() => player.play(album.songs.map((entry) => entry.id), 0, { type: "album", key: album.key })}>
            <Cover cover={album.cover} label={album.title} size={112} class="transition-transform duration-300 ease-out-soft group-hover:-translate-y-1" />
            <span class="w-full truncate text-center text-[12px] text-paper-3 group-hover:text-paper-2">{album.title}</span>
          </button>
        {/each}
      </div>
    {/if}
    <div class="flex gap-2">
      <Button kind="primary" icon="play" onclick={() => player.play(library.songs.map((entry) => entry.id), 0, { type: "songs" })}>Play everything</Button>
      <Button onclick={() => ui.go({ view: "songs" })}>Open the library</Button>
    </div>
  </div>
{:else}
  <div class="flex h-full flex-col items-center px-12 pb-8">
    <div class="flex min-h-0 w-full flex-1 flex-col items-center justify-center gap-7">
      {#key song.id}
      <div class="aspect-square h-[min(40vh,400px)] min-h-32" in:scale={{ start: 0.94, duration: motion(420), easing: cubicOut }}>
        <Cover cover={song.cover} label={title(song)} size="fill" large class="rounded-2xl! shadow-[0_32px_90px_rgb(0_0_0/0.6)]" />
      </div>
      {/key}
      {#key song.id}
      <div class="flex max-w-[min(760px,90%)] flex-col items-center text-center" in:fly={{ y: 8, duration: motion(360), delay: motion(60), easing: cubicOut }}>
        <h1 class="display w-full truncate text-[clamp(30px,3.6vw,48px)] leading-[1.05] font-semibold">{title(song)}</h1>
        <p class="mt-2 truncate text-[16px] text-paper-2">
          <button type="button" class="hover:text-paper" onclick={() => ui.go({ view: "artist", name: artist(song) })}>{artist(song)}</button>
          {#if song.album}<span class="mx-1.5 text-paper-4">·</span><span class="text-paper-3">{song.album}</span>{/if}
        </p>
      </div>
      {/key}
      <div class="grid w-full max-w-[560px] grid-cols-[1fr_auto_1fr] items-center gap-3">
        <div class="flex items-center justify-end gap-1">
          <RepeatButton class="size-10 hover:bg-ink-3" />
          <button type="button" aria-label="Edit song" title="Edit song" class="flex size-10 items-center justify-center rounded-full text-paper-3 hover:bg-ink-3 hover:text-paper" onclick={edit}>
            <Icon name="edit" size={19} />
          </button>
          <button type="button" aria-label="Queue" title="Queue" class="flex size-10 items-center justify-center rounded-full hover:bg-ink-3 {ui.queueOpen ? 'text-accent' : 'text-paper-3 hover:text-paper'}" onclick={() => (ui.queueOpen = !ui.queueOpen)}>
            <Icon name="queue" size={19} />
          </button>
          <button type="button" aria-label="Previous" class="flex size-11 items-center justify-center rounded-full text-paper-2 hover:text-paper" onclick={() => player.previous()}>
            <Icon name="previous" size={22} />
          </button>
        </div>
        <button type="button" aria-label={player.state.playing ? "Pause" : "Play"} class="flex size-16 items-center justify-center rounded-full bg-paper text-ink shadow-[0_10px_30px_rgb(0_0_0/0.4)] transition-transform duration-150 hover:scale-105 active:scale-95" onclick={() => player.toggle()}>
          <Icon name={player.state.playing ? "pause" : "play"} size={26} />
        </button>
        <div class="flex items-center gap-1">
          <button type="button" aria-label="Next" disabled={!player.following} class="flex size-11 items-center justify-center rounded-full text-paper-2 hover:text-paper disabled:opacity-30" onclick={() => player.next()}>
            <Icon name="next" size={22} />
          </button>
          <button type="button" aria-label="Visualizer" title="Visualizer" class="flex size-10 items-center justify-center rounded-full text-paper-3 hover:bg-ink-3 hover:text-paper" onclick={() => (ui.visualizer = true)}>
            <Icon name="visual" size={19} />
          </button>
          <div class="flex w-24 items-center gap-2 pl-1 text-paper-3">
            <Slider bind:value={volume} label="Volume" onchange={(value) => player.volume(value)} />
          </div>
        </div>
      </div>
    </div>

    <section class="w-full max-w-[1180px] shrink-0" aria-label="This song and the next">
      <div class="mb-2 grid grid-cols-[1fr_auto_1fr] items-baseline text-[12px]">
        <span class="numeric text-paper-2">{clock(now)}</span>
        {#if upcoming && next}
          <button
            type="button"
            class="flex items-center gap-2 rounded-full px-2.5 py-1 text-paper-3 transition-colors hover:bg-ink-3 hover:text-paper-2"
            onclick={() => (ui.seam = { from: song.id, to: next.id, context: player.current?.context ?? { type: "queue" } })}
          >
            <TransitionGlyph kind={upcoming.kind} width={26} />
            {info(upcoming.kind).label} into <span class="text-paper-2">{title(next)}</span>{upcoming.overlap > 0 ? ` at ${clock(upcoming.cue)}` : ""}
          </button>
        {:else}
          <span class="text-paper-4">The queue ends here</span>
        {/if}
        <span class="numeric text-right text-paper-3">{clock(song.duration)}</span>
      </div>
      <Waveform song={song.id} duration={song.duration} position={now} start={player.state.start} end={player.state.end} {overlap} overlapKind={upcoming?.kind} height={64} bar={3} onseek={(seconds) => player.seek(seconds)} />
      <div class="relative mt-1 h-7 overflow-hidden" aria-hidden="true">
        {#if next && lane}
          <div class="absolute inset-y-0 opacity-60" style="left:{lane.left}%;width:{lane.width}%">
            <Waveform song={next.id} duration={next.duration} position={0} start={upcoming?.start ?? 0} height={28} bar={3} />
          </div>
        {/if}
      </div>
    </section>
  </div>
{/if}
