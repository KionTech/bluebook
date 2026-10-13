<script lang="ts">
  import type { Snippet } from 'svelte';

  let { href, tabindex, children }: { href: string; tabindex?: number; children: Snippet } =
    $props();
</script>

<a
  {href}
  {tabindex}
  class="text-foreground underline decoration-neutral-300 underline-offset-4 transition-colors duration-300 ease-out hover:decoration-current!"
>
  {@render children()}
</a>
