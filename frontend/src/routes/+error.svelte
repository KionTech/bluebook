<script lang="ts">
  import { page } from '$app/state';

  import { errorTitle } from '#lib/errors.ts';
</script>

<svelte:head>
  <title>{errorTitle(page.status)}</title>
</svelte:head>

<main class="flex min-h-screen items-center justify-center bg-background px-4 text-foreground">
  <div class="max-w-md text-center">
    <p class="font-mono text-sm text-primary">ERR {page.status}</p>
    <h1 class="mt-2 text-3xl font-bold tracking-[0.08em] uppercase">
      {page.error?.message ?? errorTitle(page.status)}
    </h1>
    <p class="mt-6">
      <a
        href="/"
        data-sveltekit-reload
        class="text-sm font-medium text-primary underline-offset-4 hover:underline"
      >
        Back to Bluebook
      </a>
    </p>
  </div>
</main>
