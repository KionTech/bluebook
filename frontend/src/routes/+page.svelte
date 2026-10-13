<script lang="ts">
  import ArrowRight from '@lucide/svelte/icons/arrow-right';

  import AppLogoIcon from '#lib/components/app-logo-icon.svelte';
  import PageTitle from '#lib/components/page-title.svelte';
  import { Button } from '#lib/components/ui/button/index.ts';

  let { data } = $props();
</script>

<PageTitle title="Bluebook" />

<div class="flex min-h-screen flex-col bg-background px-4 text-foreground">
  <main class="flex flex-1 flex-col items-center justify-center text-center">
    <span class="chamfer flex size-16 items-center justify-center bg-primary [--cut:10px]">
      <AppLogoIcon class="size-8 text-primary-foreground" />
    </span>
    <p class="label-caps mt-8 text-xs text-primary">Standings service</p>
    <h1 class="mt-2 text-5xl font-bold tracking-[0.2em] uppercase">Bluebook</h1>
    <span
      aria-hidden="true"
      class="mt-4 h-px w-40 bg-linear-to-r from-transparent via-primary to-transparent"
    ></span>
    <p class="mt-4 max-w-xs text-sm text-muted-foreground">
      Alliance standings on every character you fly. Authorized pilots only.
    </p>

    {#if data.auth?.user}
      <Button href="/dashboard" size="lg" class="mt-8">
        Enter
        <ArrowRight />
      </Button>
    {:else}
      <Button href="/eve" data-sveltekit-reload size="lg" class="mt-8">
        Sign in with EVE Online
        <ArrowRight />
      </Button>
      {#if data.demoLoginEnabled}
        <a
          href="/demo-login"
          data-sveltekit-reload
          class="mt-4 text-sm text-muted-foreground underline-offset-4 hover:text-foreground hover:underline"
        >
          Demo login
        </a>
      {/if}
    {/if}
  </main>

  <footer class="label-caps py-6 text-center text-[0.625rem] text-muted-foreground">
    EVE Online and all related trademarks are property of CCP hf.
  </footer>
</div>
