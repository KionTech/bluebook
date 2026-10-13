<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';

  import { initializeTheme } from '#lib/appearance.ts';

  let { children } = $props();

  onMount(() => initializeTheme());
</script>

{@render children()}
