import type { Song } from "./bindings";

export function clock(seconds: number): string {
  const total = Math.max(0, Math.floor(seconds));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const rest = String(total % 60).padStart(2, "0");
  return hours > 0 ? `${hours}:${String(minutes).padStart(2, "0")}:${rest}` : `${minutes}:${rest}`;
}

export function precise(seconds: number): string {
  const sign = seconds < 0 ? "-" : "";
  const value = Math.abs(seconds);
  const minutes = Math.floor(value / 60);
  const rest = (value - minutes * 60).toFixed(2).padStart(5, "0");
  return `${sign}${minutes}:${rest}`;
}

export function length(seconds: number): string {
  const minutes = Math.round(seconds / 60);
  if (minutes < 60) return `${minutes} min`;
  return `${Math.floor(minutes / 60)} h ${minutes % 60} min`;
}

export function title(song: Song): string {
  return song.title ?? song.name;
}

export function artist(song: Song): string {
  return song.artist ?? song.folder;
}

export function albumKey(song: Song): string | null {
  if (!song.album) return null;
  const by = (song.albumArtist ?? song.artist ?? "").toLowerCase();
  return `${by}\u001f${song.album.toLowerCase()}`;
}

export function decibels(value: number): string {
  const rounded = Math.round(value * 10) / 10;
  return `${rounded > 0 ? "+" : ""}${rounded.toFixed(1)} dB`;
}

export function hertz(value: number): string {
  return value >= 1000 ? `${(value / 1000).toFixed(value >= 10000 ? 0 : 1)} kHz` : `${Math.round(value)} Hz`;
}

export function count(value: number, one: string, many: string): string {
  return `${value} ${value === 1 ? one : many}`;
}

const collator = new Intl.Collator(undefined, { sensitivity: "base", numeric: true });

export function compare(a: string, b: string): number {
  return collator.compare(a, b);
}

export function matches(text: string | null | undefined, query: string): boolean {
  return !!text && text.toLowerCase().includes(query);
}
