<script lang="ts">
  // The app frame: the sidebar with the navigation and the character menu,
  // the breadcrumbs, the access banners above the page and the toasts.
  import type { Snippet } from 'svelte';
  import { page } from '$app/state';
  import ChevronsUpDown from '@lucide/svelte/icons/chevrons-up-down';
  import LayoutGrid from '@lucide/svelte/icons/layout-grid';
  import LogOut from '@lucide/svelte/icons/log-out';
  import Minus from '@lucide/svelte/icons/minus';
  import Plus from '@lucide/svelte/icons/plus';
  import Settings from '@lucide/svelte/icons/settings';
  import ShieldCheck from '@lucide/svelte/icons/shield-check';

  import AccessWarnings from '#lib/components/access-warnings.svelte';
  import AppLogo from '#lib/components/app-logo.svelte';
  import Toaster from '#lib/components/toaster.svelte';
  import UserInfo from '#lib/components/user-info.svelte';
  import * as AlertDialog from '#lib/components/ui/alert-dialog/index.ts';
  import * as Avatar from '#lib/components/ui/avatar/index.ts';
  import * as Breadcrumb from '#lib/components/ui/breadcrumb/index.ts';
  import * as DropdownMenu from '#lib/components/ui/dropdown-menu/index.ts';
  import * as Sidebar from '#lib/components/ui/sidebar/index.ts';
  import { characterPortrait } from '#lib/eve.ts';
  import { send } from '#lib/form.svelte.ts';
  import { getInitials } from '#lib/initials.ts';
  import type { BreadcrumbItem, Character } from '#lib/types.ts';

  let { breadcrumbs = [], children }: { breadcrumbs?: BreadcrumbItem[]; children: Snippet } =
    $props();

  const auth = $derived(page.data.auth);
  const active = $derived(auth?.active_character);
  const otherCharacters = $derived(
    (auth?.characters ?? []).filter((character) => character.id !== active?.id),
  );
  const isLastCharacter = $derived((auth?.characters ?? []).length <= 1);

  const mainNavItems = $derived([
    { title: 'Dashboard', href: '/dashboard', icon: LayoutGrid },
    ...(auth?.is_admin ? [{ title: 'Administration', href: '/admin', icon: ShieldCheck }] : []),
  ]);

  // svelte-ignore state_referenced_locally
  let open = $state(page.data.sidebarOpen ?? true);
  let showRemoveDialog = $state(false);

  function switchCharacter(character: Character) {
    send('PUT', `/auth/character/${character.id}`);
  }

  function removeCharacter() {
    if (active) {
      send('DELETE', `/auth/character/${active.id}`);
    }
  }

  function logout() {
    send('POST', '/logout');
  }

  /** The active entry: the dashboard alone, the admin section for every
   * admin page. */
  function isActive(href: string): boolean {
    const path = page.url.pathname;
    return href === '/dashboard' ? path === href : path === href || path.startsWith(`${href}/`);
  }
</script>

<Toaster />

<Sidebar.Provider bind:open>
  <Sidebar.Root collapsible="icon" variant="inset">
    <Sidebar.Header>
      <Sidebar.Menu>
        <Sidebar.MenuItem>
          <Sidebar.MenuButton size="lg">
            {#snippet child({ props })}
              <a href="/dashboard" {...props}><AppLogo /></a>
            {/snippet}
          </Sidebar.MenuButton>
        </Sidebar.MenuItem>
      </Sidebar.Menu>
    </Sidebar.Header>

    <Sidebar.Content>
      <Sidebar.Group class="px-2 py-0">
        <Sidebar.GroupLabel>Platform</Sidebar.GroupLabel>
        <Sidebar.Menu>
          {#each mainNavItems as item (item.title)}
            <Sidebar.MenuItem>
              <Sidebar.MenuButton isActive={isActive(item.href)} tooltipContent={item.title}>
                {#snippet child({ props })}
                  <a href={item.href} {...props}>
                    <item.icon />
                    <span>{item.title}</span>
                  </a>
                {/snippet}
              </Sidebar.MenuButton>
            </Sidebar.MenuItem>
          {/each}
        </Sidebar.Menu>
      </Sidebar.Group>
    </Sidebar.Content>

    <Sidebar.Footer>
      {#if active}
        <Sidebar.Menu>
          <Sidebar.MenuItem>
            <DropdownMenu.Root>
              <DropdownMenu.Trigger>
                {#snippet child({ props })}
                  <Sidebar.MenuButton
                    size="lg"
                    class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
                    data-test="sidebar-menu-button"
                    {...props}
                  >
                    <UserInfo character={active} />
                    <ChevronsUpDown class="ml-auto size-4" />
                  </Sidebar.MenuButton>
                {/snippet}
              </DropdownMenu.Trigger>
              <DropdownMenu.Content
                class="w-(--bits-dropdown-menu-anchor-width) min-w-56 rounded-lg"
                side={Sidebar.useSidebar().isMobile
                  ? 'bottom'
                  : Sidebar.useSidebar().state === 'collapsed'
                    ? 'left'
                    : 'bottom'}
                align="end"
                sideOffset={4}
              >
                <DropdownMenu.Label class="p-0 font-normal">
                  <div class="flex items-center gap-2 px-1 py-1.5 text-left text-sm">
                    <UserInfo character={active} />
                  </div>
                </DropdownMenu.Label>

                {#if otherCharacters.length > 0}
                  <DropdownMenu.Separator />
                  <DropdownMenu.Label class="text-xs font-normal text-muted-foreground">
                    Other Characters
                  </DropdownMenu.Label>
                  <DropdownMenu.Group>
                    {#each otherCharacters as character (character.id)}
                      <DropdownMenu.Item
                        class="cursor-pointer"
                        onSelect={() => switchCharacter(character)}
                      >
                        <Avatar.Root class="mr-2 h-5 w-5 overflow-hidden rounded">
                          <Avatar.Image
                            src={characterPortrait(character.id)}
                            alt={character.name ?? ''}
                          />
                          <Avatar.Fallback class="rounded text-[10px]">
                            {getInitials(character.name)}
                          </Avatar.Fallback>
                        </Avatar.Root>
                        {character.name}
                      </DropdownMenu.Item>
                    {/each}
                  </DropdownMenu.Group>
                {/if}

                <DropdownMenu.Separator />
                <DropdownMenu.Group>
                  <DropdownMenu.Item>
                    {#snippet child({ props })}
                      <a
                        href="/eve?add_to_account=1"
                        data-sveltekit-reload
                        class="block w-full cursor-pointer"
                        {...props}
                      >
                        <Plus class="mr-2 h-4 w-4" />
                        Add Character
                      </a>
                    {/snippet}
                  </DropdownMenu.Item>
                  {#if !isLastCharacter}
                    <DropdownMenu.Item
                      class="cursor-pointer text-destructive focus:text-destructive"
                      onSelect={() => (showRemoveDialog = true)}
                    >
                      <Minus class="mr-2 h-4 w-4" />
                      Remove Character
                    </DropdownMenu.Item>
                  {/if}
                  <DropdownMenu.Item>
                    {#snippet child({ props })}
                      <a href="/settings/appearance" class="block w-full cursor-pointer" {...props}>
                        <Settings class="mr-2 h-4 w-4" />
                        Settings
                      </a>
                    {/snippet}
                  </DropdownMenu.Item>
                  <DropdownMenu.Item
                    class="block w-full cursor-pointer"
                    data-test="logout-button"
                    onSelect={logout}
                  >
                    <LogOut class="mr-2 h-4 w-4" />
                    Log out
                  </DropdownMenu.Item>
                </DropdownMenu.Group>
              </DropdownMenu.Content>
            </DropdownMenu.Root>
          </Sidebar.MenuItem>
        </Sidebar.Menu>
      {/if}
    </Sidebar.Footer>
  </Sidebar.Root>

  <Sidebar.Inset class="overflow-x-hidden">
    <header
      class="flex h-16 shrink-0 items-center gap-2 border-b border-sidebar-border/70 px-6 transition-[width,height] ease-linear group-has-data-[collapsible=icon]/sidebar-wrapper:h-12 md:px-4"
    >
      <div class="flex items-center gap-2">
        <Sidebar.Trigger class="-ml-1" />
        {#if breadcrumbs.length > 0}
          <Breadcrumb.Root>
            <Breadcrumb.List>
              {#each breadcrumbs as item, index (index)}
                <Breadcrumb.Item>
                  {#if index === breadcrumbs.length - 1}
                    <Breadcrumb.Page>{item.title}</Breadcrumb.Page>
                  {:else}
                    <Breadcrumb.Link href={item.href}>{item.title}</Breadcrumb.Link>
                  {/if}
                </Breadcrumb.Item>
                {#if index !== breadcrumbs.length - 1}
                  <Breadcrumb.Separator />
                {/if}
              {/each}
            </Breadcrumb.List>
          </Breadcrumb.Root>
        {/if}
      </div>
    </header>
    <AccessWarnings />
    {@render children()}
  </Sidebar.Inset>
</Sidebar.Provider>

<AlertDialog.Root bind:open={showRemoveDialog}>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>Remove Character</AlertDialog.Title>
      <AlertDialog.Description>
        Are you sure you want to remove
        <span class="font-medium text-foreground">{active?.name}</span>
        from your account?
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
      <AlertDialog.Action
        class="bg-destructive text-destructive-foreground hover:bg-destructive/90"
        onclick={removeCharacter}
      >
        Remove
      </AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
