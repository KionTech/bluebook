<script lang="ts">
  import { untrack } from 'svelte';
  import Mail from '@lucide/svelte/icons/mail';
  import RefreshCw from '@lucide/svelte/icons/refresh-cw';

  import AppShell from '#lib/components/app-shell.svelte';
  import EmptyState from '#lib/components/empty-state.svelte';
  import EntityAvatar from '#lib/components/entity-avatar.svelte';
  import PageTitle from '#lib/components/page-title.svelte';
  import SectionHeading from '#lib/components/section-heading.svelte';
  import { Button } from '#lib/components/ui/button/index.ts';
  import * as Dialog from '#lib/components/ui/dialog/index.ts';
  import { Input } from '#lib/components/ui/input/index.ts';
  import { Label } from '#lib/components/ui/label/index.ts';
  import * as Select from '#lib/components/ui/select/index.ts';
  import { dateWithAgo } from '#lib/date.ts';
  import { Form, send } from '#lib/form.svelte.ts';
  import { Now } from '#lib/now.svelte.ts';
  import { nextRefresh } from '#lib/refresh.ts';

  let { data } = $props();

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
</script>

<PageTitle title="Admin settings" />

<AppShell title="Settings" description="The standings source and the notifications." admin>
  <section class="mb-12" aria-labelledby="source-heading">
    <SectionHeading
      id="source-heading"
      title="Standings source"
      description="The character, corporation or alliance whose contacts every pilot mirrors."
    >
      {#snippet actions()}
        {#if data.source}
          <Button type="button" onclick={syncNow}>
            <RefreshCw />
            Sync now
          </Button>
        {/if}
        <Dialog.Root bind:open={changeSourceOpen}>
          <Dialog.Trigger>
            {#snippet child({ props })}
              <Button type="button" variant={data.source ? 'outline' : 'default'} {...props}>
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
                      <span class="text-muted-foreground">
                        Select your character, corp or alliance
                      </span>
                    {/if}
                  </Select.Trigger>
                  <Select.Content>
                    {#each data.sourceTypes as type (type.value)}
                      <Select.Item value={type.value} label={type.label} disabled={!type.available}>
                        {type.label}
                        {#if type.entity_name}
                          <span class="text-muted-foreground">({type.entity_name})</span>
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
                <Button
                  type="submit"
                  variant={data.source ? 'destructive' : 'default'}
                  disabled={sourceForm.processing}
                >
                  {data.source ? 'Change source' : 'Save source'}
                </Button>
              </Dialog.Footer>
            </form>
          </Dialog.Content>
        </Dialog.Root>
      {/snippet}
    </SectionHeading>

    {#if !data.adminCharacter}
      <p role="alert" class="frame tone-error tone-bar mb-4 px-4 py-3 text-sm [--cut:8px]">
        Your active character could not be resolved.
      </p>
    {/if}

    {#if data.source}
      <dl class="frame grid grid-cols-2 gap-x-6 gap-y-4 px-5 py-4 text-sm sm:grid-cols-4">
        <div class="col-span-2 min-w-0 sm:col-span-1">
          <dt class="label-caps text-[0.6875rem] text-muted-foreground">{data.source.type}</dt>
          <dd class="flex min-w-0 items-center gap-2 font-medium">
            <EntityAvatar
              type={data.source.type}
              id={data.source.entity_id}
              name={data.source.entity_name}
              class="size-6"
            />
            <span class="truncate">{data.source.entity_name ?? data.source.entity_id}</span>
          </dd>
        </div>
        <div>
          <dt class="label-caps text-[0.6875rem] text-muted-foreground">Canonical contacts</dt>
          <dd class="font-mono text-base font-medium text-primary">{data.contactsCount}</dd>
        </div>
        <div>
          <dt class="label-caps text-[0.6875rem] text-muted-foreground">Last refreshed</dt>
          <dd class="font-mono text-[0.8125rem]">{lastRefreshed}</dd>
        </div>
        {#if nextRefreshIn}
          <div>
            <dt class="label-caps text-[0.6875rem] text-muted-foreground">Next refresh</dt>
            <dd class="font-mono text-[0.8125rem]">{nextRefreshIn}</dd>
          </div>
        {/if}
      </dl>
    {:else}
      <EmptyState title="No source is set">
        Nothing syncs until you pick one. Use Set source above.
      </EmptyState>
    {/if}

    {#if data.adminCharacter && !data.adminCharacter.has_mail_scope}
      <div
        class="frame tone-warning tone-bar mt-6 flex flex-col gap-3 px-4 py-3 sm:flex-row sm:items-center"
      >
        <Mail class="size-5 shrink-0" />
        <p class="flex-1 text-sm">
          <span class="font-medium">In-game expiry mail is off.</span>
          <span>
            Grant the mail permission so pilots whose tokens expired get an EVE mail asking them to
            log in again.
          </span>
        </p>
        <Button href={data.grantMailScopeUrl} data-sveltekit-reload size="sm" variant="tone">
          Grant mail permission
        </Button>
      </div>
    {:else if data.adminCharacter}
      <p class="mt-4 flex items-center gap-2 text-sm text-muted-foreground">
        <Mail class="size-4" />
        In-game expiry mail is sent by {data.adminCharacter.name}.
      </p>
    {/if}
  </section>

  <section aria-labelledby="discord-heading">
    <SectionHeading
      id="discord-heading"
      title="Discord notifications"
      description="Where standing requests and an unreadable source are announced. Optionally ping a role."
    />
    <form class="max-w-xl space-y-4" onsubmit={saveDiscord}>
      <div class="space-y-2">
        <Label for="webhook_url">Webhook URL</Label>
        <Input
          id="webhook_url"
          name="webhook_url"
          bind:value={discordForm.data.webhook_url}
          type="url"
          placeholder="https://discord.com/api/webhooks/..."
          aria-invalid={discordForm.errors.webhook_url ? true : undefined}
          aria-describedby={discordForm.errors.webhook_url ? 'webhook_url-error' : undefined}
        />
        {#if discordForm.errors.webhook_url}
          <p id="webhook_url-error" class="text-sm text-destructive">
            {discordForm.errors.webhook_url}
          </p>
        {/if}
      </div>
      <div class="space-y-2">
        <Label for="role_id">Role to ping (optional)</Label>
        <Input
          id="role_id"
          name="role_id"
          bind:value={discordForm.data.role_id}
          placeholder="Role ID"
          class="sm:max-w-60"
          aria-invalid={discordForm.errors.role_id ? true : undefined}
          aria-describedby={discordForm.errors.role_id ? 'role_id-error' : undefined}
        />
        {#if discordForm.errors.role_id}
          <p id="role_id-error" class="text-sm text-destructive">{discordForm.errors.role_id}</p>
        {/if}
      </div>
      <Button type="submit" disabled={discordForm.processing}>Save Discord settings</Button>
    </form>
  </section>
</AppShell>
