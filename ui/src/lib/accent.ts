/** Picks a lively colour from a cover to tint the interface while it plays. */
export async function accentFrom(url: string): Promise<{ accent: string; onAccent: string } | null> {
  const image = new Image();
  image.decoding = "async";
  image.src = url;
  try {
    await image.decode();
  } catch {
    return null;
  }
  const size = 32;
  const canvas = new OffscreenCanvas(size, size);
  const context = canvas.getContext("2d", { willReadFrequently: true });
  if (!context) return null;
  context.drawImage(image, 0, 0, size, size);
  const pixels = context.getImageData(0, 0, size, size).data;
  let best = { score: -1, h: 20, s: 0.8, l: 0.62 };
  for (let i = 0; i < pixels.length; i += 4) {
    const [h, s, l] = hsl(pixels[i]! / 255, pixels[i + 1]! / 255, pixels[i + 2]! / 255);
    const score = s * (1 - Math.abs(l - 0.55) * 1.6);
    if (score > best.score) best = { score, h, s, l };
  }
  if (best.score < 0.12) return null;
  const saturation = Math.min(0.85, Math.max(0.45, best.s));
  const accent = `hsl(${Math.round(best.h)} ${Math.round(saturation * 100)}% 66%)`;
  return { accent, onAccent: `hsl(${Math.round(best.h)} 40% 9%)` };
}

function hsl(r: number, g: number, b: number): [number, number, number] {
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const l = (max + min) / 2;
  if (max === min) return [0, 0, l];
  const d = max - min;
  const s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
  let h = max === r ? (g - b) / d + (g < b ? 6 : 0) : max === g ? (b - r) / d + 2 : (r - g) / d + 4;
  h *= 60;
  return [h, s, l];
}
