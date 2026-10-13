<script lang="ts">
  // The settings pages: their navigation beside the content.
  import type { Snippet } from 'svelte';
  import { page } from '$app/state';

  import Heading from '#lib/components/heading.svelte';
  import { Button } from '#lib/components/ui/button/index.ts';
  import { Separator } from '#lib/components/ui/separator/index.ts';

  let { children }: { children: Snippet } = $props();

  const items = [{ title: 'Appearance', href: '/settings/appearance' }];
</script>

<div class="px-4 py-6">
  <Heading title="Settings" description="Manage your account settings" />

  <div class="flex flex-col lg:flex-row lg:space-x-12">
    <aside class="w-full max-w-xl lg:w-48">
      <nav class="flex flex-col space-y-1 space-x-0" aria-label="Settings">
        {#each items as item (item.href)}
          <Button
            variant="ghost"
            href={item.href}
            class={['w-full justify-start', page.url.pathname.startsWith(item.href) && 'bg-muted']}
          >
            {item.title}
          </Button>
        {/each}
      </nav>
    </aside>

    <Separator class="my-6 lg:hidden" />

    <div class="flex-1 md:max-w-2xl">
      <section class="max-w-xl space-y-12">
        {@render children()}
      </section>
    </div>
  </div>
</div>
