<script lang="ts">
  import type { ClassValue } from 'svelte/elements';

  let { message, class: className }: { message?: string | null; class?: ClassValue } = $props();
</script>

<div class={className} hidden={!message}>
  <p class="text-sm text-red-600">{message}</p>
</div>
