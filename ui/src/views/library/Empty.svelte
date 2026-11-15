<script lang="ts">
  import { library } from "../../lib/stores/library.svelte";
  import Button from "../../components/ui/Button.svelte";
</script>

<div class="flex h-full flex-col items-center justify-center gap-5 pb-16 text-center">
  <h1 class="display text-[34px] leading-tight font-semibold">Your music, your endings.</h1>
  <p class="max-w-sm text-[14px] text-paper-3">Point Attacca at the folders where your music lives. Nothing leaves this computer.</p>
  <Button kind="primary" icon="folder" onclick={() => library.addFolder()}>Add a folder</Button>
</div>
