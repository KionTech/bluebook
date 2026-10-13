<script lang="ts">
  import { Button } from '#lib/components/ui/button/index.ts';
  import { linkLabel, sitePath } from '#lib/pagination.ts';
  import type { PaginationLink } from '#lib/types.ts';

  let { links }: { links: PaginationLink[] } = $props();
</script>

{#if links.length > 3}
  <nav class="flex flex-wrap items-center justify-center gap-1" aria-label="Pagination">
    {#each links as link, index (index)}
      {#if link.url}
        <Button
          variant={link.active ? 'secondary' : 'ghost'}
          size="sm"
          class="min-w-8 tabular-nums"
          href={sitePath(link.url)}
          data-sveltekit-noscroll
        >
          {linkLabel(link.label)}
        </Button>
      {:else}
        <span class="px-2 text-sm text-muted-foreground tabular-nums">{linkLabel(link.label)}</span>
      {/if}
    {/each}
  </nav>
{/if}
