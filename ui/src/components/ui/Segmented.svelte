<script lang="ts" generics="T">
  let { value = $bindable(), options, label, onchange }: {
    value: T;
    options: { value: T; label: string }[];
    label: string;
    onchange?: (value: T) => void;
  } = $props();
</script>

<div role="radiogroup" aria-label={label} class="inline-flex rounded-lg bg-ink-3 p-[3px]">
  {#each options as option, index (index)}
    <button
      type="button"
      role="radio"
      aria-checked={value === option.value}
      class="rounded-md px-3 py-1 text-[13px] transition-colors {value === option.value ? 'bg-ink-5 text-paper' : 'text-paper-3 hover:text-paper-2'}"
      onclick={() => {
        value = option.value;
        onchange?.(option.value);
      }}
    >
      {option.label}
    </button>
  {/each}
</div>
