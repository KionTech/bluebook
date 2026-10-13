<script lang="ts">
  import { onMount } from 'svelte';
  import { refreshAll } from '$app/navigation';
  import ArrowRight from '@lucide/svelte/icons/arrow-right';

  import AdminLayout from '#lib/components/layouts/admin-layout.svelte';
  import AppLayout from '#lib/components/layouts/app-layout.svelte';
  import PageTitle from '#lib/components/page-title.svelte';
  import * as Avatar from '#lib/components/ui/avatar/index.ts';
  import * as Card from '#lib/components/ui/card/index.ts';
  import { dateWithAgo } from '#lib/date.ts';
  import { eveImage, standingLabel, standingTextClass } from '#lib/eve.ts';
  import { getInitials } from '#lib/initials.ts';
  import { Now } from '#lib/now.svelte.ts';
  import { POLL_INTERVAL_MS, startPolling } from '#lib/poll.ts';
  import { nextRefresh } from '#lib/refresh.ts';

  let { data } = $props();

  const breadcrumbs = [{ title: 'Administration', href: '/admin' }];

  // Reload the props every 30 s so incoming standing requests appear live.
  onMount(() => startPolling(refreshAll, POLL_INTERVAL_MS));

  const now = new Now();
  const lastRefreshed = $derived(dateWithAgo(data.source?.last_synced_at, now.current));
  const nextRefreshIn = $derived(nextRefresh(data.source?.last_synced_at, now.current));

  const tiles = $derived([
    {
      label: 'Pending requests',
      value: data.stats.pending_requests,
      hint: data.stats.pending_requests === 0 ? 'All caught up' : 'Waiting for a decision',
      href: '/admin/standing-requests',
      attention: data.stats.pending_requests > 0,
    },
    {
      label: 'Pilots',
      value: data.stats.pilots,
      hint: 'Registered accounts',
      href: '/admin/pilots',
      attention: false,
    },
    {
      label: 'Syncing characters',
      value: data.stats.syncing_characters,
      hint: 'Mirroring the source',
      href: '/admin/pilots',
      attention: false,
    },
    {
      label: 'Source contacts',
      value: data.stats.source_contacts,
      hint: 'Canonical standings',
      href: '/admin/settings',
      attention: false,
    },
  ]);

  function subjectImage(type: string, id: number): string {
    return eveImage(type, id) ?? '';
  }
</script>

<PageTitle title="Administration" />

<AppLayout {breadcrumbs}>
  <AdminLayout title="Overview" description="The state of the standings service at a glance.">
    <!-- Highlights -->
    <div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
      {#each tiles as tile (tile.label)}
        <a
          href={tile.href}
          class="group relative overflow-hidden rounded-xl border bg-card p-5 transition-colors hover:bg-muted/50"
        >
          {#if tile.attention}
            <span class="absolute inset-x-0 top-0 h-0.5 bg-amber-500"></span>
          {/if}
          <p class="text-sm text-muted-foreground">{tile.label}</p>
          <p class="mt-2 text-3xl font-semibold tracking-tight tabular-nums">{tile.value}</p>
          <p class="mt-1 flex items-center gap-1 text-xs text-muted-foreground">
            {tile.hint}
            <ArrowRight class="h-3 w-3 opacity-0 transition-opacity group-hover:opacity-100" />
          </p>
        </a>
      {/each}
    </div>

    <div class="grid gap-6 lg:grid-cols-2">
      <!-- Source status -->
      <Card.Root>
        <Card.Header>
          <Card.Title>Standings source</Card.Title>
          <Card.Description>The entity every character mirrors.</Card.Description>
        </Card.Header>
        <Card.Content>
          {#if data.source}
            <div class="flex items-center gap-4">
              <Avatar.Root class="h-12 w-12 shrink-0 overflow-hidden rounded-lg">
                <Avatar.Image
                  src={subjectImage(data.source.type, data.source.entity_id)}
                  alt={data.source.entity_name ?? ''}
                />
                <Avatar.Fallback class="rounded-lg text-xs uppercase">
                  {data.source.type.slice(0, 2)}
                </Avatar.Fallback>
              </Avatar.Root>
              <div class="min-w-0 flex-1">
                <p class="truncate text-sm font-medium">
                  {data.source.entity_name ?? data.source.entity_id}
                </p>
                <p class="text-xs text-muted-foreground capitalize">{data.source.type}</p>
              </div>
              <div class="text-right text-xs">
                <p class="text-muted-foreground">Refreshed {lastRefreshed}</p>
                {#if nextRefreshIn}
                  <p class="mt-0.5 font-medium tabular-nums">Next in {nextRefreshIn}</p>
                {/if}
              </div>
            </div>
          {:else}
            <p class="text-sm text-muted-foreground">
              No source is configured yet. Set one in the settings to start syncing.
            </p>
          {/if}
          <a
            href="/admin/settings"
            class="mt-4 inline-flex items-center gap-1 text-sm text-muted-foreground transition-colors hover:text-foreground"
          >
            Manage source
            <ArrowRight class="h-3.5 w-3.5" />
          </a>
        </Card.Content>
      </Card.Root>

      <!-- Recent pending requests -->
      <Card.Root>
        <Card.Header>
          <Card.Title>Pending requests</Card.Title>
          <Card.Description>The latest standing requests waiting for a decision.</Card.Description>
        </Card.Header>
        <Card.Content>
          {#if data.recentRequests.length === 0}
            <p class="py-6 text-center text-sm text-muted-foreground">
              Nothing waiting. New requests appear here.
            </p>
          {:else}
            <ul class="-mx-2">
              {#each data.recentRequests as request (request.id)}
                <li
                  class="flex items-center gap-3 rounded-md px-2 py-2 transition-colors odd:bg-muted/40"
                >
                  <Avatar.Root class="h-8 w-8 shrink-0 overflow-hidden rounded">
                    <Avatar.Image
                      src={subjectImage(request.subject.type, request.subject.id)}
                      alt={request.subject.name ?? ''}
                    />
                    <Avatar.Fallback class="rounded text-xs">
                      {getInitials(request.subject.name ?? '?')}
                    </Avatar.Fallback>
                  </Avatar.Root>
                  <div class="min-w-0 flex-1">
                    <p class="truncate text-sm font-medium">
                      {request.subject.name ?? request.subject.id}
                    </p>
                    <p class="truncate text-xs text-muted-foreground">
                      Requested by {request.requested_by}
                    </p>
                  </div>
                  {#if request.effective_standing}
                    <span
                      class={[
                        'shrink-0 text-xs font-medium',
                        standingTextClass(request.effective_standing.standing),
                      ]}
                    >
                      {standingLabel(request.effective_standing.standing)}
                    </span>
                  {/if}
                </li>
              {/each}
            </ul>
          {/if}
          <a
            href="/admin/standing-requests"
            class="mt-4 inline-flex items-center gap-1 text-sm text-muted-foreground transition-colors hover:text-foreground"
          >
            Review requests
            <ArrowRight class="h-3.5 w-3.5" />
          </a>
        </Card.Content>
      </Card.Root>
    </div>
  </AdminLayout>
</AppLayout>
