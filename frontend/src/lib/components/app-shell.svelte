<script lang="ts">
  // The frame of every signed-in page: the top bar, the prompts, the
  // page heading and a single column of content.
  import type { Snippet } from 'svelte';
  import { page } from '$app/state';

  import AppLogoIcon from '#lib/components/app-logo-icon.svelte';
  import CharacterMenu from '#lib/components/character-menu.svelte';
  import Notices from '#lib/components/notices.svelte';
  import Toaster from '#lib/components/toaster.svelte';
  import { ADMIN_SECTIONS, isSectionActive, isTabActive, primarySections } from '#lib/nav.ts';

  let {
    title,
    description,
    admin = false,
    reauthNotice = true,
    actions,
    children,
  }: {
    title: string;
    description?: string;
    admin?: boolean;
    reauthNotice?: boolean;
    actions?: Snippet;
    children: Snippet;
  } = $props();

  const auth = $derived(page.data.auth);
  const sections = $derived(primarySections(Boolean(auth?.is_admin)));
  const path = $derived(page.url.pathname);
</script>

<Toaster />

<a
  href="#content"
  class="sr-only focus:not-sr-only focus:fixed focus:top-2 focus:left-2 focus:z-50 focus:border focus:border-primary focus:bg-background focus:px-3 focus:py-2"
>
  Skip to content
</a>

<header class="sticky top-0 z-40 border-b bg-background">
  <div class="mx-auto flex h-14 w-full max-w-5xl items-stretch gap-2 px-4 sm:gap-6 sm:px-6">
    <a href="/dashboard" class="flex shrink-0 items-center gap-2.5">
      <span class="chamfer flex size-8 items-center justify-center bg-primary [--cut:5px]">
        <AppLogoIcon class="size-4 text-primary-foreground" />
      </span>
      <span class="label-caps hidden text-base tracking-[0.2em] sm:inline"
        >{page.data.name ?? 'Bluebook'}</span
      >
    </a>
    <nav aria-label="Main" class="flex min-w-0 flex-1 items-stretch">
      {#each sections as section (section.href)}
        <a
          href={section.href}
          aria-current={isSectionActive(section.href, path) ? 'page' : undefined}
          class={[
            'label-caps flex items-center px-3 text-[0.8125rem] transition-colors sm:px-4',
            isSectionActive(section.href, path)
              ? 'bg-primary/8 text-primary shadow-[inset_0_-2px_0_var(--primary)]'
              : 'text-muted-foreground hover:bg-primary/5 hover:text-foreground',
          ]}
        >
          {section.title}
        </a>
      {/each}
    </nav>
    {#if auth?.active_character}
      <div class="flex items-center">
        <CharacterMenu active={auth.active_character} />
      </div>
    {/if}
  </div>
  <span
    aria-hidden="true"
    class="absolute inset-x-0 -bottom-px h-px bg-linear-to-r from-transparent via-primary/60 to-transparent"
  ></span>
</header>

<main
  id="content"
  class="mx-auto min-h-[calc(100vh-3.5rem)] w-full max-w-5xl bg-background px-4 pt-8 pb-16 sm:border-x sm:px-6"
>
  <Notices reauth={reauthNotice} />

  <div class="mb-8 flex flex-wrap items-end justify-between gap-4">
    <div class="min-w-0">
      {#if admin}
        <p class="label-caps text-xs text-primary">Admin</p>
      {/if}
      <h1 class="text-3xl leading-tight font-bold tracking-[0.08em] uppercase sm:text-4xl">
        {title}
      </h1>
      {#if description}
        <p class="mt-1 max-w-2xl text-sm text-muted-foreground">{description}</p>
      {/if}
    </div>
    {#if actions}
      <div class="flex flex-wrap items-center gap-2">{@render actions()}</div>
    {/if}
  </div>

  {#if admin}
    <nav aria-label="Admin sections" class="-mt-4 mb-8 flex overflow-x-auto border-b">
      {#each ADMIN_SECTIONS as tab (tab.href)}
        <a
          href={tab.href}
          aria-current={isTabActive(tab.href, path) ? 'page' : undefined}
          class={[
            'label-caps -mb-px border-b-2 px-4 py-2.5 text-xs whitespace-nowrap transition-colors',
            isTabActive(tab.href, path)
              ? 'border-primary bg-primary/8 text-primary'
              : 'border-transparent text-muted-foreground hover:bg-primary/5 hover:text-foreground',
          ]}
        >
          {tab.title}
        </a>
      {/each}
    </nav>
  {/if}

  {@render children()}
</main>
