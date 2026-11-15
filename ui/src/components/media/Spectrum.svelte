<script lang="ts">
  import { BANDS, reducedMotion, visual } from "../../lib/stores/visual.svelte";

  let { bars = 20, height = 22, class: className = "" }: { bars?: number; height?: number; class?: string } = $props();

  let canvas: HTMLCanvasElement;

  $effect(() => {
    const stop = visual.watch();
    const context = canvas.getContext("2d")!;
    const ratio = window.devicePixelRatio || 1;
    const width = bars * 4;
    canvas.width = width * ratio;
    canvas.height = height * ratio;
    context.scale(ratio, ratio);
    const levels = new Float32Array(bars);
    let frame = 0;
    const draw = () => {
      frame = requestAnimationFrame(draw);
      if (reducedMotion.matches) return;
      context.clearRect(0, 0, width, height);
      context.fillStyle = getComputedStyle(canvas).color;
      for (let i = 0; i < bars; i++) {
        const from = Math.floor((i / bars) * BANDS * 0.85);
        const to = Math.max(from + 1, Math.floor(((i + 1) / bars) * BANDS * 0.85));
        let level = 0;
        for (let band = from; band < to; band++) level = Math.max(level, visual.band(band));
        levels[i] = Math.max(level, levels[i]! * 0.82);
        const bar = Math.max(1.5, levels[i]! * height);
        context.fillRect(i * 4, height - bar, 2.5, bar);
      }
    };
    draw();
    return () => {
      cancelAnimationFrame(frame);
      stop();
    };
  });
</script>

<canvas bind:this={canvas} class="text-accent {className}" style="width:{bars * 4}px;height:{height}px" aria-hidden="true"></canvas>
