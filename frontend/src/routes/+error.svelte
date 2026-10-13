<script lang="ts">
  import { page } from '$app/state';

  import { errorTitle } from '#lib/errors.ts';
</script>

<svelte:head>
  <title>{errorTitle(page.status)}</title>
</svelte:head>

<div
  class="relative flex min-h-screen items-top justify-center bg-white text-black sm:items-center sm:pt-0"
  role="main"
>
  <div class="mx-auto max-w-xl sm:px-6 lg:px-8">
    <div class="flex items-center pt-8 sm:justify-start sm:pt-0">
      <h1 class="border-r border-gray-400 px-4 text-lg text-black">{page.status}</h1>
      <div class="ml-4 text-lg text-black">{page.error?.message ?? errorTitle(page.status)}</div>
    </div>
  </div>
</div>
