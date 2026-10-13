<script lang="ts">
  // The administration pages: their heading, actions and section tabs.
  import type { Snippet } from 'svelte';
  import { page } from '$app/state';

  let {
    title,
    description,
    actions,
    children,
  }: { title: string; description?: string; actions?: Snippet; children: Snippet } = $props();

  const tabs = [
    { title: 'Overview', href: '/admin' },
    { title: 'Standing requests', href: '/admin/standing-requests' },
    { title: 'Pilots', href: '/admin/pilots' },
    { title: 'Settings', href: '/admin/settings' },
  ];
</script>

<div class="flex flex-1 flex-col">
  <div class="border-b px-4 pt-6 sm:px-6">
    <p class="text-xs font-medium tracking-[0.2em] text-muted-foreground uppercase">
      Administration
    </p>
    <div class="mt-1.5 flex flex-wrap items-end justify-between gap-4">
      <div>
        <h1 class="text-2xl font-semibold tracking-tight">{title}</h1>
        {#if description}
          <p class="mt-1 text-sm text-muted-foreground">{description}</p>
        {/if}
      </div>
      <div class="pb-1">
        {@render actions?.()}
      </div>
    </div>
    <nav class="mt-4 -mb-px flex gap-1 overflow-x-auto" aria-label="Administration sections">
      {#each tabs as tab (tab.href)}
        <a
          href={tab.href}
          class={[
            'border-b-2 px-3 py-2 text-sm whitespace-nowrap transition-colors',
            page.url.pathname === tab.href
              ? 'border-primary font-medium text-foreground'
              : 'border-transparent text-muted-foreground hover:border-border hover:text-foreground',
          ]}
        >
          {tab.title}
        </a>
      {/each}
    </nav>
  </div>
  <div class="flex flex-1 flex-col gap-6 p-4 sm:p-6">
    {@render children()}
  </div>
</div>
