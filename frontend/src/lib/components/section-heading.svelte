<script lang="ts">
  // The heading of a section of a page: a corner tick, the title in
  // capitals over a hairline with an accent segment, and its actions on
  // the right.
  import type { Snippet } from 'svelte';

  let {
    id,
    title,
    description,
    actions,
  }: { id?: string; title: string; description?: string; actions?: Snippet } = $props();
</script>

<div class="relative mb-5 flex flex-wrap items-end justify-between gap-3 pb-2.5">
  <div class="min-w-0">
    <h2
      {id}
      class="relative pl-3.5 text-base font-semibold tracking-[0.14em] uppercase before:absolute before:top-0.5 before:left-0 before:size-2 before:border-t-2 before:border-l-2 before:border-primary"
    >
      {title}
    </h2>
    {#if description}
      <p class="mt-1 pl-3.5 text-sm text-muted-foreground">{description}</p>
    {/if}
  </div>
  {#if actions}
    <div class="flex flex-wrap items-center gap-2">{@render actions()}</div>
  {/if}
  <span aria-hidden="true" class="absolute inset-x-0 bottom-0 h-px bg-border"></span>
  <span aria-hidden="true" class="absolute bottom-0 left-0 h-px w-16 bg-primary"></span>
</div>
