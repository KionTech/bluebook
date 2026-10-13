<script lang="ts">
  // One character of the admin's pilot tables: the character, its
  // account, its affiliations and its effective standing.
  import EveEntityCell from '#lib/components/eve-entity-cell.svelte';
  import * as Avatar from '#lib/components/ui/avatar/index.ts';
  import { Badge } from '#lib/components/ui/badge/index.ts';
  import { effectiveStandingTextClass, eveImage, standingLabel } from '#lib/eve.ts';
  import { getInitials } from '#lib/initials.ts';
  import type { PilotCharacter } from '#lib/types.ts';

  let {
    character,
    showAccount = true,
    showCorporation = true,
    showAlliance = true,
  }: {
    character: PilotCharacter;
    showAccount?: boolean;
    showCorporation?: boolean;
    showAlliance?: boolean;
  } = $props();
</script>

<div
  class="col-span-full grid grid-cols-subgrid items-center gap-4 border-b px-3 py-2 transition-colors last:border-b-0 hover:bg-muted"
>
  <div class="flex min-w-0 items-center gap-3">
    <Avatar.Root class="h-7 w-7 shrink-0 overflow-hidden rounded">
      <Avatar.Image src={eveImage('character', character.id)} alt={character.name ?? ''} />
      <Avatar.Fallback class="rounded text-xs">{getInitials(character.name ?? '?')}</Avatar.Fallback
      >
    </Avatar.Root>
    <span class="truncate text-sm font-medium">{character.name ?? character.id}</span>
    {#if character.is_main}
      <Badge variant="secondary" class="shrink-0">Main</Badge>
    {/if}
  </div>
  {#if showAccount}
    <span class="truncate text-sm text-muted-foreground">
      {character.account.name ?? character.account.id}
    </span>
  {/if}
  {#if showCorporation}
    <EveEntityCell type="corporation" entity={character.corporation} />
  {/if}
  {#if showAlliance}
    <EveEntityCell type="alliance" entity={character.alliance} />
  {/if}
  {#if character.standing}
    <div class="text-right">
      <p
        class={[
          'text-sm font-semibold tabular-nums',
          effectiveStandingTextClass(character.standing),
        ]}
      >
        {standingLabel(character.standing.value)}
      </p>
      <p class="text-[10px] leading-tight text-muted-foreground">{character.standing.source}</p>
    </div>
  {:else}
    <span class="text-right text-sm text-muted-foreground">-</span>
  {/if}
</div>
