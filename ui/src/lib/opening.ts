import { app, events } from "@lantharos/sabine";
import { commands } from "./bindings";
import { ui } from "./stores/ui.svelte";

/** Turns a file URL the desktop handed over into a path. */
function pathOf(url: string): string | null {
  if (!url.startsWith("file://")) return null;
  return decodeURIComponent(new URL(url).pathname);
}

export function open(paths: string[]) {
  const shared = paths.find((path) => path.toLowerCase().endsWith(".attacca"));
  if (shared) ui.importing = shared;
  const audio = paths.filter((path) => !path.toLowerCase().endsWith(".attacca"));
  if (audio.length) void commands.libraryOpenFiles({ paths: audio });
}

async function take() {
  const paths = (await app.takeOpenUrls()).map(pathOf).filter((path): path is string => path !== null);
  if (paths.length) open(paths);
}

/** Opens the songs and shared playlists people hand the app, now and later. */
export function followOpenedFiles() {
  events.openUrlsAvailable(() => void take());
  void take();
}
