import type { Context } from "../bindings";

export type Route =
  | { view: "songs" }
  | { view: "recent" }
  | { view: "albums" }
  | { view: "album"; key: string }
  | { view: "artists" }
  | { view: "artist"; name: string }
  | { view: "playlists" }
  | { view: "playlist"; id: number }
  | { view: "sound" }
  | { view: "now" }
  | { view: "edit"; song: string; next: string | null; context: Context }
  | { view: "search" };

export type CoverTarget = { type: "song"; id: string } | { type: "album"; key: string } | { type: "playlist"; id: number };

export interface MenuItem {
  label: string;
  action: () => void;
  danger?: boolean;
}

export interface Toast {
  id: number;
  message: string;
}

class UiStore {
  route = $state<Route>({ view: "songs" });
  queueOpen = $state(false);
  visualizer = $state(false);
  search = $state("");
  coverTarget = $state<CoverTarget | null>(null);
  importing = $state<string | null>(null);
  menu = $state<{ x: number; y: number; items: MenuItem[] } | null>(null);
  toasts = $state<Toast[]>([]);
  #history: Route[] = [];
  #toastId = 0;

  go(route: Route) {
    if (JSON.stringify(route) === JSON.stringify(this.route)) return;
    this.#history.push(this.route);
    if (this.#history.length > 40) this.#history.shift();
    this.route = route;
  }

  back() {
    this.route = this.#history.pop() ?? { view: "songs" };
  }

  get canGoBack() {
    return this.#history.length > 0;
  }

  openMenu(event: MouseEvent, items: MenuItem[]) {
    event.preventDefault();
    this.menu = { x: event.clientX, y: event.clientY, items };
  }

  toast(message: string) {
    const id = ++this.#toastId;
    this.toasts = [...this.toasts, { id, message }];
    setTimeout(() => (this.toasts = this.toasts.filter((toast) => toast.id !== id)), 4200);
  }
}

export const ui = new UiStore();
