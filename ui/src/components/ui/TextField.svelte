<script lang="ts">
  let {
    value = $bindable(),
    placeholder = "",
    label,
    autofocus = false,
    onsubmit,
    oninput,
    class: className = "",
  }: {
    value: string;
    placeholder?: string;
    label: string;
    autofocus?: boolean;
    onsubmit?: (value: string) => void;
    oninput?: (value: string) => void;
    class?: string;
  } = $props();

  let input: HTMLInputElement;
  $effect(() => {
    if (autofocus) input.focus();
  });
</script>

<input
  bind:this={input}
  bind:value
  aria-label={label}
  {placeholder}
  spellcheck="false"
  class="h-9 w-full rounded-lg bg-ink-3 px-3 text-[13px] text-paper outline-none placeholder:text-paper-4 focus:bg-ink-4 {className}"
  oninput={() => oninput?.(value)}
  onkeydown={(event) => event.key === "Enter" && onsubmit?.(value)}
/>
