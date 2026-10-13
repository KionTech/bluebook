<script lang="ts">
  // The document title: "<page> - <app name>".
  import { page } from '$app/state';

  import { pageTitle } from '#lib/title.ts';

  let { title }: { title?: string } = $props();
</script>

<svelte:head>
  <title>{pageTitle(title, page.data.name ?? 'Bluebook')}</title>
</svelte:head>
