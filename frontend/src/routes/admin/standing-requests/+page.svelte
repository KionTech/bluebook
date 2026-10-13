<script lang="ts">
  import { onMount } from 'svelte';
  import { goto, refreshAll } from '$app/navigation';
  import Check from '@lucide/svelte/icons/check';
  import Copy from '@lucide/svelte/icons/copy';
  import Inbox from '@lucide/svelte/icons/inbox';

  import AppShell from '#lib/components/app-shell.svelte';
  import EmptyState from '#lib/components/empty-state.svelte';
  import EntityAvatar from '#lib/components/entity-avatar.svelte';
  import FilterTabs from '#lib/components/filter-tabs.svelte';
  import PageTitle from '#lib/components/page-title.svelte';
  import Pagination from '#lib/components/pagination.svelte';
  import { Badge } from '#lib/components/ui/badge/index.ts';
  import { Button } from '#lib/components/ui/button/index.ts';
  import * as Dialog from '#lib/components/ui/dialog/index.ts';
  import { formatDateTime } from '#lib/date.ts';
  import { standingLabel, standingTextClass } from '#lib/eve.ts';
  import { send } from '#lib/form.svelte.ts';
  import { POLL_INTERVAL_MS, startPolling } from '#lib/poll.ts';
  import {
    STATUS_FILTERS,
    type StatusFilter,
    statusFilterOf,
    statusVariant,
  } from '#lib/requests.ts';
  import type { StandingRequestItem } from '#lib/types.ts';

  let { data } = $props();

  // Reload the props every 30 s so incoming standing requests appear live.
  onMount(() => startPolling(refreshAll, POLL_INTERVAL_MS));

  const statusFilter = $derived(statusFilterOf(data.filters.status));
  const filterItems = $derived(
    STATUS_FILTERS.map((filter) => ({
      value: filter,
      label: filter[0].toUpperCase() + filter.slice(1),
      count: data.counts[filter],
    })),
  );

  function setStatusFilter(filter: StatusFilter) {
    goto(
      filter === 'pending'
        ? '/admin/standing-requests'
        : `/admin/standing-requests?status=${filter}`,
      { replace: true, reset: false },
    );
  }

  let copiedId: number | null = $state(null);

  function copyName(request: StandingRequestItem) {
    const name = request.subject.name ?? String(request.subject.id);
    navigator.clipboard?.writeText(name).catch(() => {});
    copiedId = request.id;
    setTimeout(() => {
      if (copiedId === request.id) {
        copiedId = null;
      }
    }, 1500);
  }

  let openDialog: string | null = $state(null);

  function resolveRequest(request: StandingRequestItem, status: 'done' | 'rejected') {
    openDialog = null;
    send('PUT', `/admin/standing-requests/${request.id}`, { status });
  }
</script>

<PageTitle title="Standing requests" />

<AppShell
  title="Standing requests"
  description="Pilots asking for a character, corporation or alliance to be added to the standings."
  admin
>
  <div class="mb-6">
    <FilterTabs
      label="Filter by status"
      items={filterItems}
      value={statusFilter}
      onselect={setStatusFilter}
    />
  </div>

  {#if data.standingRequests.data.length === 0}
    <EmptyState
      title={statusFilter === 'pending' ? 'No pending requests' : `No ${statusFilter} requests`}
      icon={Inbox}
    >
      {#if statusFilter === 'pending'}
        New requests from pilots appear here.
      {:else}
        Requests you resolve as {statusFilter} are listed here.
      {/if}
    </EmptyState>
  {:else}
    <ul class="rows divide-y border-b">
      {#each data.standingRequests.data as request (request.id)}
        <li class="flex flex-wrap items-center gap-x-4 gap-y-3 px-2 py-4">
          <div class="flex min-w-0 flex-1 basis-64 items-center gap-3">
            <EntityAvatar
              type={request.subject.type}
              id={request.subject.id}
              name={request.subject.name}
              class="size-10"
            />
            <div class="min-w-0">
              <p class="flex items-center gap-1.5 text-sm font-medium">
                <span class="truncate">{request.subject.name ?? request.subject.id}</span>
                <span class="shrink-0 label-caps text-[0.6875rem] text-muted-foreground">
                  {request.subject.type}
                </span>
                <Button
                  type="button"
                  variant="ghost"
                  size="icon-sm"
                  class="size-7 shrink-0 text-muted-foreground"
                  aria-label={`Copy ${request.subject.name}`}
                  onclick={() => copyName(request)}
                >
                  {#if copiedId === request.id}
                    <Check />
                  {:else}
                    <Copy />
                  {/if}
                </Button>
              </p>
              <p class="truncate text-xs text-muted-foreground">
                Requested by {request.requested_by}
                <span aria-hidden="true">&middot;</span>
                <time class="font-mono text-[0.6875rem]" datetime={request.created_at}
                  >{formatDateTime(request.created_at)}</time
                >
              </p>
              <p class="text-xs">
                {#if request.effective_standing}
                  <span
                    class={[
                      'font-mono font-medium',
                      standingTextClass(request.effective_standing.standing),
                    ]}
                  >
                    {standingLabel(request.effective_standing.standing)}
                  </span>
                  <span class="text-muted-foreground">
                    {request.effective_standing.source === 'direct'
                      ? 'set directly'
                      : `inherited via ${request.effective_standing.via_name ?? request.effective_standing.via_type}`}
                  </span>
                {:else}
                  <span class="text-muted-foreground">No standing yet</span>
                {/if}
              </p>
            </div>
          </div>

          <div class="flex shrink-0 items-center gap-2">
            {#if request.status === 'pending'}
              <Dialog.Root
                open={openDialog === `done:${request.id}`}
                onOpenChange={(open) => (openDialog = open ? `done:${request.id}` : null)}
              >
                <Dialog.Trigger>
                  {#snippet child({ props })}
                    <Button type="button" size="sm" {...props}>Mark done</Button>
                  {/snippet}
                </Dialog.Trigger>
                <Dialog.Content class="sm:max-w-md">
                  <Dialog.Header>
                    <Dialog.Title>Mark request as done?</Dialog.Title>
                    <Dialog.Description>
                      Confirm that
                      <span class="font-medium text-foreground">{request.subject.name}</span>
                      has been added to the standings.
                    </Dialog.Description>
                  </Dialog.Header>
                  <Dialog.Footer>
                    <Dialog.Close>
                      {#snippet child({ props })}
                        <Button variant="outline" {...props}>Cancel</Button>
                      {/snippet}
                    </Dialog.Close>
                    <Button onclick={() => resolveRequest(request, 'done')}>Confirm</Button>
                  </Dialog.Footer>
                </Dialog.Content>
              </Dialog.Root>

              <Dialog.Root
                open={openDialog === `rejected:${request.id}`}
                onOpenChange={(open) => (openDialog = open ? `rejected:${request.id}` : null)}
              >
                <Dialog.Trigger>
                  {#snippet child({ props })}
                    <Button type="button" size="sm" variant="outline" {...props}>Reject</Button>
                  {/snippet}
                </Dialog.Trigger>
                <Dialog.Content class="sm:max-w-md">
                  <Dialog.Header>
                    <Dialog.Title>Reject request?</Dialog.Title>
                    <Dialog.Description>
                      Reject the standing request for
                      <span class="font-medium text-foreground">{request.subject.name}</span>?
                    </Dialog.Description>
                  </Dialog.Header>
                  <Dialog.Footer>
                    <Dialog.Close>
                      {#snippet child({ props })}
                        <Button variant="outline" {...props}>Cancel</Button>
                      {/snippet}
                    </Dialog.Close>
                    <Button
                      variant="destructive"
                      onclick={() => resolveRequest(request, 'rejected')}
                    >
                      Confirm
                    </Button>
                  </Dialog.Footer>
                </Dialog.Content>
              </Dialog.Root>
            {:else}
              <Badge variant={statusVariant(request.status)} class="capitalize">
                {request.status}
              </Badge>
            {/if}
          </div>
        </li>
      {/each}
    </ul>
  {/if}
  {#if data.standingRequests.last_page > 1}
    <div class="mt-6">
      <Pagination links={data.standingRequests.links} />
    </div>
  {/if}
</AppShell>
