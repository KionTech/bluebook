<script lang="ts">
  import { untrack } from 'svelte';
  import { goto } from '$app/navigation';
  import ArrowDown from '@lucide/svelte/icons/arrow-down';
  import ArrowUp from '@lucide/svelte/icons/arrow-up';
  import ChevronDown from '@lucide/svelte/icons/chevron-down';
  import ChevronsUpDown from '@lucide/svelte/icons/chevrons-up-down';
  import Search from '@lucide/svelte/icons/search';
  import Users from '@lucide/svelte/icons/users';

  import AppShell from '#lib/components/app-shell.svelte';
  import EmptyState from '#lib/components/empty-state.svelte';
  import EntityAvatar from '#lib/components/entity-avatar.svelte';
  import FilterTabs from '#lib/components/filter-tabs.svelte';
  import PageTitle from '#lib/components/page-title.svelte';
  import Pagination from '#lib/components/pagination.svelte';
  import { Badge } from '#lib/components/ui/badge/index.ts';
  import { Input } from '#lib/components/ui/input/index.ts';
  import {
    effectiveStandingChipClass,
    effectiveStandingTextClass,
    standingLabel,
  } from '#lib/eve.ts';
  import {
    type SortKey,
    VIEWS,
    type View,
    pilotsUrl,
    viewOf,
    visibleCharacters,
  } from '#lib/pilots.ts';
  import type { EntitySummary, PilotCharacter } from '#lib/types.ts';

  let { data } = $props();

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

  function ariaSort(key: SortKey): 'ascending' | 'descending' | 'none' {
    if (sortKey !== key) {
      return 'none';
    }
    return sortAsc ? 'ascending' : 'descending';
  }

  const rows = $derived(visibleCharacters(data.characters ?? [], search, sortKey, sortAsc));

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

  function entityName(entity: EntitySummary | null): string {
    if (!entity) {
      return '-';
    }
    return `${entity.name ?? entity.id}${entity.ticker ? ` [${entity.ticker}]` : ''}`;
  }
</script>

{#snippet sortHeader(key: SortKey, label: string, cellClass: string)}
  <th scope="col" class={['py-2 font-semibold', cellClass]} aria-sort={ariaSort(key)}>
    <button
      type="button"
      class={[
        'label-caps inline-flex items-center gap-1 transition-colors hover:text-primary',
        key === 'standing' && 'flex-row-reverse',
      ]}
      onclick={() => sortBy(key)}
    >
      {label}
      {#if sortKey === key && sortAsc}
        <ArrowUp class="size-3" />
      {:else if sortKey === key}
        <ArrowDown class="size-3" />
      {:else}
        <ChevronsUpDown class="size-3 opacity-50" />
      {/if}
    </button>
  </th>
{/snippet}

{#snippet characterRow(character: PilotCharacter, showCorporation: boolean, showAlliance: boolean)}
  <tr>
    <td class="py-2 pr-3 pl-2">
      <div class="flex min-w-0 items-center gap-2.5">
        <EntityAvatar type="character" id={character.id} name={character.name} class="size-7" />
        <div class="min-w-0">
          <p class="flex items-center gap-2">
            <span class="truncate font-medium">{character.name ?? character.id}</span>
            {#if character.is_main}
              <Badge variant="secondary">Main</Badge>
            {/if}
          </p>
          <p class="truncate text-xs text-muted-foreground sm:hidden">
            {character.account.name ?? character.account.id}
          </p>
        </div>
      </div>
    </td>
    <td class="hidden truncate py-2 pr-3 text-muted-foreground sm:table-cell">
      {character.account.name ?? character.account.id}
    </td>
    {#if showCorporation}
      <td class="hidden truncate py-2 pr-3 md:table-cell">{entityName(character.corporation)}</td>
    {/if}
    {#if showAlliance}
      <td class="hidden truncate py-2 pr-3 lg:table-cell">{entityName(character.alliance)}</td>
    {/if}
    <td class="py-2 pr-2 text-right">
      {#if character.standing}
        <p class={['font-mono font-medium', effectiveStandingTextClass(character.standing)]}>
          {standingLabel(character.standing.value)}
        </p>
        <p class="label-caps text-[0.625rem] text-muted-foreground">{character.standing.source}</p>
      {:else}
        <span class="text-muted-foreground">-</span>
      {/if}
    </td>
  </tr>
{/snippet}

<PageTitle title="Pilots" />

<AppShell title="Pilots" {description} admin>
  <div class="mb-6 flex flex-wrap items-center gap-3">
    <FilterTabs
      label="Group pilots by"
      items={VIEWS.map((view) => ({ value: view.value, label: view.label }))}
      value={currentView}
      onselect={setView}
    />
    <div class="relative w-full sm:w-72">
      <Search
        class="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
      />
      <Input
        bind:value={search}
        oninput={onSearch}
        type="search"
        name="pilots-search"
        placeholder="Search pilots and characters"
        class="pl-8"
        aria-label="Search pilots and characters"
      />
    </div>
  </div>

  {#if currentView === 'characters' && data.characters}
    {#if rows.length === 0}
      <EmptyState
        title={search ? 'No characters match your search' : 'No characters yet'}
        icon={Users}
      >
        {#if search}
          Nothing matches "{search}". Try a different name.
        {:else}
          Characters appear here after their first login.
        {/if}
      </EmptyState>
    {:else}
      <table class="w-full table-fixed text-sm">
        <thead>
          <tr class="label-caps border-b text-left text-[0.6875rem] text-muted-foreground">
            {@render sortHeader('name', 'Character', 'pl-2 pr-3')}
            {@render sortHeader('account', 'Account', 'hidden sm:table-cell pr-3')}
            {@render sortHeader('corporation', 'Corporation', 'hidden md:table-cell pr-3')}
            {@render sortHeader('alliance', 'Alliance', 'hidden lg:table-cell pr-3')}
            {@render sortHeader('standing', 'Standing', 'w-24 pr-2 text-right')}
          </tr>
        </thead>
        <tbody class="rows divide-y">
          {#each rows as character (character.id)}
            {@render characterRow(character, true, true)}
          {/each}
        </tbody>
      </table>
    {/if}
  {:else if data.groups}
    {#if data.groups.data.length === 0}
      <EmptyState
        title={data.filters.search
          ? `No ${currentView} match your search`
          : `No ${currentView} yet`}
        icon={Users}
      >
        {#if data.filters.search}
          Nothing matches "{data.filters.search}". Try a different name.
        {:else}
          {currentView === 'alliances' ? 'Alliances' : 'Corporations'} appear here once a registered character
          belongs to one.
        {/if}
      </EmptyState>
    {:else}
      <table class="w-full table-fixed text-sm">
        <thead>
          <tr class="label-caps border-b text-left text-[0.6875rem] text-muted-foreground">
            <th scope="col" class="py-2 pr-3 pl-2 font-semibold">Character</th>
            <th scope="col" class="hidden py-2 pr-3 font-semibold sm:table-cell">Account</th>
            {#if currentView === 'alliances'}
              <th scope="col" class="hidden py-2 pr-3 font-semibold md:table-cell">Corporation</th>
            {/if}
            <th scope="col" class="w-24 py-2 pr-2 text-right font-semibold">Standing</th>
          </tr>
        </thead>
        {#each data.groups.data as group (group.id)}
          <tbody class="rows divide-y border-b">
            <tr class="bg-secondary/60">
              <th
                scope="colgroup"
                colspan={currentView === 'alliances' ? 4 : 3}
                class="p-0 text-left font-normal"
              >
                <button
                  type="button"
                  class="flex w-full items-center gap-3 px-2 py-2 text-left transition-colors hover:bg-primary/8"
                  aria-expanded={!collapsedGroups.has(group.id)}
                  onclick={() => toggleGroup(group.id)}
                >
                  <ChevronDown
                    class={[
                      'size-4 shrink-0 text-muted-foreground transition-transform',
                      collapsedGroups.has(group.id) && '-rotate-90',
                    ]}
                  />
                  <EntityAvatar
                    type={groupEntityType}
                    id={group.id}
                    name={group.name}
                    class="size-7"
                  />
                  <span class="min-w-0 flex-1">
                    <span class="block truncate font-semibold">
                      {group.name ?? group.id}
                      {#if group.ticker}
                        <span class="font-normal text-muted-foreground">[{group.ticker}]</span>
                      {/if}
                    </span>
                    <span class="block text-xs text-muted-foreground">
                      {group.characters.length}
                      {group.characters.length === 1 ? 'character' : 'characters'}
                    </span>
                  </span>
                  {#if group.standing}
                    <span
                      class={[
                        'shrink-0 border border-current/30 px-2 py-0.5 font-mono text-xs font-medium',
                        effectiveStandingChipClass(group.standing),
                      ]}
                    >
                      {standingLabel(group.standing.value)}
                      <span class="font-sans">via {group.standing.source}</span>
                    </span>
                  {:else}
                    <span class="shrink-0 text-xs text-muted-foreground">No standing</span>
                  {/if}
                </button>
              </th>
            </tr>
            {#if !collapsedGroups.has(group.id)}
              {#each group.characters as character (character.id)}
                {@render characterRow(character, currentView === 'alliances', false)}
              {/each}
            {/if}
          </tbody>
        {/each}
      </table>
    {/if}
    {#if data.groups.last_page > 1}
      <div class="mt-6">
        <Pagination links={data.groups.links} />
      </div>
    {/if}
  {/if}
</AppShell>
