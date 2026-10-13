<script lang="ts">
  import { onMount } from 'svelte';
  import { refreshAll } from '$app/navigation';
  import ArrowRight from '@lucide/svelte/icons/arrow-right';

  import AppShell from '#lib/components/app-shell.svelte';
  import EmptyState from '#lib/components/empty-state.svelte';
  import EntityAvatar from '#lib/components/entity-avatar.svelte';
  import PageTitle from '#lib/components/page-title.svelte';
  import SectionHeading from '#lib/components/section-heading.svelte';
  import { Button } from '#lib/components/ui/button/index.ts';
  import { dateWithAgo } from '#lib/date.ts';
  import { standingLabel, standingTextClass } from '#lib/eve.ts';
  import { Now } from '#lib/now.svelte.ts';
  import { POLL_INTERVAL_MS, startPolling } from '#lib/poll.ts';
  import { nextRefresh } from '#lib/refresh.ts';

  let { data } = $props();

  // Reload the props every 30 s so incoming standing requests appear live.
  onMount(() => startPolling(refreshAll, POLL_INTERVAL_MS));

  const now = new Now();
  const lastRefreshed = $derived(dateWithAgo(data.source?.last_synced_at, now.current));
  const nextRefreshIn = $derived(nextRefresh(data.source?.last_synced_at, now.current));

  const stats = $derived([
    {
      label: 'Pending requests',
      value: data.stats.pending_requests,
      href: '/admin/standing-requests',
      attention: data.stats.pending_requests > 0,
    },
    { label: 'Pilots', value: data.stats.pilots, href: '/admin/pilots', attention: false },
    {
      label: 'Syncing characters',
      value: data.stats.syncing_characters,
      href: '/admin/pilots',
      attention: false,
    },
    {
      label: 'Source contacts',
      value: data.stats.source_contacts,
      href: '/admin/settings',
      attention: false,
    },
  ]);
</script>

<PageTitle title="Administration" />

<AppShell title="Overview" description="The state of the standings service at a glance." admin>
  <ul class="mb-12 grid grid-cols-2 gap-3 sm:grid-cols-4" aria-label="Statistics">
    {#each stats as stat (stat.label)}
      <li>
        <a
          href={stat.href}
          class="frame flex h-full flex-col gap-1 px-4 py-4 transition-colors hover:bg-accent hover:[--edge:var(--primary)]"
        >
          <span
            class={[
              'font-mono text-3xl font-medium',
              stat.attention ? 'text-primary' : 'text-foreground',
            ]}
          >
            {stat.value}
          </span>
          <span class="label-caps text-[0.6875rem] text-muted-foreground">{stat.label}</span>
        </a>
      </li>
    {/each}
  </ul>

  <section class="mb-12" aria-labelledby="source-heading">
    <SectionHeading id="source-heading" title="Standings source">
      {#snippet actions()}
        <Button href="/admin/settings" variant="outline" size="sm">Manage source</Button>
      {/snippet}
    </SectionHeading>
    {#if data.source}
      <div class="frame flex flex-wrap items-center gap-x-6 gap-y-3 px-5 py-4 text-sm">
        <div class="flex min-w-0 flex-1 items-center gap-3">
          <EntityAvatar
            type={data.source.type}
            id={data.source.entity_id}
            name={data.source.entity_name}
            class="size-10"
          />
          <div class="min-w-0">
            <p class="truncate font-medium">{data.source.entity_name ?? data.source.entity_id}</p>
            <p class="label-caps text-[0.6875rem] text-muted-foreground">{data.source.type}</p>
          </div>
        </div>
        <p class="font-mono text-[0.8125rem] text-muted-foreground">Refreshed {lastRefreshed}</p>
        {#if nextRefreshIn}
          <p class="font-mono text-[0.8125rem]">Next in {nextRefreshIn}</p>
        {/if}
      </div>
    {:else}
      <EmptyState title="No source is set">
        Pick the character, corporation or alliance whose contacts everyone mirrors.
        <a
          href="/admin/settings"
          class="font-medium text-primary underline-offset-4 hover:underline"
        >
          Set the source
        </a>
      </EmptyState>
    {/if}
  </section>

  <section aria-labelledby="pending-heading">
    <SectionHeading
      id="pending-heading"
      title="Pending requests"
      description="The latest standing requests waiting for a decision."
    >
      {#snippet actions()}
        <Button href="/admin/standing-requests" variant="outline" size="sm">
          Review requests
          <ArrowRight />
        </Button>
      {/snippet}
    </SectionHeading>
    {#if data.recentRequests.length === 0}
      <EmptyState title="Nothing waiting">New requests from pilots appear here.</EmptyState>
    {:else}
      <ul class="rows divide-y border-b">
        {#each data.recentRequests as request (request.id)}
          <li class="flex items-center gap-3 px-2 py-3">
            <EntityAvatar
              type={request.subject.type}
              id={request.subject.id}
              name={request.subject.name}
            />
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
                  'shrink-0 font-mono text-sm font-medium',
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
  </section>
</AppShell>
