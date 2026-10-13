<script lang="ts">
  import { onMount } from 'svelte';
  import { goto, refreshAll } from '$app/navigation';
  import Check from '@lucide/svelte/icons/check';
  import Copy from '@lucide/svelte/icons/copy';

  import AdminLayout from '#lib/components/layouts/admin-layout.svelte';
  import AppLayout from '#lib/components/layouts/app-layout.svelte';
  import PageTitle from '#lib/components/page-title.svelte';
  import Pagination from '#lib/components/pagination.svelte';
  import * as Avatar from '#lib/components/ui/avatar/index.ts';
  import { Badge } from '#lib/components/ui/badge/index.ts';
  import { Button } from '#lib/components/ui/button/index.ts';
  import * as Card from '#lib/components/ui/card/index.ts';
  import * as Dialog from '#lib/components/ui/dialog/index.ts';
  import { eveImage, standingLabel, standingTextClass } from '#lib/eve.ts';
  import { send } from '#lib/form.svelte.ts';
  import { getInitials } from '#lib/initials.ts';
  import { POLL_INTERVAL_MS, startPolling } from '#lib/poll.ts';
  import {
    STATUS_FILTERS,
    type StatusFilter,
    statusFilterOf,
    statusVariant,
  } from '#lib/requests.ts';
  import type { StandingRequestItem } from '#lib/types.ts';

  let { data } = $props();

  const breadcrumbs = [
    { title: 'Administration', href: '/admin' },
    { title: 'Standing requests', href: '/admin/standing-requests' },
  ];

  // Reload the props every 30 s so incoming standing requests appear live.
  onMount(() => startPolling(refreshAll, POLL_INTERVAL_MS));

  const statusFilter = $derived(statusFilterOf(data.filters.status));

  function setStatusFilter(filter: StatusFilter) {
    goto(
      filter === 'pending'
        ? '/admin/standing-requests'
        : `/admin/standing-requests?status=${filter}`,
      {
        replace: true,
        reset: false,
      },
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

  function subjectImage(type: string, id: number): string {
    return eveImage(type, id) ?? '';
  }
</script>

<PageTitle title="Standing requests" />

<AppLayout {breadcrumbs}>
  <AdminLayout
    title="Standing requests"
    description="Pilots asking for a character, corporation or alliance to be added to the standings."
  >
    {#snippet actions()}
      <div
        class="flex items-center gap-1 rounded-lg border bg-card p-1"
        role="group"
        aria-label="Filter by status"
      >
        {#each STATUS_FILTERS as filter (filter)}
          <Button
            type="button"
            size="sm"
            variant={statusFilter === filter ? 'secondary' : 'ghost'}
            class="h-7 px-2.5 text-xs capitalize"
            onclick={() => setStatusFilter(filter)}
          >
            {filter}
            <span class="text-muted-foreground tabular-nums">{data.counts[filter]}</span>
          </Button>
        {/each}
      </div>
    {/snippet}

    <Card.Root>
      <Card.Content>
        {#if data.standingRequests.data.length === 0}
          <p class="py-10 text-center text-sm text-muted-foreground">
            {#if statusFilter === 'pending'}
              No pending standing requests. New requests from pilots appear here.
            {:else}
              No {statusFilter} requests.
            {/if}
          </p>
        {:else}
          <ul class="-mx-2">
            {#each data.standingRequests.data as request (request.id)}
              <li
                class="flex items-center gap-4 rounded-md px-2 py-3 transition-colors odd:bg-muted/40 hover:bg-muted"
              >
                <Avatar.Root class="h-9 w-9 overflow-hidden rounded">
                  <Avatar.Image
                    src={subjectImage(request.subject.type, request.subject.id)}
                    alt={request.subject.name ?? ''}
                  />
                  <Avatar.Fallback class="rounded text-xs">
                    {getInitials(request.subject.name ?? '?')}
                  </Avatar.Fallback>
                </Avatar.Root>
                <div class="flex-1">
                  <p class="text-sm font-medium">
                    {request.subject.name ?? request.subject.id}
                    <span class="text-xs text-muted-foreground capitalize">
                      ({request.subject.type})
                    </span>
                  </p>
                  <p class="text-xs text-muted-foreground">Requested by {request.requested_by}</p>
                  <p class="text-xs">
                    {#if request.effective_standing}
                      <span
                        class={[
                          'font-medium',
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

                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  class="h-8 w-8"
                  aria-label={`Copy ${request.subject.name}`}
                  onclick={() => copyName(request)}
                >
                  {#if copiedId === request.id}
                    <Check class="h-4 w-4" />
                  {:else}
                    <Copy class="h-4 w-4" />
                  {/if}
                </Button>

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
                  <Badge variant={statusVariant(request.status)}>{request.status}</Badge>
                {/if}
              </li>
            {/each}
          </ul>
        {/if}
        {#if data.standingRequests.last_page > 1}
          <div class="mt-4 border-t pt-4">
            <Pagination links={data.standingRequests.links} />
          </div>
        {/if}
      </Card.Content>
    </Card.Root>
  </AdminLayout>
</AppLayout>
