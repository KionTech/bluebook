<script lang="ts">
  import { onMount } from 'svelte';
  import { refreshAll } from '$app/navigation';
  import KeyRound from '@lucide/svelte/icons/key-round';
  import Lock from '@lucide/svelte/icons/lock';
  import Plus from '@lucide/svelte/icons/plus';
  import RefreshCw from '@lucide/svelte/icons/refresh-cw';
  import Search from '@lucide/svelte/icons/search';
  import Star from '@lucide/svelte/icons/star';

  import AppLayout from '#lib/components/layouts/app-layout.svelte';
  import PageTitle from '#lib/components/page-title.svelte';
  import SetupWizard from '#lib/components/setup-wizard.svelte';
  import * as Avatar from '#lib/components/ui/avatar/index.ts';
  import { Badge } from '#lib/components/ui/badge/index.ts';
  import { Button } from '#lib/components/ui/button/index.ts';
  import * as Card from '#lib/components/ui/card/index.ts';
  import * as Dialog from '#lib/components/ui/dialog/index.ts';
  import { Input } from '#lib/components/ui/input/index.ts';
  import { Switch } from '#lib/components/ui/switch/index.ts';
  import { dateWithAgo } from '#lib/date.ts';
  import { eveImage, standingLabel, standingTextClass } from '#lib/eve.ts';
  import { send } from '#lib/form.svelte.ts';
  import { getInitials } from '#lib/initials.ts';
  import { Now } from '#lib/now.svelte.ts';
  import { POLL_INTERVAL_MS, startPolling } from '#lib/poll.ts';
  import { nextRefresh } from '#lib/refresh.ts';
  import { canRequest, filterStandings, optionStatusLabel, sortStandings } from '#lib/standings.ts';
  import type { RequestOption, SyncCharacter } from '#lib/types.ts';

  let { data } = $props();

  const breadcrumbs = [{ title: 'Dashboard', href: '/dashboard' }];

  // Reload the props every 30 s so standings and characters stay live.
  onMount(() => startPolling(refreshAll, POLL_INTERVAL_MS));

  const now = new Now();
  const lastRefreshed = $derived(dateWithAgo(data.source?.last_synced_at, now.current));
  const nextRefreshIn = $derived(nextRefresh(data.source?.last_synced_at, now.current));

  let standingsSearch = $state('');
  const sortedStandings = $derived(sortStandings(data.standings ?? []));
  const filteredStandings = $derived(filterStandings(sortedStandings, standingsSearch));

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

<AppLayout {breadcrumbs}>
  {#if data.showSetupWizard}
    <SetupWizard characters={data.characters} />
  {/if}
  <div class="flex flex-1 flex-col gap-6 p-4">
    <!-- Source -->
    <Card.Root>
      <Card.Header class="flex flex-row items-center gap-3">
        {#if data.source}
          <Avatar.Root class="h-10 w-10 shrink-0 overflow-hidden rounded">
            {#if eveImage(data.source.type, data.source.entity_id)}
              <Avatar.Image
                src={eveImage(data.source.type, data.source.entity_id)}
                alt={data.source.entity_name ?? ''}
              />
            {/if}
            <Avatar.Fallback class="rounded text-xs uppercase">
              {data.source.type.slice(0, 2)}
            </Avatar.Fallback>
          </Avatar.Root>
        {/if}
        <div>
          <Card.Title>Standings source</Card.Title>
          <Card.Description>
            {#if data.source}
              Mirroring
              <span class="font-medium text-foreground"
                >{data.source.entity_name ?? data.source.type}</span
              >'s {data.source.type} contacts onto every opted-in character.
            {:else}
              No source has been configured yet.
            {/if}
          </Card.Description>
        </div>
      </Card.Header>
      <Card.Content>
        <dl class="grid gap-3 text-sm sm:grid-cols-3">
          {#if data.standings}
            <div>
              <dt class="text-muted-foreground">Standings</dt>
              <dd class="font-medium">{data.standings.length}</dd>
            </div>
          {/if}
          <div>
            <dt class="text-muted-foreground">Last refreshed</dt>
            <dd class="font-medium">{lastRefreshed}</dd>
          </div>
          {#if nextRefreshIn}
            <div>
              <dt class="text-muted-foreground">Next refresh</dt>
              <dd class="font-medium tabular-nums">{nextRefreshIn}</dd>
            </div>
          {/if}
        </dl>
      </Card.Content>
    </Card.Root>

    <!-- Current standings overview -->
    <Card.Root>
      <Card.Header>
        <Card.Title>Current standings</Card.Title>
        <Card.Description>
          The canonical standings pulled from the source. These are applied to every syncing
          character.
        </Card.Description>
      </Card.Header>
      <Card.Content>
        {#if !data.canViewStandings}
          <div class="flex flex-col items-center gap-2 py-10 text-center">
            <Lock class="h-5 w-5 text-muted-foreground" />
            <p class="text-sm font-medium">Standings are hidden</p>
            <p class="max-w-sm text-sm text-muted-foreground">
              None of your characters is eligible yet. Request a standing for a character,
              corporation or alliance below &mdash; once an admin approves it, the standings unlock.
            </p>
          </div>
        {:else if !data.standings || data.standings.length === 0}
          <p class="py-8 text-center text-sm text-muted-foreground">
            No standings have been pulled from the source yet.
          </p>
        {:else}
          <div class="relative mb-3">
            <Search
              class="absolute top-1/2 left-2.5 h-4 w-4 -translate-y-1/2 text-muted-foreground"
            />
            <Input
              bind:value={standingsSearch}
              type="search"
              name="standings-search"
              placeholder="Search standings..."
              aria-label="Search standings"
              class="pl-8"
            />
          </div>
          {#if filteredStandings.length === 0}
            <p class="py-8 text-center text-sm text-muted-foreground">
              No standings match your search.
            </p>
          {:else}
            <ul class="-mx-2">
              {#each filteredStandings as standing (standing.contact_id)}
                <li
                  class="flex items-center gap-3 rounded-md px-2 py-1.5 transition-colors odd:bg-muted/40 hover:bg-muted"
                >
                  <Avatar.Root class="h-5 w-5 shrink-0 overflow-hidden rounded-sm">
                    {#if eveImage(standing.contact_type, standing.contact_id)}
                      <Avatar.Image
                        src={eveImage(standing.contact_type, standing.contact_id)}
                        alt={`${standing.contact_id}`}
                      />
                    {/if}
                    <Avatar.Fallback class="rounded-sm text-[8px] uppercase">
                      {standing.contact_type.slice(0, 2)}
                    </Avatar.Fallback>
                  </Avatar.Root>
                  <div class="min-w-0 flex-1">
                    <p class="truncate text-sm font-medium">
                      {standing.name ?? standing.contact_id}
                    </p>
                    {#if standing.redundant_via}
                      <p
                        class="truncate text-xs text-amber-600 dark:text-amber-400"
                        title={`This standing already applies through the ${standing.redundant_via.contact_type} and can be removed from the source.`}
                      >
                        Redundant via
                        {standing.redundant_via.name ?? standing.redundant_via.contact_id}
                      </p>
                    {/if}
                  </div>
                  <span class="flex-1 truncate text-xs text-muted-foreground capitalize">
                    {standing.contact_type}
                  </span>
                  <span
                    class={[
                      'w-12 shrink-0 text-right text-sm font-semibold tabular-nums',
                      standingTextClass(standing.standing),
                    ]}
                  >
                    {standingLabel(standing.standing)}
                  </span>
                </li>
              {/each}
            </ul>
          {/if}
        {/if}
      </Card.Content>
    </Card.Root>

    <!-- Your characters -->
    <Card.Root>
      <Card.Header class="flex flex-row items-start justify-between gap-4">
        <div>
          <Card.Title>Your characters</Card.Title>
          <Card.Description>
            Choose which characters mirror the source's standings, and star your main character.
          </Card.Description>
        </div>
        <div class="flex items-center gap-2">
          {#if data.requestableOptions.length > 0}
            <Dialog.Root bind:open={requestOpen}>
              <Dialog.Trigger>
                {#snippet child({ props })}
                  <Button type="button" variant="outline" size="sm" {...props}>
                    <Plus class="h-4 w-4" />
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
                        class="flex w-full items-center gap-3 rounded-md px-2 py-2 text-left transition-colors hover:bg-muted disabled:pointer-events-none disabled:opacity-60"
                        onclick={() => requestStanding(option)}
                      >
                        <Avatar.Root class="h-8 w-8 shrink-0 overflow-hidden rounded">
                          {#if eveImage(option.type, option.id)}
                            <Avatar.Image
                              src={eveImage(option.type, option.id)}
                              alt={option.name ?? ''}
                            />
                          {/if}
                          <Avatar.Fallback class="rounded text-[10px] uppercase">
                            {option.type.slice(0, 2)}
                          </Avatar.Fallback>
                        </Avatar.Root>
                        <div class="flex-1">
                          <p class="text-sm font-medium">{option.name ?? option.id}</p>
                          <p class="text-xs text-muted-foreground capitalize">{option.type}</p>
                        </div>
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
          <Button type="button" variant="secondary" size="sm" onclick={syncMyCharacters}>
            <RefreshCw class="h-4 w-4" />
            Sync now
          </Button>
        </div>
      </Card.Header>
      <Card.Content>
        <ul class="-mx-2">
          {#each data.characters as character (character.id)}
            <li
              class="flex items-center gap-4 rounded-md px-2 py-3 transition-colors odd:bg-muted/40 hover:bg-muted"
            >
              <Avatar.Root class="h-9 w-9 overflow-hidden rounded">
                <Avatar.Image src={eveImage('character', character.id)} alt={character.name} />
                <Avatar.Fallback class="rounded text-xs"
                  >{getInitials(character.name)}</Avatar.Fallback
                >
              </Avatar.Root>
              <div class="flex-1">
                <p class="flex items-center gap-2 text-sm font-medium">
                  {character.name}
                  {#if character.is_main}
                    <Badge variant="secondary">Main</Badge>
                  {/if}
                </p>
                <div class="mt-0.5 flex items-center gap-2 text-xs text-muted-foreground">
                  {#if character.corporation}
                    <span class="flex items-center gap-1">
                      <img
                        src={eveImage('corporation', character.corporation.id)}
                        alt={character.corporation.name ?? ''}
                        class="h-4 w-4 rounded-sm"
                      />
                      {character.corporation.name}
                    </span>
                  {/if}
                  {#if character.alliance}
                    <span class="flex items-center gap-1">
                      <img
                        src={eveImage('alliance', character.alliance.id)}
                        alt={character.alliance.name ?? ''}
                        class="h-4 w-4 rounded-sm"
                      />
                      {character.alliance.name}
                    </span>
                  {/if}
                </div>
              </div>

              <div class="flex items-center gap-3">
                {#if !character.is_main}
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    class="h-7 w-7 text-muted-foreground"
                    title="Set as main character"
                    aria-label={`Set ${character.name} as your main character`}
                    onclick={() => setMainCharacter(character)}
                  >
                    <Star class="h-4 w-4" />
                  </Button>
                {/if}
                <span class="text-xs text-muted-foreground">
                  {#if character.inherits_source}
                    Inherits the source
                  {:else if character.should_sync && character.has_write_scope}
                    {character.synced_contacts_count} synced
                  {/if}
                </span>
                {#if character.inherits_source}
                  <Badge variant="outline">Inherited</Badge>
                {:else if !character.has_write_scope}
                  <a
                    href={data.auth.sync_scopes_url}
                    data-sveltekit-reload
                    class="inline-flex items-center gap-1 text-xs font-medium text-amber-600 hover:underline dark:text-amber-400"
                    title={`Grant ${character.name} the contact permissions needed to sync standings`}
                  >
                    <KeyRound class="h-3 w-3" />
                    Grant sync access
                  </a>
                {:else}
                  <Switch
                    checked={character.should_sync}
                    aria-label={`Toggle standings sync for ${character.name}`}
                    onCheckedChange={(value) => toggleSync(character, value === true)}
                  />
                {/if}
              </div>
            </li>
          {/each}
        </ul>
      </Card.Content>
    </Card.Root>
  </div>
</AppLayout>
