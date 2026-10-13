<script lang="ts">
  // The menu on the right of the top bar: the active character, switching
  // to another one, adding or removing a character, the appearance and
  // logging out.
  import { page } from '$app/state';
  import ChevronDown from '@lucide/svelte/icons/chevron-down';
  import LogOut from '@lucide/svelte/icons/log-out';
  import Palette from '@lucide/svelte/icons/palette';
  import Plus from '@lucide/svelte/icons/plus';
  import UserMinus from '@lucide/svelte/icons/user-minus';

  import EntityAvatar from '#lib/components/entity-avatar.svelte';
  import * as AlertDialog from '#lib/components/ui/alert-dialog/index.ts';
  import * as DropdownMenu from '#lib/components/ui/dropdown-menu/index.ts';
  import { otherCharacters } from '#lib/characters.ts';
  import { send } from '#lib/form.svelte.ts';
  import type { Character } from '#lib/types.ts';

  let { active }: { active: Character } = $props();

  const characters = $derived(page.data.auth?.characters ?? []);
  const others = $derived(otherCharacters(characters, active.id));

  let showRemoveDialog = $state(false);

  function switchTo(character: Character) {
    send('PUT', `/auth/character/${character.id}`);
  }

  function remove() {
    send('DELETE', `/auth/character/${active.id}`);
  }

  function logout() {
    send('POST', '/logout');
  }
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger
    class="flex h-10 min-w-0 items-center gap-2.5 border border-transparent px-2 text-sm transition-colors hover:border-border-strong hover:bg-primary/5 data-[state=open]:border-primary data-[state=open]:bg-primary/10"
    aria-label={`Character menu, ${active.name ?? 'unknown character'}`}
    data-test="character-menu"
  >
    <EntityAvatar type="character" id={active.id} name={active.name} class="size-7" />
    <span class="hidden max-w-40 truncate font-display font-semibold tracking-[0.06em] sm:inline"
      >{active.name}</span
    >
    <ChevronDown class="size-4 shrink-0 text-muted-foreground" />
  </DropdownMenu.Trigger>
  <DropdownMenu.Content class="w-64" align="end" sideOffset={6}>
    <DropdownMenu.Label class="font-normal">
      <p class="label-caps text-[0.6875rem] text-muted-foreground">Signed in as</p>
      <p class="truncate font-medium">{active.name}</p>
    </DropdownMenu.Label>

    {#if others.length > 0}
      <DropdownMenu.Separator />
      <DropdownMenu.Group>
        <DropdownMenu.GroupHeading class="label-caps text-[0.6875rem] text-muted-foreground">
          Switch character
        </DropdownMenu.GroupHeading>
        {#each others as character (character.id)}
          <DropdownMenu.Item onSelect={() => switchTo(character)}>
            <EntityAvatar type="character" id={character.id} name={character.name} class="size-5" />
            <span class="truncate">{character.name}</span>
          </DropdownMenu.Item>
        {/each}
      </DropdownMenu.Group>
    {/if}

    <DropdownMenu.Separator />
    <DropdownMenu.Group>
      <DropdownMenu.Item>
        {#snippet child({ props })}
          <a href="/eve?add_to_account=1" data-sveltekit-reload {...props}>
            <Plus />
            Add character
          </a>
        {/snippet}
      </DropdownMenu.Item>
      {#if others.length > 0}
        <DropdownMenu.Item onSelect={() => (showRemoveDialog = true)}>
          <UserMinus />
          Remove {active.name}
        </DropdownMenu.Item>
      {/if}
      <DropdownMenu.Item>
        {#snippet child({ props })}
          <a href="/settings/appearance" {...props}>
            <Palette />
            Appearance
          </a>
        {/snippet}
      </DropdownMenu.Item>
    </DropdownMenu.Group>
    <DropdownMenu.Separator />
    <DropdownMenu.Item data-test="logout-button" onSelect={logout}>
      <LogOut />
      Log out
    </DropdownMenu.Item>
  </DropdownMenu.Content>
</DropdownMenu.Root>

<AlertDialog.Root bind:open={showRemoveDialog}>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>Remove {active.name}?</AlertDialog.Title>
      <AlertDialog.Description>
        {active.name} leaves your account and stops syncing standings. You can add it again by logging
        in with it.
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
      <AlertDialog.Action
        class="bg-destructive text-destructive-foreground hover:bg-destructive/90"
        onclick={remove}
      >
        Remove
      </AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
