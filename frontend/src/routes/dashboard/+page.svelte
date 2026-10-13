<script lang="ts">
  import { onMount } from 'svelte';
  import { refreshAll } from '$app/navigation';
  import KeyRound from '@lucide/svelte/icons/key-round';
  import Lock from '@lucide/svelte/icons/lock';
  import Plus from '@lucide/svelte/icons/plus';
  import RefreshCw from '@lucide/svelte/icons/refresh-cw';
  import Search from '@lucide/svelte/icons/search';
  import Star from '@lucide/svelte/icons/star';

  import AppShell from '#lib/components/app-shell.svelte';
  import EmptyState from '#lib/components/empty-state.svelte';
  import EntityAvatar from '#lib/components/entity-avatar.svelte';
  import PageTitle from '#lib/components/page-title.svelte';
  import SectionHeading from '#lib/components/section-heading.svelte';
  import SetupWizard from '#lib/components/setup-wizard.svelte';
  import { Badge } from '#lib/components/ui/badge/index.ts';
  import { Button } from '#lib/components/ui/button/index.ts';
  import * as Dialog from '#lib/components/ui/dialog/index.ts';
  import { Input } from '#lib/components/ui/input/index.ts';
  import { Switch } from '#lib/components/ui/switch/index.ts';
  import { syncState, syncSummary } from '#lib/characters.ts';
  import { dateWithAgo } from '#lib/date.ts';
  import { standingLabel, standingTextClass } from '#lib/eve.ts';
  import { send } from '#lib/form.svelte.ts';
  import { Now } from '#lib/now.svelte.ts';
  import { POLL_INTERVAL_MS, startPolling } from '#lib/poll.ts';
  import { nextRefresh } from '#lib/refresh.ts';
  import {
    canRequest,
    filterStandings,
    optionStatusLabel,
    requestedOptions,
    sortStandings,
  } from '#lib/standings.ts';
  import type { RequestOption, SyncCharacter } from '#lib/types.ts';

  let { data } = $props();

  // Reload the props every 30 s so standings and characters stay live.
  onMount(() => startPolling(refreshAll, POLL_INTERVAL_MS));

  const now = new Now();
  const lastRefreshed = $derived(dateWithAgo(data.source?.last_synced_at, now.current));
  const nextRefreshIn = $derived(nextRefresh(data.source?.last_synced_at, now.current));

  let standingsSearch = $state('');
  const sortedStandings = $derived(sortStandings(data.standings ?? []));
  const filteredStandings = $derived(filterStandings(sortedStandings, standingsSearch));

  const reauthIds = $derived(
    new Set((data.auth.reauth_characters ?? []).map((character) => character.id)),
  );
  const requests = $derived(requestedOptions(data.requestableOptions));

  const REQUEST_STATUS: Record<
    string,
    { label: string; variant: 'outline' | 'secondary'; pip: string }
  > = {
    pending: { label: 'Waiting for an admin', variant: 'outline', pip: 'text-warning' },
    done: { label: 'Approved', variant: 'secondary', pip: 'text-success' },
    rejected: { label: 'Rejected', variant: 'outline', pip: 'text-destructive' },
  };

  // The colour of a character's status pip.
  const PIP = {
    on: 'text-success',
    off: 'text-standing-neutral',
    attention: 'text-warning',
    inherited: 'text-primary',
  };

  function pipOf(character: SyncCharacter, state: string): keyof typeof PIP {
    if (state === 'inherited') {
      return 'inherited';
    }
    if (state === 'reauth' || state === 'grant') {
      return 'attention';
    }
    return character.should_sync ? 'on' : 'off';
  }

  let requestOpen = $state(false);

  function syncMyCharacters() {
    send('POST', '/standings/sync');
  }

  function setMainCharacter(character: SyncCharacter) {
    send('PUT', `/auth/character/${character.id}/main`);
  }

  function toggleSync(character: SyncCharacter, value: boolean) {
    send('PUT', `/standings/${character.id}`, { should_sync: value });
  }

  function requestStanding(option: RequestOption) {
    if (!canRequest(option, data.standings)) {
      return;
    }
    requestOpen = false;
    send('POST', `/standings/${option.via_character_id}/request`, { type: option.type });
  }
</script>

<PageTitle title="Standings" />

{#if data.showSetupWizard}
  <SetupWizard characters={data.characters} />
{/if}

<AppShell
  title="Standings"
  description={data.source
    ? `Your characters mirror the contacts of ${data.source.entity_name ?? data.source.type}.`
    : 'Your characters mirror the contacts of the standings source.'}
  reauthNotice={false}
>
  {#snippet actions()}
    {#if data.requestableOptions.length > 0}
      <Dialog.Root bind:open={requestOpen}>
        <Dialog.Trigger>
          {#snippet child({ props })}
            <Button type="button" variant="outline" {...props}>
              <Plus />
              Request standing
            </Button>
          {/snippet}
        </Dialog.Trigger>
        <Dialog.Content class="sm:max-w-md">
          <Dialog.Header>
            <Dialog.Title>Request standing</Dialog.Title>
            <Dialog.Description>
              Ask the admins to add one of your characters, corporations or alliances to the
              standings.
            </Dialog.Description>
          </Dialog.Header>
          <ul class="-mx-2 max-h-80 overflow-y-auto">
            {#each data.requestableOptions as option (`${option.type}:${option.id}`)}
              <li>
                <button
                  type="button"
                  disabled={!canRequest(option, data.standings)}
                  class="flex w-full items-center gap-3 px-2 py-2 text-left transition-[background-color,box-shadow] hover:bg-primary/8 hover:shadow-[inset_2px_0_0_var(--primary)] disabled:pointer-events-none disabled:opacity-60"
                  onclick={() => requestStanding(option)}
                >
                  <EntityAvatar type={option.type} id={option.id} name={option.name} />
                  <span class="min-w-0 flex-1">
                    <span class="block truncate text-sm font-medium"
                      >{option.name ?? option.id}</span
                    >
                    <span class="label-caps block text-[0.6875rem] text-muted-foreground"
                      >{option.type}</span
                    >
                  </span>
                  {#if optionStatusLabel(option, data.standings)}
                    <span class="text-xs text-muted-foreground">
                      {optionStatusLabel(option, data.standings)}
                    </span>
                  {/if}
                </button>
              </li>
            {/each}
          </ul>
        </Dialog.Content>
      </Dialog.Root>
    {/if}
    <Button type="button" onclick={syncMyCharacters}>
      <RefreshCw />
      Sync now
    </Button>
  {/snippet}

  <!-- Source -->
  {#if data.source}
    <dl class="frame mb-10 grid grid-cols-2 gap-x-6 gap-y-4 px-5 py-4 text-sm sm:grid-cols-4">
      <div class="col-span-2 min-w-0 sm:col-span-1">
        <dt class="label-caps text-[0.6875rem] text-muted-foreground">Source</dt>
        <dd class="flex min-w-0 items-center gap-2 font-medium">
          <EntityAvatar
            type={data.source.type}
            id={data.source.entity_id}
            name={data.source.entity_name}
            class="size-6"
          />
          <span class="truncate">{data.source.entity_name ?? data.source.entity_id}</span>
        </dd>
      </div>
      {#if data.standings}
        <div>
          <dt class="label-caps text-[0.6875rem] text-muted-foreground">Standings</dt>
          <dd class="font-mono text-base font-medium text-primary">{data.standings.length}</dd>
        </div>
      {/if}
      <div>
        <dt class="label-caps text-[0.6875rem] text-muted-foreground">Last refreshed</dt>
        <dd class="font-mono text-[0.8125rem]">{lastRefreshed}</dd>
      </div>
      {#if nextRefreshIn}
        <div>
          <dt class="label-caps text-[0.6875rem] text-muted-foreground">Next refresh</dt>
          <dd class="font-mono text-[0.8125rem]">{nextRefreshIn}</dd>
        </div>
      {/if}
    </dl>
  {:else}
    <div class="mb-10">
      <EmptyState title="No standings source yet">
        An admin has to choose the character, corporation or alliance whose contacts everyone
        mirrors. Until then there is nothing to sync.
      </EmptyState>
    </div>
  {/if}

  <!-- Your characters -->
  <section class="mb-12" aria-labelledby="characters-heading">
    <SectionHeading
      id="characters-heading"
      title="Your characters"
      description="Turn sync on for each character that should carry the standings. The star marks your main."
    />
    <ul class="rows divide-y border-b">
      {#each data.characters as character (character.id)}
        {@const state = syncState(character, reauthIds)}
        <li class="px-2 py-3" data-test={`character-${character.id}`}>
          <div class="flex items-center gap-3">
            <EntityAvatar
              type="character"
              id={character.id}
              name={character.name}
              class="size-10"
            />
            <div class="min-w-0 flex-1">
              <p class="flex items-center gap-2 font-medium">
                <span class="truncate">{character.name}</span>
                {#if character.is_main}
                  <Badge variant="secondary">Main</Badge>
                {/if}
              </p>
              {#if character.corporation || character.alliance}
                <p class="truncate text-xs text-muted-foreground">
                  {#if character.corporation}{character.corporation.name}{/if}
                  {#if character.corporation && character.alliance}<span aria-hidden="true">
                      &middot;
                    </span>{/if}
                  {#if character.alliance}{character.alliance.name}{/if}
                </p>
              {/if}
              <p class="flex items-center gap-1.5 text-xs text-muted-foreground">
                <span aria-hidden="true" class={['pip', PIP[pipOf(character, state)]]}></span>
                {syncSummary(character, state)}
              </p>
            </div>
            <div class="flex shrink-0 items-center gap-1 sm:gap-3">
              {#if !character.is_main}
                <Button
                  type="button"
                  variant="ghost"
                  size="icon-sm"
                  class="text-muted-foreground"
                  title="Set as main character"
                  aria-label={`Set ${character.name} as your main character`}
                  onclick={() => setMainCharacter(character)}
                >
                  <Star />
                </Button>
              {/if}
              {#if state === 'inherited'}
                <Badge variant="outline">Inherited</Badge>
              {:else if state === 'grant'}
                <Button
                  href={data.auth.sync_scopes_url}
                  data-sveltekit-reload
                  size="sm"
                  variant="outline"
                  title={`Grant ${character.name} the contact permissions needed to sync standings`}
                >
                  <KeyRound />
                  Grant sync access
                </Button>
              {:else if state === 'toggle'}
                <Switch
                  checked={character.should_sync}
                  aria-label={`Sync standings to ${character.name}`}
                  onCheckedChange={(value) => toggleSync(character, value === true)}
                />
              {/if}
            </div>
          </div>
          {#if state === 'reauth'}
            <div
              role="alert"
              class="frame tone-warning tone-bar mt-3 flex flex-col gap-3 px-3 py-2.5 text-sm [--cut:8px] sm:ml-13 sm:flex-row sm:items-center"
            >
              <p class="flex-1">
                <span class="font-medium">{character.name} needs to log in again.</span>
                Its token lapsed, so its contacts are not being synced.
              </p>
              <Button
                href={data.auth.sync_scopes_url}
                data-sveltekit-reload
                size="sm"
                variant="tone"
                class="self-start sm:self-auto"
              >
                <KeyRound />
                Log in again
              </Button>
            </div>
          {/if}
        </li>
      {/each}
    </ul>
  </section>

  <!-- Current standings -->
  <section class="mb-12" aria-labelledby="standings-heading">
    <SectionHeading
      id="standings-heading"
      title="Current standings"
      description="The standings of the source, applied to every syncing character."
    />
    {#if !data.source}
      <EmptyState title="No standings yet">
        Standings appear here once an admin sets a source.
      </EmptyState>
    {:else if !data.canViewStandings}
      <EmptyState title="Standings are hidden" icon={Lock}>
        None of your characters has a standing yet. Request one for a character, corporation or
        alliance. Once an admin approves it, the list unlocks.
      </EmptyState>
    {:else if !data.standings || data.standings.length === 0}
      <EmptyState title="The source has no standings yet">
        Nothing has been pulled from the source so far. The next refresh brings them in.
      </EmptyState>
    {:else}
      <div class="relative mb-4 max-w-sm">
        <Search
          class="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
        />
        <Input
          bind:value={standingsSearch}
          type="search"
          name="standings-search"
          placeholder="Search standings"
          aria-label="Search standings"
          class="pl-8"
        />
      </div>
      {#if filteredStandings.length === 0}
        <EmptyState title="No standings match your search.">Try part of a name instead.</EmptyState>
      {:else}
        <table class="w-full table-fixed text-sm">
          <thead>
            <tr class="label-caps border-b text-left text-[0.6875rem] text-muted-foreground">
              <th scope="col" class="px-2 py-2 font-semibold">Name</th>
              <th scope="col" class="hidden w-32 py-2 font-semibold sm:table-cell">Type</th>
              <th scope="col" class="w-24 px-2 py-2 text-right font-semibold">Standing</th>
            </tr>
          </thead>
          <tbody class="rows divide-y">
            {#each filteredStandings as standing (standing.contact_id)}
              <tr>
                <td class="py-2 pr-3 pl-2">
                  <div class="flex min-w-0 items-center gap-2.5">
                    <EntityAvatar
                      type={standing.contact_type}
                      id={standing.contact_id}
                      name={standing.name}
                      class="size-6"
                    />
                    <div class="min-w-0">
                      <p class="truncate font-medium" data-test="standing-name">
                        {standing.name ?? standing.contact_id}
                      </p>
                      {#if standing.redundant_via}
                        <p
                          class="truncate text-xs text-warning"
                          title={`This standing already applies through the ${standing.redundant_via.contact_type} and can be removed from the source.`}
                        >
                          Redundant via
                          {standing.redundant_via.name ?? standing.redundant_via.contact_id}
                        </p>
                      {/if}
                      <p class="text-xs text-muted-foreground capitalize sm:hidden">
                        {standing.contact_type}
                      </p>
                    </div>
                  </div>
                </td>
                <td
                  class="label-caps hidden py-2 text-[0.6875rem] text-muted-foreground sm:table-cell"
                >
                  {standing.contact_type}
                </td>
                <td
                  class={[
                    'px-2 py-2 text-right font-mono font-medium',
                    standingTextClass(standing.standing),
                  ]}
                >
                  {standingLabel(standing.standing)}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    {/if}
  </section>

  <!-- Your requests -->
  {#if data.requestableOptions.length > 0}
    <section aria-labelledby="requests-heading">
      <SectionHeading
        id="requests-heading"
        title="Your requests"
        description="Standings you asked the admins for."
      />
      {#if requests.length === 0}
        <EmptyState title="No requests yet">
          Use Request standing above to ask the admins for one.
        </EmptyState>
      {:else}
        <ul class="rows divide-y border-b">
          {#each requests as option (`${option.type}:${option.id}`)}
            <li class="flex items-center gap-3 px-2 px-2 py-3">
              <EntityAvatar type={option.type} id={option.id} name={option.name} />
              <div class="min-w-0 flex-1">
                <p class="truncate text-sm font-medium">{option.name ?? option.id}</p>
                <p class="label-caps text-[0.6875rem] text-muted-foreground">{option.type}</p>
              </div>
              <Badge variant={REQUEST_STATUS[option.status!].variant}>
                <span
                  aria-hidden="true"
                  class={['pip size-1.5', REQUEST_STATUS[option.status!].pip]}
                ></span>
                {REQUEST_STATUS[option.status!].label}
              </Badge>
            </li>
          {/each}
        </ul>
      {/if}
    </section>
  {/if}
</AppShell>
