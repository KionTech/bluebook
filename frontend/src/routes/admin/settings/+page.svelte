<script lang="ts">
  import { untrack } from 'svelte';
  import Mail from '@lucide/svelte/icons/mail';
  import RefreshCw from '@lucide/svelte/icons/refresh-cw';

  import AdminLayout from '#lib/components/layouts/admin-layout.svelte';
  import AppLayout from '#lib/components/layouts/app-layout.svelte';
  import PageTitle from '#lib/components/page-title.svelte';
  import * as Avatar from '#lib/components/ui/avatar/index.ts';
  import { Button } from '#lib/components/ui/button/index.ts';
  import * as Card from '#lib/components/ui/card/index.ts';
  import * as Dialog from '#lib/components/ui/dialog/index.ts';
  import { Input } from '#lib/components/ui/input/index.ts';
  import { Label } from '#lib/components/ui/label/index.ts';
  import * as Select from '#lib/components/ui/select/index.ts';
  import { dateWithAgo } from '#lib/date.ts';
  import { eveImage } from '#lib/eve.ts';
  import { Form, send } from '#lib/form.svelte.ts';
  import { Now } from '#lib/now.svelte.ts';
  import { nextRefresh } from '#lib/refresh.ts';

  let { data } = $props();

  const breadcrumbs = [
    { title: 'Administration', href: '/admin' },
    { title: 'Settings', href: '/admin/settings' },
  ];

  const now = new Now();
  const lastRefreshed = $derived(dateWithAgo(data.source?.last_synced_at, now.current));
  const nextRefreshIn = $derived(nextRefresh(data.source?.last_synced_at, now.current));

  const sourceForm = new Form(untrack(() => ({ type: data.source?.type ?? '' })));
  const discordForm = new Form(
    untrack(() => ({
      webhook_url: data.discordSettings.webhook_url ?? '',
      role_id: data.discordSettings.role_id ?? '',
    })),
  );

  let changeSourceOpen = $state(false);
  const selectedLabel = $derived(
    data.sourceTypes.find((type) => type.value === sourceForm.data.type)?.label,
  );

  async function saveSource(event: SubmitEvent) {
    event.preventDefault();
    if (await sourceForm.submit('PUT', '/admin/standings')) {
      changeSourceOpen = false;
    }
  }

  async function saveDiscord(event: SubmitEvent) {
    event.preventDefault();
    await discordForm.submit('PUT', '/admin/discord-settings');
  }

  function syncNow() {
    send('POST', '/admin/standings/sync');
  }

  function subjectImage(type: string, id: number): string {
    return eveImage(type, id) ?? '';
  }
</script>

<PageTitle title="Admin settings" />

<AppLayout {breadcrumbs}>
  <AdminLayout title="Settings" description="Configure the standings source and notifications.">
    <div class="grid gap-6 lg:grid-cols-2">
      <!-- Source (read-only with confirm-to-change) -->
      <Card.Root>
        <Card.Header class="flex flex-row items-start justify-between gap-4">
          <div class="flex items-center gap-3">
            {#if data.source}
              <Avatar.Root class="h-10 w-10 shrink-0 overflow-hidden rounded">
                <Avatar.Image
                  src={subjectImage(data.source.type, data.source.entity_id)}
                  alt={data.source.entity_name ?? ''}
                />
                <Avatar.Fallback class="rounded text-xs uppercase">
                  {data.source.type.slice(0, 2)}
                </Avatar.Fallback>
              </Avatar.Root>
            {/if}
            <div>
              <Card.Title>Standings source</Card.Title>
              <Card.Description>
                {#if data.source}
                  <span class="font-medium text-foreground"
                    >{data.source.entity_name ?? data.source.entity_id}</span
                  >
                  <span class="capitalize">&middot; {data.source.type}</span>
                {:else}
                  No source has been configured yet.
                {/if}
              </Card.Description>
            </div>
          </div>
          <div class="flex shrink-0 items-center gap-2">
            <Dialog.Root bind:open={changeSourceOpen}>
              <Dialog.Trigger>
                {#snippet child({ props })}
                  <Button
                    type="button"
                    size="sm"
                    variant={data.source ? 'destructive' : 'default'}
                    {...props}
                  >
                    {data.source ? 'Change source' : 'Set source'}
                  </Button>
                {/snippet}
              </Dialog.Trigger>
              <Dialog.Content class="sm:max-w-md">
                <Dialog.Header>
                  <Dialog.Title>
                    {data.source ? 'Change standings source?' : 'Set standings source'}
                  </Dialog.Title>
                  <Dialog.Description>
                    {#if data.source}
                      Changing the source clears the current standings and re-syncs every character.
                    {:else}
                      Select which of your character, corporation or alliance defines the canonical
                      standings.
                    {/if}
                  </Dialog.Description>
                </Dialog.Header>
                <form class="space-y-4" onsubmit={saveSource}>
                  <div class="space-y-2">
                    <Label for="type">Source</Label>
                    <Select.Root type="single" bind:value={sourceForm.data.type}>
                      <Select.Trigger id="type" class="w-full">
                        {#if selectedLabel}
                          {selectedLabel}
                        {:else}
                          <span class="text-muted-foreground"
                            >Select your character, corp or alliance</span
                          >
                        {/if}
                      </Select.Trigger>
                      <Select.Content>
                        {#each data.sourceTypes as type (type.value)}
                          <Select.Item
                            value={type.value}
                            label={type.label}
                            disabled={!type.available}
                          >
                            {type.label}
                            {#if type.entity_name}
                              <span class="text-muted-foreground">&mdash; {type.entity_name}</span>
                            {:else if !type.has_scope}
                              <span class="text-muted-foreground">(no permission)</span>
                            {:else}
                              <span class="text-muted-foreground">(unavailable)</span>
                            {/if}
                          </Select.Item>
                        {/each}
                      </Select.Content>
                    </Select.Root>
                    {#if sourceForm.errors.type}
                      <p class="text-sm text-destructive">{sourceForm.errors.type}</p>
                    {/if}
                  </div>
                  <Dialog.Footer>
                    <Dialog.Close>
                      {#snippet child({ props })}
                        <Button type="button" variant="outline" {...props}>Cancel</Button>
                      {/snippet}
                    </Dialog.Close>
                    <Button type="submit" disabled={sourceForm.processing}>
                      {data.source ? 'Change source' : 'Save source'}
                    </Button>
                  </Dialog.Footer>
                </form>
              </Dialog.Content>
            </Dialog.Root>
            {#if data.source}
              <Button type="button" variant="secondary" size="sm" onclick={syncNow}>
                <RefreshCw class="h-4 w-4" />
                Sync now
              </Button>
            {/if}
          </div>
        </Card.Header>
        <Card.Content>
          {#if !data.adminCharacter}
            <div
              class="mb-4 rounded-md border border-destructive/40 bg-destructive/10 px-4 py-3 text-sm text-destructive"
            >
              Your active character could not be resolved.
            </div>
          {/if}

          {#if data.source}
            <dl class="grid gap-3 text-sm sm:grid-cols-3">
              <div>
                <dt class="text-muted-foreground">Canonical contacts</dt>
                <dd class="font-medium">{data.contactsCount}</dd>
              </div>
              <div>
                <dt class="text-muted-foreground">Last refreshed</dt>
                <dd class="font-medium">{lastRefreshed}</dd>
              </div>
              {#if nextRefreshIn}
                <div>
                  <dt class="text-muted-foreground">Next refresh</dt>
                  <dd class="font-medium tabular-nums">{nextRefreshIn}</dd>
                </div>
              {/if}
            </dl>
          {:else}
            <p class="text-sm text-muted-foreground">No source is configured yet.</p>
          {/if}

          {#if data.adminCharacter && !data.adminCharacter.has_mail_scope}
            <div
              class="mt-4 flex flex-wrap items-center justify-between gap-3 rounded-md border bg-muted/40 px-4 py-3"
            >
              <div class="flex items-start gap-3 text-sm">
                <Mail class="mt-0.5 h-4 w-4 shrink-0 text-muted-foreground" />
                <p class="text-muted-foreground">
                  <span class="font-medium text-foreground">In-game expiry mail is off.</span>
                  Grant the mail permission so pilots whose tokens expired get an EVE mail asking them
                  to re-authenticate.
                </p>
              </div>
              <Button
                href={data.grantMailScopeUrl}
                data-sveltekit-reload
                size="sm"
                variant="outline"
              >
                Grant mail permission
              </Button>
            </div>
          {:else if data.adminCharacter}
            <p class="mt-4 flex items-center gap-2 text-xs text-muted-foreground">
              <Mail class="h-3.5 w-3.5" />
              In-game expiry mail is enabled for {data.adminCharacter.name}.
            </p>
          {/if}
        </Card.Content>
      </Card.Root>

      <!-- Discord notifications -->
      <Card.Root>
        <Card.Header>
          <Card.Title>Discord notifications</Card.Title>
          <Card.Description>
            Where standing requests are announced. Optionally ping a role.
          </Card.Description>
        </Card.Header>
        <Card.Content>
          <form class="space-y-4" onsubmit={saveDiscord}>
            <div class="grid gap-4 sm:grid-cols-[2fr_1fr]">
              <div class="space-y-2">
                <Label for="webhook_url">Webhook URL</Label>
                <Input
                  id="webhook_url"
                  name="webhook_url"
                  bind:value={discordForm.data.webhook_url}
                  type="url"
                  placeholder="https://discord.com/api/webhooks/..."
                />
                {#if discordForm.errors.webhook_url}
                  <p class="text-sm text-destructive">{discordForm.errors.webhook_url}</p>
                {/if}
              </div>
              <div class="space-y-2">
                <Label for="role_id">Role to ping (optional)</Label>
                <Input
                  id="role_id"
                  name="role_id"
                  bind:value={discordForm.data.role_id}
                  placeholder="Role ID"
                />
                {#if discordForm.errors.role_id}
                  <p class="text-sm text-destructive">{discordForm.errors.role_id}</p>
                {/if}
              </div>
            </div>
            <div class="flex justify-end">
              <Button type="submit" disabled={discordForm.processing}>Save Discord settings</Button>
            </div>
          </form>
        </Card.Content>
      </Card.Root>
    </div>
  </AdminLayout>
</AppLayout>
