<script lang="ts">
  import { commands, type Playlist, type Song, type Transition } from "../../lib/bindings";
  import { title } from "../../lib/format";
  import { info } from "../../lib/transitions";
  import Cover from "../../components/media/Cover.svelte";
  import TransitionGlyph from "../../components/media/TransitionGlyph.svelte";
  import TransitionPicker from "../../components/media/TransitionPicker.svelte";
  import Popover from "../../components/ui/Popover.svelte";

  let { playlist, songs, fallback }: { playlist: Playlist; songs: Song[]; fallback: Transition } = $props();

  function pair(from: Song, to: Song) {
    return playlist.pairs.find((entry) => entry.from === from.id && entry.to === to.id)?.transition ?? null;
  }

  /** The transition that will play: the pair's own, then the song's, then the playlist's. */
  function effective(from: Song, to: Song): { transition: Transition; own: boolean } {
    const own = pair(from, to);
    if (own) return { transition: own, own: true };
    return { transition: from.transition ?? fallback, own: false };
  }

  function set(from: Song, to: Song, transition: Transition | null) {
    void commands.playlistsSetPair({ id: playlist.id, from: from.id, to: to.id, transition });
  }
</script>

<section class="mb-8" aria-label="Transitions in this playlist">
  <div class="scroll-x -mx-2 flex items-center px-2 py-3">
    {#each songs as song, index (song.id + index)}
      <div class="shrink-0" title={title(song)}>
        <Cover cover={song.cover} label={title(song)} size={44} />
      </div>
      {#if songs[index + 1]}
        {@const next = songs[index + 1]!}
        {@const chosen = effective(song, next)}
        <Popover width={340}>
          {#snippet trigger(toggle)}
            <button
              type="button"
              aria-label="{info(chosen.transition.kind).label} into {title(next)}"
              class="mx-1 flex h-11 w-[58px] shrink-0 flex-col items-center justify-center gap-1 rounded-lg transition-colors hover:bg-ink-3 {chosen.own ? 'text-paper-2' : 'text-paper-4'}"
              onclick={toggle}
            >
              <TransitionGlyph kind={chosen.transition.kind} width={36} />
              {#if chosen.own}<span class="size-1 rounded-full bg-accent"></span>{/if}
            </button>
          {/snippet}
          {#snippet children()}
            <div class="mb-3 truncate text-[13px] text-paper-2">{title(song)} into {title(next)}</div>
            <TransitionPicker
              value={pair(song, next)}
              fallback={{ label: song.transition ? "The song's own" : "Playlist default", transition: song.transition ?? fallback }}
              onchange={(transition) => set(song, next, transition)}
            />
          {/snippet}
        </Popover>
      {/if}
    {/each}
  </div>
</section>
