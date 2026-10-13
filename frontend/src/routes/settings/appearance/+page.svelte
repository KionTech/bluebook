<script lang="ts">
  import AppShell from '#lib/components/app-shell.svelte';
  import AppearanceTabs from '#lib/components/appearance-tabs.svelte';
  import PageTitle from '#lib/components/page-title.svelte';
</script>

<PageTitle title="Appearance settings" />

<AppShell
  title="Appearance"
  description="Pick a light or dark theme, or follow your system. The choice is kept in this browser."
>
  <AppearanceTabs />
</AppShell>
