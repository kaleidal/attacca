import { shell } from "@lantharos/sabine";
import { commands, type Context, type Song } from "./bindings";
import { title } from "./format";
import { library } from "./stores/library.svelte";
import { ui, type MenuItem } from "./stores/ui.svelte";

/** What a song's menu offers, wherever the song is listed. */
export function songMenu(songs: Song[], context: Context, next: Song | null, extra: MenuItem[] = []): MenuItem[] {
  const ids = songs.map((song) => song.id);
  const first = songs[0]!;
  const items: MenuItem[] = [
    { label: "Play next", action: () => void commands.playerPlayNext({ songs: ids }) },
    { label: "Add to queue", action: () => void commands.playerEnqueue({ songs: ids }) },
  ];
  for (const playlist of library.playlists.slice(0, 8)) {
    items.push({ label: `Add to ${playlist.name}`, action: () => void commands.playlistsAdd({ id: playlist.id, songs: ids }) });
  }
  items.push({
    label: "New playlist from this",
    action: async () => {
      const id = await commands.playlistsCreate({ name: songs.length === 1 ? title(first) : "New playlist", songs: ids });
      ui.go({ view: "playlist", id });
    },
  });
  if (songs.length === 1) {
    items.push({ label: "Edit song", action: () => ui.go({ view: "edit", song: first.id, next: next?.id ?? null, context }) });
    items.push({ label: "Change cover", action: () => (ui.coverTarget = { type: "song", id: first.id }) });
    items.push({
      label: "Show in folder",
      action: async () => {
        const detail = await commands.libraryDetail({ id: first.id });
        if (detail?.path) await shell.showFolder(detail.path.slice(0, detail.path.lastIndexOf("/")));
      },
    });
  }
  return [...items, ...extra];
}
