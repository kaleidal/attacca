const THRESHOLD = 5;
const EDGE = 80;
const MAX_SPEED = 1100;
const CAPSULE_ROOM = 92;

interface Span {
  top: number;
  bottom: number;
}

/** Moves a row of a list with the pointer, scrolling near the edges, and drops it between rows or after the last. */
export class Reorder {
  dragging = $state<number | null>(null);
  lift = $state(0);
  /** Where the dragged row would land, or null while it would stay put. */
  landing = $state<number | null>(null);
  marker = $state(0);

  #list: () => HTMLElement | undefined;
  #rows: () => HTMLElement[];
  #onmove: (from: number, to: number) => void;
  #pressed: { index: number; x: number; y: number } | null = null;
  #spans: Span[] = [];
  #origin = 0;
  #pointer = 0;
  #scroller: HTMLElement | null = null;
  #frame = 0;
  #last = 0;

  constructor(list: () => HTMLElement | undefined, rows: () => HTMLElement[], onmove: (from: number, to: number) => void) {
    this.#list = list;
    this.#rows = rows;
    this.#onmove = onmove;
  }

  press(event: PointerEvent, index: number) {
    if (event.button !== 0) return;
    this.#pressed = { index, x: event.clientX, y: event.clientY };
    window.addEventListener("pointermove", this.#moved);
    window.addEventListener("pointerup", this.#released);
    window.addEventListener("pointercancel", this.#cancel);
    window.addEventListener("keydown", this.#key);
  }

  #moved = (event: PointerEvent) => {
    this.#pointer = event.clientY;
    if (this.dragging === null) {
      const pressed = this.#pressed;
      if (!pressed || Math.hypot(event.clientX - pressed.x, event.clientY - pressed.y) < THRESHOLD) return;
      this.#begin(pressed.index);
    }
    this.#follow();
  };

  #begin(index: number) {
    const list = this.#list()!;
    const top = list.getBoundingClientRect().top;
    this.#spans = this.#rows().map((row) => {
      const box = row.getBoundingClientRect();
      return { top: box.top - top, bottom: box.bottom - top };
    });
    this.#origin = this.#pointer - top;
    this.#scroller = list.closest<HTMLElement>(".scroll");
    this.dragging = index;
    this.#last = performance.now();
    this.#frame = requestAnimationFrame(this.#scroll);
  }

  #follow() {
    const from = this.dragging!;
    const y = this.#pointer - this.#list()!.getBoundingClientRect().top;
    this.lift = y - this.#origin;
    const before = this.#spans.filter((span) => (span.top + span.bottom) / 2 < y).length;
    const to = before > from ? before - 1 : before;
    this.landing = to === from ? null : to;
    const spans = this.#spans;
    if (before === 0) this.marker = spans[0]!.top - 2;
    else if (before === spans.length) this.marker = spans.at(-1)!.bottom + 2;
    else this.marker = (spans[before - 1]!.bottom + spans[before]!.top) / 2;
  }

  #scroll = (now: number) => {
    const scroller = this.#scroller;
    const seconds = Math.min(0.05, (now - this.#last) / 1000);
    this.#last = now;
    if (scroller) {
      const box = scroller.getBoundingClientRect();
      const bottom = Math.min(box.bottom, window.innerHeight - CAPSULE_ROOM);
      const down = (this.#pointer - (bottom - EDGE)) / EDGE;
      const up = (box.top + EDGE - this.#pointer) / EDGE;
      const push = down > 0 ? Math.min(1, down) : up > 0 ? -Math.min(1, up) : 0;
      if (push !== 0) {
        scroller.scrollTop += push * Math.abs(push) * MAX_SPEED * seconds;
        this.#follow();
      }
    }
    this.#frame = requestAnimationFrame(this.#scroll);
  };

  #released = () => {
    const [from, to] = [this.dragging, this.landing];
    const dragged = from !== null;
    this.#end();
    if (dragged) {
      window.addEventListener("click", swallow, { capture: true, once: true });
      setTimeout(() => window.removeEventListener("click", swallow, { capture: true }));
    }
    if (from !== null && to !== null) this.#onmove(from, to);
  };

  #cancel = () => this.#end();

  #key = (event: KeyboardEvent) => {
    if (event.key === "Escape" && this.dragging !== null) this.#end();
  };

  #end() {
    cancelAnimationFrame(this.#frame);
    window.removeEventListener("pointermove", this.#moved);
    window.removeEventListener("pointerup", this.#released);
    window.removeEventListener("pointercancel", this.#cancel);
    window.removeEventListener("keydown", this.#key);
    this.#pressed = null;
    this.dragging = null;
    this.landing = null;
    this.lift = 0;
  }
}

function swallow(event: MouseEvent) {
  event.stopPropagation();
  event.preventDefault();
}
