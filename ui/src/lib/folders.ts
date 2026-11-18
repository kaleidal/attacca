import type { Song } from "./bindings";
import { compare, title } from "./format";

export interface Folder {
  path: string;
  name: string;
  folders: Folder[];
  songs: Song[];
  /** Every song in the folder and the ones inside it. */
  all: Song[];
  cover: string | null;
}

function parentOf(path: string): string {
  return path.slice(0, path.lastIndexOf("/")) || "/";
}

function nameOf(path: string): string {
  return path.slice(path.lastIndexOf("/") + 1) || path;
}

/** The library as the folders on disk hold it, starting from the folders it watches. */
export function folderTree(roots: string[], songs: Song[]): { roots: Folder[]; byPath: Map<string, Folder> } {
  const byPath = new Map<string, Folder>();
  const tops: Folder[] = [];
  const watched = roots.map((root) => root.replace(/\/+$/, "")).sort((a, b) => b.length - a.length);

  const folder = (path: string, top: string): Folder => {
    let found = byPath.get(path);
    if (found) return found;
    found = { path, name: nameOf(path), folders: [], songs: [], all: [], cover: null };
    byPath.set(path, found);
    if (path === top) tops.push(found);
    else folder(parentOf(path), top).folders.push(found);
    return found;
  };

  for (const song of songs) {
    if (!song.path) continue;
    const home = parentOf(song.path);
    const top = watched.find((root) => home === root || home.startsWith(`${root}/`)) ?? home;
    const holder = folder(home, top);
    holder.songs.push(song);
    for (let at: Folder | undefined = holder; at; at = at.path === top ? undefined : byPath.get(parentOf(at.path))) {
      at.all.push(song);
      at.cover ??= song.cover;
    }
  }

  for (const entry of byPath.values()) {
    entry.folders.sort((a, b) => compare(a.name, b.name));
    entry.songs.sort((a, b) => compare(a.path, b.path) || compare(title(a), title(b)));
  }
  tops.sort((a, b) => compare(a.path, b.path));
  return { roots: tops, byPath };
}
