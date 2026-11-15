<script lang="ts">
  let {
    value = $bindable(),
    min = 0,
    max = 1,
    step = 0,
    vertical = false,
    centered = false,
    label,
    onchange,
    oncommit,
    class: className = "",
  }: {
    value: number;
    min?: number;
    max?: number;
    step?: number;
    vertical?: boolean;
    centered?: boolean;
    label: string;
    onchange?: (value: number) => void;
    oncommit?: (value: number) => void;
    class?: string;
  } = $props();

  let track: HTMLDivElement;
  let dragging = $state(false);
  const fraction = $derived((value - min) / (max - min));
  const origin = $derived(centered ? (0 - min) / (max - min) : 0);

  function set(next: number) {
    const stepped = step > 0 ? Math.round(next / step) * step : next;
    const clamped = Math.min(max, Math.max(min, stepped));
    if (clamped !== value) {
      value = clamped;
      onchange?.(clamped);
    }
  }

  function at(event: PointerEvent) {
    const rect = track.getBoundingClientRect();
    const t = vertical ? 1 - (event.clientY - rect.top) / rect.height : (event.clientX - rect.left) / rect.width;
    set(min + Math.min(1, Math.max(0, t)) * (max - min));
  }

  function down(event: PointerEvent) {
    track.setPointerCapture(event.pointerId);
    dragging = true;
    at(event);
  }

  function up() {
    if (!dragging) return;
    dragging = false;
    oncommit?.(value);
  }

  function key(event: KeyboardEvent) {
    const amount = (step || (max - min) / 100) * (event.shiftKey ? 10 : 1);
    const keys: Record<string, number> = { ArrowUp: amount, ArrowRight: amount, ArrowDown: -amount, ArrowLeft: -amount };
    if (event.key in keys) {
      event.preventDefault();
      set(value + keys[event.key]!);
      oncommit?.(value);
    }
  }

  function reset() {
    if (!centered) return;
    set(0);
    oncommit?.(0);
  }
</script>

<div
  bind:this={track}
  role="slider"
  tabindex="0"
  aria-label={label}
  aria-valuemin={min}
  aria-valuemax={max}
  aria-valuenow={value}
  aria-orientation={vertical ? "vertical" : "horizontal"}
  class="group relative touch-none {vertical ? 'h-full w-5' : 'h-5 w-full'} {className}"
  onpointerdown={down}
  onpointermove={(event) => dragging && at(event)}
  onpointerup={up}
  onpointercancel={up}
  onkeydown={key}
  ondblclick={reset}
>
  <div class="absolute rounded-full bg-ink-5 {vertical ? 'inset-y-0 left-1/2 w-[3px] -translate-x-1/2' : 'inset-x-0 top-1/2 h-[3px] -translate-y-1/2'}"></div>
  <div
    class="absolute rounded-full bg-paper-2 transition-colors group-hover:bg-paper {vertical ? 'left-1/2 w-[3px] -translate-x-1/2' : 'top-1/2 h-[3px] -translate-y-1/2'}"
    style={vertical
      ? `bottom:${Math.min(origin, fraction) * 100}%;height:${Math.abs(fraction - origin) * 100}%`
      : `left:${Math.min(origin, fraction) * 100}%;width:${Math.abs(fraction - origin) * 100}%`}
  ></div>
  <div
    class="absolute size-3 rounded-full bg-paper shadow-[0_1px_4px_rgb(0_0_0/0.5)] transition-transform {dragging ? 'scale-125' : 'scale-0 group-hover:scale-100 group-focus-visible:scale-100'} {vertical ? 'left-1/2 -translate-x-1/2 translate-y-1/2' : 'top-1/2 -translate-x-1/2 -translate-y-1/2'}"
    style={vertical ? `bottom:${fraction * 100}%` : `left:${fraction * 100}%`}
  ></div>
</div>
