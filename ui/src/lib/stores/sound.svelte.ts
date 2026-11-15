import { commands, events, type DeviceInfo, type SoundSettings } from "../bindings";

class SoundStore {
  settings = $state<SoundSettings | null>(null);
  devices = $state<DeviceInfo[]>([]);
  #saving: ReturnType<typeof setTimeout> | null = null;

  async start() {
    const state = await commands.soundGet();
    this.settings = state.settings;
    this.devices = state.devices;
    events.soundDevices((devices) => (this.devices = devices));
  }

  /** Applies a change at once and saves it shortly after the last one. */
  change(update: (settings: SoundSettings) => void) {
    if (!this.settings) return;
    update(this.settings);
    if (this.#saving) clearTimeout(this.#saving);
    const settings = $state.snapshot(this.settings);
    this.#saving = setTimeout(() => void commands.soundSet({ settings }), 60);
  }
}

export const sound = new SoundStore();
