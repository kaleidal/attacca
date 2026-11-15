<script lang="ts">
  import { cubicOut } from "svelte/easing";
  import { fly, scale } from "svelte/transition";
  import { motion } from "../../lib/motion";
  import { artist, title } from "../../lib/format";
  import { player } from "../../lib/stores/player.svelte";
  import { ui } from "../../lib/stores/ui.svelte";
  import Cover from "../media/Cover.svelte";
  import Icon from "../ui/Icon.svelte";

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
  const progress = $derived(song ? Math.min(1, now / song.duration) : 0);
  const cue = $derived(song && player.state.upcoming ? player.state.upcoming.cue / song.duration : null);
</script>

{#if song}
  <div
    transition:fly={{ y: 20, duration: motion(260), easing: cubicOut }}
    class="fixed bottom-5 left-1/2 z-40 flex h-[60px] w-[min(640px,calc(100%-48px))] -translate-x-1/2 items-center gap-3 overflow-hidden rounded-[20px] bg-ink-3/95 pr-2 pl-2 shadow-[0_18px_50px_rgb(0_0_0/0.55)] backdrop-blur-md"
  >
    <button type="button" class="flex min-w-0 flex-1 items-center gap-3 text-left" onclick={() => ui.go({ view: "listen" })}>
      {#key song.id}
      <div in:scale={{ start: 0.85, duration: motion(260), easing: cubicOut }}><Cover cover={song.cover} label={title(song)} size={44} class="rounded-xl!" /></div>
      {/key}
      <div class="min-w-0">
        <div class="truncate text-[14px] font-medium">{title(song)}</div>
        <div class="truncate text-[12px] text-paper-3">{artist(song)}</div>
      </div>
    </button>
    <div class="flex items-center">
      <button type="button" aria-label="Previous" class="flex size-9 items-center justify-center rounded-full text-paper-2 hover:text-paper" onclick={() => player.previous()}>
        <Icon name="previous" size={17} />
      </button>
      <button type="button" aria-label={player.state.playing ? "Pause" : "Play"} class="flex size-10 items-center justify-center rounded-full bg-paper text-ink transition-transform duration-150 hover:scale-105 active:scale-95" onclick={() => player.toggle()}>
        <Icon name={player.state.playing ? "pause" : "play"} size={17} />
      </button>
      <button type="button" aria-label="Next" disabled={!player.following} class="flex size-9 items-center justify-center rounded-full text-paper-2 hover:text-paper disabled:opacity-30" onclick={() => player.next()}>
        <Icon name="next" size={17} />
      </button>
    </div>
    <button type="button" aria-label="Queue" class="flex size-9 items-center justify-center rounded-full hover:bg-ink-4 {ui.queueOpen ? 'text-accent' : 'text-paper-2'}" onclick={() => (ui.queueOpen = !ui.queueOpen)}>
      <Icon name="queue" size={17} />
    </button>
    <div class="absolute inset-x-5 bottom-0 h-[2px] bg-ink-5" aria-hidden="true">
      <div class="h-full bg-accent" style="width:{progress * 100}%"></div>
      {#if cue !== null}<div class="absolute -top-[3px] h-2 w-[2px] rounded-full bg-paper-2" style="left:{cue * 100}%"></div>{/if}
    </div>
  </div>
{/if}
