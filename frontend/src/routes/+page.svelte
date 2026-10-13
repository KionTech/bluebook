<script lang="ts">
  import ArrowRight from '@lucide/svelte/icons/arrow-right';

  import AppLogoIcon from '#lib/components/app-logo-icon.svelte';
  import PageTitle from '#lib/components/page-title.svelte';

  let { data } = $props();
</script>

<PageTitle title="Bluebook" />

<div
  class="relative flex min-h-screen flex-col items-center justify-center overflow-hidden bg-neutral-950 px-6 text-neutral-200 antialiased"
>
  <!-- Ambient glow -->
  <div
    class="pointer-events-none absolute top-1/3 left-1/2 h-[42rem] w-[42rem] -translate-x-1/2 -translate-y-1/2 rounded-full bg-sky-500/10 blur-[130px]"
  ></div>
  <div
    class="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_center,transparent_30%,rgba(0,0,0,0.7)_100%)]"
  ></div>

  <main
    class="relative z-10 flex w-full max-w-sm flex-col items-center text-center opacity-100 transition-opacity duration-700 starting:opacity-0"
  >
    <div
      class="flex h-16 w-16 items-center justify-center rounded-2xl border border-white/10 bg-white/5 shadow-2xl shadow-black/40 backdrop-blur"
    >
      <AppLogoIcon class="h-8 w-8 text-white" />
    </div>

    <h1 class="mt-8 text-3xl font-semibold tracking-tight text-white">Bluebook</h1>
    <p class="mt-3 text-sm text-neutral-500">Authorized pilots only.</p>

    {#if data.auth?.user}
      <a
        href="/dashboard"
        class="group mt-10 inline-flex items-center justify-center gap-2 rounded-lg bg-white px-6 py-2.5 text-sm font-medium text-neutral-900 transition hover:bg-neutral-200"
      >
        Enter
        <ArrowRight class="h-4 w-4 transition-transform group-hover:translate-x-0.5" />
      </a>
    {:else}
      <a
        href="/eve"
        data-sveltekit-reload
        class="group mt-10 inline-flex items-center justify-center gap-2 rounded-lg border border-white/15 bg-white/5 px-6 py-2.5 text-sm font-medium text-white transition hover:border-white/30 hover:bg-white/10"
      >
        Sign in with EVE Online
        <ArrowRight class="h-4 w-4 transition-transform group-hover:translate-x-0.5" />
      </a>
      {#if data.demoLoginEnabled}
        <a
          href="/demo-login"
          data-sveltekit-reload
          class="mt-4 text-sm text-neutral-400 underline-offset-4 transition hover:text-white hover:underline"
        >
          Demo login
        </a>
      {/if}
    {/if}
  </main>

  <footer class="absolute bottom-6 z-10 px-6 text-center text-[11px] text-neutral-700">
    EVE Online and all related trademarks are property of CCP hf.
  </footer>
</div>
