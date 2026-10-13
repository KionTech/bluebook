<script lang="ts">
  // The prompts above a page: an admin's unreadable standings source, and
  // (away from the dashboard, which asks per character) the characters
  // whose token lapsed.
  import { page } from '$app/state';
  import KeyRound from '@lucide/svelte/icons/key-round';
  import TriangleAlert from '@lucide/svelte/icons/triangle-alert';

  import { Button } from '#lib/components/ui/button/index.ts';

  let { reauth = true }: { reauth?: boolean } = $props();

  const auth = $derived(page.data.auth);
  const sourceUnreadable = $derived(Boolean(auth?.user && auth.is_admin && auth.source_unreadable));
  const reauthCharacters = $derived(reauth && auth?.user ? (auth.reauth_characters ?? []) : []);
  const reauthNames = $derived(
    reauthCharacters.map((character: { name: string | null }) => character.name).join(', '),
  );
</script>

{#if sourceUnreadable || reauthCharacters.length > 0}
  <div class="mb-8 space-y-3">
    {#if sourceUnreadable}
      <div
        role="alert"
        class="frame flex flex-col gap-3 border-destructive/60 bg-destructive/5 px-4 py-3 shadow-[inset_3px_0_0_var(--destructive)] sm:flex-row sm:items-center"
      >
        <TriangleAlert class="size-5 shrink-0 text-destructive" />
        <div class="flex-1 text-sm">
          <p class="label-caps text-destructive">The standings source cannot be read.</p>
          <p class="text-muted-foreground">
            No admin character can read it: its token is gone or it lacks the in-game role to see
            the contacts. Syncing is paused until the source's admin character logs in again or gets
            the role back.
          </p>
        </div>
        <Button href={auth?.admin_scopes_url ?? undefined} data-sveltekit-reload size="sm">
          <KeyRound />
          Re-authenticate source
        </Button>
      </div>
    {/if}

    {#if reauthCharacters.length > 0}
      <div
        role="alert"
        class="frame flex flex-col gap-3 border-warning-border bg-warning-soft px-4 py-3 shadow-[inset_3px_0_0_var(--warning)] sm:flex-row sm:items-center"
      >
        <KeyRound class="size-5 shrink-0 text-warning" />
        <p class="flex-1 text-sm">
          <span class="font-medium">{reauthNames}</span>
          {reauthCharacters.length === 1 ? 'needs' : 'need'} to log in again before syncing can continue.
        </p>
        <Button href={auth?.sync_scopes_url} data-sveltekit-reload size="sm" variant="outline">
          Log in again
        </Button>
      </div>
    {/if}
  </div>
{/if}
