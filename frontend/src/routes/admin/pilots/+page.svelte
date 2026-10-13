<script lang="ts">
  import { untrack } from 'svelte';
  import { goto } from '$app/navigation';
  import ArrowDown from '@lucide/svelte/icons/arrow-down';
  import ArrowUp from '@lucide/svelte/icons/arrow-up';
  import ChevronDown from '@lucide/svelte/icons/chevron-down';
  import ChevronsUpDown from '@lucide/svelte/icons/chevrons-up-down';
  import Search from '@lucide/svelte/icons/search';

  import AdminLayout from '#lib/components/layouts/admin-layout.svelte';
  import AppLayout from '#lib/components/layouts/app-layout.svelte';
  import PageTitle from '#lib/components/page-title.svelte';
  import Pagination from '#lib/components/pagination.svelte';
  import PilotCharacterRow from '#lib/components/pilot-character-row.svelte';
  import * as Avatar from '#lib/components/ui/avatar/index.ts';
  import { Button } from '#lib/components/ui/button/index.ts';
  import { Input } from '#lib/components/ui/input/index.ts';
  import { effectiveStandingChipClass, eveImage, standingLabel } from '#lib/eve.ts';
  import {
    type SortKey,
    VIEWS,
    type View,
    pilotsUrl,
    viewOf,
    visibleCharacters,
  } from '#lib/pilots.ts';

  let { data } = $props();

  const breadcrumbs = [
    { title: 'Administration', href: '/admin' },
    { title: 'Pilots', href: '/admin/pilots' },
  ];

  const currentView = $derived(viewOf(data.filters.view));
  let search = $state(untrack(() => data.filters.search));

  function visitWith(view: View, term: string) {
    goto(pilotsUrl(view, term), { replace: true, reset: false });
  }

  function setView(view: View) {
    visitWith(view, search);
  }

  // The characters view filters in the browser; only the paginated group
  // views ask the server to search.
  let debounce: ReturnType<typeof setTimeout> | undefined;
  function onSearch() {
    clearTimeout(debounce);
    if (currentView !== 'characters') {
      const view = currentView;
      const term = search;
      debounce = setTimeout(() => visitWith(view, term), 300);
    }
  }

  const description = $derived(
    currentView === 'corporations'
      ? 'Corporations that registered characters belong to, their standing and why it applies.'
      : currentView === 'alliances'
        ? 'Alliances that registered characters belong to, their standing and why it applies.'
        : 'Every registered character, the account it belongs to, and its effective standing.',
  );

  const groupEntityType = $derived(currentView === 'alliances' ? 'alliance' : 'corporation');

  let sortKey: SortKey = $state('name');
  let sortAsc = $state(true);

  function sortBy(key: SortKey) {
    if (sortKey === key) {
      sortAsc = !sortAsc;
    } else {
      sortKey = key;
      // Standings read best blue first.
      sortAsc = key !== 'standing';
    }
  }

  const rows = $derived(visibleCharacters(data.characters ?? [], search, sortKey, sortAsc));

  const SORTABLE_COLUMNS: { key: SortKey; label: string }[] = [
    { key: 'name', label: 'Character' },
    { key: 'account', label: 'Account' },
    { key: 'corporation', label: 'Corporation' },
    { key: 'alliance', label: 'Alliance' },
  ];

  // Group views: collapsible sections, expanded by default.
  let collapsedGroups = $state(new Set<number>());

  function toggleGroup(id: number) {
    const next = new Set(collapsedGroups);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    collapsedGroups = next;
  }
</script>

{#snippet sortIcon(key: SortKey)}
  {#if sortKey === key && sortAsc}
    <ArrowUp class="h-3 w-3" />
  {:else if sortKey === key}
    <ArrowDown class="h-3 w-3" />
  {:else}
    <ChevronsUpDown class="h-3 w-3 opacity-50" />
  {/if}
{/snippet}

<PageTitle title="Pilots" />

<AppLayout {breadcrumbs}>
  <AdminLayout title="Pilots" {description}>
    {#snippet actions()}
      <div class="flex flex-wrap items-center gap-2">
        <div
          class="flex items-center gap-1 rounded-lg border bg-card p-1"
          role="group"
          aria-label="Group pilots by"
        >
          {#each VIEWS as view (view.value)}
            <Button
              type="button"
              size="sm"
              variant={currentView === view.value ? 'secondary' : 'ghost'}
              class="h-7 px-2.5 text-xs"
              onclick={() => setView(view.value)}
            >
              {view.label}
            </Button>
          {/each}
        </div>
        <div class="relative">
          <Search
            class="pointer-events-none absolute top-1/2 left-2.5 h-4 w-4 -translate-y-1/2 text-muted-foreground"
          />
          <Input
            bind:value={search}
            oninput={onSearch}
            type="search"
            placeholder="Search pilots and characters"
            class="w-64 pl-8"
            aria-label="Search pilots and characters"
          />
        </div>
      </div>
    {/snippet}

    {#if currentView === 'characters' && data.characters}
      {#if rows.length === 0}
        <p class="py-16 text-center text-sm text-muted-foreground">
          {#if search}
            No characters match "{search}". Try a different name.
          {:else}
            No characters registered yet. They appear here after their first login.
          {/if}
        </p>
      {:else}
        <div class="overflow-x-auto">
          <div
            class="grid min-w-[48rem] grid-cols-[minmax(0,1.2fr)_minmax(0,1fr)_minmax(0,1fr)_minmax(0,1fr)_5rem] rounded-xl border bg-card"
          >
            <div
              class="col-span-full grid grid-cols-subgrid gap-4 border-b px-3 py-2 text-xs font-medium text-muted-foreground"
            >
              {#each SORTABLE_COLUMNS as column (column.key)}
                <button
                  type="button"
                  class="flex cursor-pointer items-center gap-1 text-left transition-colors hover:text-foreground"
                  onclick={() => sortBy(column.key)}
                >
                  {column.label}
                  {@render sortIcon(column.key)}
                </button>
              {/each}
              <button
                type="button"
                class="flex cursor-pointer items-center justify-end gap-1 text-right transition-colors hover:text-foreground"
                onclick={() => sortBy('standing')}
              >
                Standing
                {@render sortIcon('standing')}
              </button>
            </div>
            {#each rows as character (character.id)}
              <PilotCharacterRow {character} />
            {/each}
          </div>
        </div>
      {/if}
    {:else if data.groups}
      {#if data.groups.data.length === 0}
        <p class="py-16 text-center text-sm text-muted-foreground">
          {#if data.filters.search}
            No {currentView} match "{data.filters.search}". Try a different name.
          {:else}
            No {currentView} with registered characters yet.
          {/if}
        </p>
      {:else}
        <div class="overflow-x-auto">
          <div
            class={[
              'grid min-w-[40rem] rounded-xl border bg-card',
              currentView === 'alliances'
                ? 'grid-cols-[minmax(0,1.2fr)_minmax(0,1fr)_minmax(0,1fr)_5rem]'
                : 'grid-cols-[minmax(0,1.2fr)_minmax(0,1fr)_5rem]',
            ]}
          >
            <div
              class="col-span-full grid grid-cols-subgrid gap-4 border-b px-3 py-2 text-xs font-medium text-muted-foreground"
            >
              <span>Character</span>
              <span>Account</span>
              {#if currentView === 'alliances'}
                <span>Corporation</span>
              {/if}
              <span class="text-right">Standing</span>
            </div>
            {#each data.groups.data as group (group.id)}
              <button
                type="button"
                class="col-span-full flex cursor-pointer items-center justify-between gap-4 border-b bg-muted/60 px-3 py-1.5 text-left transition-colors hover:bg-muted"
                aria-expanded={!collapsedGroups.has(group.id)}
                onclick={() => toggleGroup(group.id)}
              >
                <span class="flex min-w-0 items-center gap-2 text-sm font-semibold">
                  <ChevronDown
                    class={[
                      'h-3.5 w-3.5 shrink-0 text-muted-foreground transition-transform',
                      collapsedGroups.has(group.id) && '-rotate-90',
                    ]}
                  />
                  <Avatar.Root class="h-7 w-7 shrink-0 overflow-hidden rounded">
                    <Avatar.Image
                      src={eveImage(groupEntityType, group.id)}
                      alt={group.name ?? ''}
                    />
                    <Avatar.Fallback class="rounded text-[9px] uppercase">
                      {groupEntityType.slice(0, 2)}
                    </Avatar.Fallback>
                  </Avatar.Root>
                  <span class="truncate">{group.name ?? group.id}</span>
                  {#if group.ticker}
                    <span class="shrink-0 text-xs font-normal text-muted-foreground">
                      [{group.ticker}]
                    </span>
                  {/if}
                </span>
                <span class="flex shrink-0 items-center gap-3 text-xs">
                  {#if group.standing}
                    <span
                      class={[
                        'flex items-center gap-1 rounded-full px-2 py-0.5 font-semibold tabular-nums',
                        effectiveStandingChipClass(group.standing),
                      ]}
                    >
                      {standingLabel(group.standing.value)}
                      <span class="font-normal opacity-75">via {group.standing.source}</span>
                    </span>
                  {:else}
                    <span class="text-muted-foreground italic">no standing</span>
                  {/if}
                  <span class="text-muted-foreground">
                    {group.characters.length}
                    {group.characters.length === 1 ? 'character' : 'characters'}
                  </span>
                </span>
              </button>
              {#if !collapsedGroups.has(group.id)}
                {#each group.characters as character (character.id)}
                  <PilotCharacterRow
                    {character}
                    showCorporation={currentView === 'alliances'}
                    showAlliance={false}
                  />
                {/each}
              {/if}
            {/each}
          </div>
        </div>
      {/if}
      {#if data.groups.last_page > 1}
        <Pagination links={data.groups.links} />
      {/if}
    {/if}
  </AdminLayout>
</AppLayout>
