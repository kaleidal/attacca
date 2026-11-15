const remembered = new Map<string, { left: number; width: number }>();

/**
 * Follows the active item in a row of buttons, so a highlight can glide between them. Rows that
 * are rebuilt on every switch pass a name, and their highlight glides from where it last was.
 */
export class Indicator {
  left = $state(0);
  width = $state(0);
  ready = $state(false);
  items: HTMLElement[] = $state([]);

  constructor(active: () => number, name?: string) {
    const last = name ? remembered.get(name) : undefined;
    if (last) {
      this.left = last.left;
      this.width = last.width;
    }
    $effect(() => {
      const element = this.items[active()];
      if (!element) return;
      const measure = () => {
        this.left = element.offsetLeft;
        this.width = element.offsetWidth;
        if (name) remembered.set(name, { left: this.left, width: this.width });
      };
      let frame = requestAnimationFrame(() => {
        this.ready = true;
        frame = requestAnimationFrame(measure);
      });
      if (!last && !this.ready) measure();
      const observer = new ResizeObserver(measure);
      observer.observe(element);
      return () => {
        cancelAnimationFrame(frame);
        observer.disconnect();
      };
    });
  }
}
