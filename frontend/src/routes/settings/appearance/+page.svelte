<script lang="ts">
  import AppearanceTabs from '#lib/components/appearance-tabs.svelte';
  import Heading from '#lib/components/heading.svelte';
  import AppLayout from '#lib/components/layouts/app-layout.svelte';
  import SettingsLayout from '#lib/components/layouts/settings-layout.svelte';
  import PageTitle from '#lib/components/page-title.svelte';

  const breadcrumbs = [{ title: 'Appearance settings', href: '/settings/appearance' }];
</script>

<PageTitle title="Appearance settings" />

<AppLayout {breadcrumbs}>
  <h1 class="sr-only">Appearance settings</h1>

  <SettingsLayout>
    <div class="space-y-6">
      <Heading
        variant="small"
        title="Appearance settings"
        description="Update your account's appearance settings"
      />
      <AppearanceTabs />
    </div>
  </SettingsLayout>
</AppLayout>
