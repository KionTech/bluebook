<script lang="ts">
  // The banners for an unreadable source (admins) and for characters that
  // sync without the contact permissions.
  import { page } from '$app/state';
  import KeyRound from '@lucide/svelte/icons/key-round';
  import TriangleAlert from '@lucide/svelte/icons/triangle-alert';

  const auth = $derived(page.data.auth);
  const sourceUnreadable = $derived(Boolean(auth?.user && auth.is_admin && auth.source_unreadable));
  const reauthCharacters = $derived(auth?.user ? (auth.reauth_characters ?? []) : []);
  const reauthNames = $derived(reauthCharacters.map((character) => character.name).join(', '));
</script>

{#if sourceUnreadable || reauthCharacters.length > 0}
  <div class="space-y-3 p-4 pb-0">
    {#if sourceUnreadable}
      <div
        class="flex flex-col gap-3 rounded-lg border border-destructive/40 bg-destructive/10 p-4 text-destructive sm:flex-row sm:items-center"
      >
        <TriangleAlert class="h-5 w-5 shrink-0" />
        <div class="flex-1">
          <p class="font-medium">The standings source can't be read.</p>
          <p class="text-sm opacity-90">
            No admin character has a valid token for it &mdash; syncing is paused until it's
            re-authenticated.
          </p>
        </div>
        <a
          href={auth?.admin_scopes_url ?? undefined}
          data-sveltekit-reload
          class="inline-flex shrink-0 items-center justify-center gap-2 rounded-md bg-destructive px-4 py-2 text-sm font-medium text-destructive-foreground transition hover:bg-destructive/90"
        >
          <KeyRound class="h-4 w-4" />
          Re-authenticate
        </a>
      </div>
    {/if}

    {#if reauthCharacters.length > 0}
      <div
        class="flex flex-col gap-3 rounded-lg border border-amber-500/40 bg-amber-500/10 p-4 text-amber-700 sm:flex-row sm:items-center dark:text-amber-400"
      >
        <TriangleAlert class="h-5 w-5 shrink-0" />
        <div class="flex-1">
          <p class="font-medium">
            {reauthCharacters.length} character(s) need re-authentication.
          </p>
          <p class="text-sm opacity-90">
            <span class="font-medium">{reauthNames}</span> can't sync standings until you grant contact
            permissions.
          </p>
        </div>
        <a
          href={auth?.sync_scopes_url}
          data-sveltekit-reload
          class="inline-flex shrink-0 items-center justify-center gap-2 rounded-md border border-amber-500/50 px-4 py-2 text-sm font-medium transition hover:bg-amber-500/10"
        >
          <KeyRound class="h-4 w-4" />
          Grant access
        </a>
      </div>
    {/if}
  </div>
{/if}
