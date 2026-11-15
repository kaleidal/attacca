<script lang="ts">
  let { checked = $bindable(), label, onchange }: { checked: boolean; label: string; onchange?: (checked: boolean) => void } = $props();
</script>

<button
  type="button"
  role="switch"
  aria-checked={checked}
  aria-label={label}
  class="relative h-[22px] w-[38px] shrink-0 rounded-full transition-colors duration-200 {checked ? 'bg-accent' : 'bg-ink-5'}"
  onclick={() => {
    checked = !checked;
    onchange?.(checked);
  }}
>
  <span
    class="absolute top-[3px] left-[3px] size-4 rounded-full transition-transform duration-200 ease-out-soft {checked ? 'translate-x-4 bg-on-accent' : 'bg-paper-2'}"
  ></span>
</button>
