import { fileUrl, invoke } from "@lantharos/sabine";

let coversFolder = "";

export function setCoversFolder(folder: string) {
  coversFolder = folder;
}

export function coverUrl(cover: string | null | undefined, small = true): string | null {
  if (!cover || !coversFolder) return null;
  return fileUrl(`${coversFolder}/${cover}${small ? "-small" : ""}.jpg`);
}

const waveforms = new Map<string, Promise<Uint8Array>>();

/** Peak and RMS per fiftieth of a second, two bytes each. */
export function waveform(song: string): Promise<Uint8Array> {
  let found = waveforms.get(song);
  if (!found) {
    found = invoke<Uint8Array>("library.waveform", { id: song });
    waveforms.set(song, found);
  }
  return found;
}

export function forgetWaveform(song: string) {
  waveforms.delete(song);
}

export const WAVEFORM_RATE = 50;
