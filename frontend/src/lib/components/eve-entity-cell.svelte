<script lang="ts">
  import * as Avatar from '#lib/components/ui/avatar/index.ts';
  import { eveImage } from '#lib/eve.ts';
  import type { EntitySummary } from '#lib/types.ts';

  let { type, entity }: { type: 'corporation' | 'alliance'; entity: EntitySummary | null } =
    $props();
</script>

<div class="flex min-w-0 items-center gap-2 text-sm">
  {#if entity}
    <Avatar.Root class="h-5 w-5 shrink-0 overflow-hidden rounded">
      <Avatar.Image src={eveImage(type, entity.id)} alt={entity.name ?? ''} />
      <Avatar.Fallback class="rounded text-[9px] uppercase">{type.slice(0, 2)}</Avatar.Fallback>
    </Avatar.Root>
    <span class="truncate">{entity.name ?? entity.id}</span>
    {#if entity.ticker}
      <span class="shrink-0 text-xs text-muted-foreground">[{entity.ticker}]</span>
    {/if}
  {:else}
    <span class="text-muted-foreground">-</span>
  {/if}
</div>
