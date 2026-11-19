<script lang="ts" module>
  const paths = {
    play: "M6.5 4.2v11.6c0 .6.7 1 1.2.6l8.4-5.8c.4-.3.4-1 0-1.3L7.7 3.6c-.5-.4-1.2 0-1.2.6Z",
    pause: "M6 4.5h2.4v11H6zM11.6 4.5H14v11h-2.4z",
    next: "M4.5 4.8v10.4c0 .5.6.8 1 .5l7.2-5.2c.3-.3.3-.8 0-1L5.5 4.3c-.4-.3-1 0-1 .5ZM15 4.5v11",
    previous: "M15.5 4.8v10.4c0 .5-.6.8-1 .5L7.3 10.5c-.3-.3-.3-.8 0-1l7.2-5.2c.4-.3 1 0 1 .5ZM5 4.5v11",
    queue: "M3.5 5.5h13M3.5 10h13M3.5 14.5h7M14.5 12.5v4l3-2z",
    volume: "M3.5 8v4h3l4 3.5v-11l-4 3.5zM13.5 7.3a4 4 0 0 1 0 5.4M15.6 5.2a7 7 0 0 1 0 9.6",
    mute: "M3.5 8v4h3l4 3.5v-11l-4 3.5zM13.5 8l4 4M17.5 8l-4 4",
    visual: "M3.5 16V11M7.2 16V6M10.9 16V9M14.6 16V4M18.3 16v-6",
    search: "M8.8 15a6.2 6.2 0 1 0 0-12.4 6.2 6.2 0 0 0 0 12.4ZM13.3 13.3l4.2 4.2",
    plus: "M10 4v12M4 10h12",
    back: "M12.5 4.5 7 10l5.5 5.5",
    forward: "M7.5 4.5 13 10l-5.5 5.5",
    more: "M5 10h.01M10 10h.01M15 10h.01",
    edit: "M3 14V6M17 14V6M6.5 12V8M10 15V5M13.5 12V8",
    share: "M10 12.5V3.5M6.5 6.5 10 3l3.5 3.5M5 9.5H4v7h12v-7h-1",
    folder: "M3 6.2c0-.7.5-1.2 1.2-1.2h3.6l1.8 2h6.2c.7 0 1.2.5 1.2 1.2v6.6c0 .7-.5 1.2-1.2 1.2H4.2c-.7 0-1.2-.5-1.2-1.2z",
    sound: "M5 3.5v13M10 3.5v13M15 3.5v13M3.2 12h3.6M8.2 7h3.6M13.2 10.5h3.6",
    songs: "M7.5 14.5V4.8l9-1.8v9.6M7.5 14.5a2 2 0 1 1-4 0 2 2 0 0 1 4 0ZM16.5 12.6a2 2 0 1 1-4 0 2 2 0 0 1 4 0Z",
    albums: "M10 17a7 7 0 1 0 0-14 7 7 0 0 0 0 14ZM10 11.5a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3Z",
    artists: "M10 9.5a3.2 3.2 0 1 0 0-6.4 3.2 3.2 0 0 0 0 6.4ZM4 16.8c.6-3 3-4.6 6-4.6s5.4 1.6 6 4.6",
    recent: "M10 17a7 7 0 1 0 0-14 7 7 0 0 0 0 14ZM10 6v4l2.8 1.8",
    playlist: "M3.5 5h9M3.5 9h9M3.5 13h5M15 11.5v5M12.5 14h5",
    close: "M5 5l10 10M15 5L5 15",
    check: "M4.5 10.5l3.5 3.5 7.5-8",
    grip: "M7.5 6h.01M7.5 10h.01M7.5 14h.01M12.5 6h.01M12.5 10h.01M12.5 14h.01",
    minimize: "M4.5 10h11",
    maximize: "M6.5 4.5h7a2 2 0 0 1 2 2v7a2 2 0 0 1-2 2h-7a2 2 0 0 1-2-2v-7a2 2 0 0 1 2-2Z",
    restore: "M7.5 4.5h6a2 2 0 0 1 2 2v6M6.5 7.5h5a2 2 0 0 1 2 2v4a2 2 0 0 1-2 2h-5a2 2 0 0 1-2-2v-4a2 2 0 0 1 2-2Z",
    image: "M4 4.5h12v11H4zM4 13l3.5-3.5 3 3 2-2 3.5 3.5M12.5 8a1 1 0 1 0 0-2 1 1 0 0 0 0 2Z",
    options: "M3.5 6.5h6M13.5 6.5h3M3.5 13.5h2M9.5 13.5h7M11.5 4.5v4M7.5 11.5v4",
    reset: "M4.5 9.5a5.5 5.5 0 1 1 1.6 3.9M4.5 14.5v-5h5",
    repeat: "M3.5 9.5V8a2.5 2.5 0 0 1 2.5-2.5h10M13.5 3l2.5 2.5L13.5 8M16.5 10.5V12a2.5 2.5 0 0 1-2.5 2.5H4M6.5 17 4 14.5 6.5 12",
    "repeat-one": "M3.5 9.5V8a2.5 2.5 0 0 1 2.5-2.5h10M13.5 3l2.5 2.5L13.5 8M16.5 10.5V12a2.5 2.5 0 0 1-2.5 2.5H4M6.5 17 4 14.5 6.5 12M9.2 9l1.3-.8v4",
    shuffle: "M3.5 6h3c3.5 0 3.5 8 7 8h3M14 11.5l2.5 2.5-2.5 2.5M3.5 14h3c1.3 0 2-.9 2.6-2.1M11.4 8.1c.6-1.2 1.3-2.1 2.6-2.1h2.5M14 3.5 16.5 6 14 8.5",
  } as const;

  export type IconName = keyof typeof paths;
</script>

<script lang="ts">
  let { name, size = 20, class: className = "" }: { name: IconName; size?: number; class?: string } = $props();
  const filled = $derived(name === "play" || name === "pause");
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 20 20"
  fill={filled ? "currentColor" : "none"}
  stroke={filled ? "none" : "currentColor"}
  stroke-width="1.6"
  stroke-linecap="round"
  stroke-linejoin="round"
  class="shrink-0 {className}"
  aria-hidden="true"
>
  <path d={paths[name]} />
</svg>
