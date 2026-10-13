<script lang="ts">
  import * as Avatar from '#lib/components/ui/avatar/index.ts';
  import { characterPortrait } from '#lib/eve.ts';
  import { getInitials } from '#lib/initials.ts';
  import type { Character } from '#lib/types.ts';

  let { character }: { character: Character } = $props();
</script>

<Avatar.Root class="h-8 w-8 overflow-hidden rounded-lg">
  <Avatar.Image src={characterPortrait(character.id)} alt={character.name ?? ''} />
  <Avatar.Fallback class="rounded-lg text-black dark:text-white">
    {getInitials(character.name)}
  </Avatar.Fallback>
</Avatar.Root>

<div class="grid flex-1 text-left text-sm leading-tight">
  <span class="truncate font-medium">{character.name}</span>
</div>
