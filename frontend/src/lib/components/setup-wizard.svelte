<script lang="ts">
  // The first-login wizard: what Bluebook does, adding every character,
  // how the sync works, and picking the main character.
  import { onMount, untrack } from 'svelte';
  import Check from '@lucide/svelte/icons/check';
  import MessageSquarePlus from '@lucide/svelte/icons/message-square-plus';
  import Plus from '@lucide/svelte/icons/plus';
  import RefreshCw from '@lucide/svelte/icons/refresh-cw';
  import Star from '@lucide/svelte/icons/star';
  import Users from '@lucide/svelte/icons/users';

  import * as Avatar from '#lib/components/ui/avatar/index.ts';
  import { Button } from '#lib/components/ui/button/index.ts';
  import * as Dialog from '#lib/components/ui/dialog/index.ts';
  import { eveImage } from '#lib/eve.ts';
  import { send } from '#lib/form.svelte.ts';
  import { getInitials } from '#lib/initials.ts';
  import { STORAGE_KEY, TOTAL_STEPS, storedStep } from '#lib/wizard.ts';

  let { characters }: { characters: { id: number; name: string }[] } = $props();

  let open = $state(true);
  let step = $state(0);
  let selectedMainId: number | null = $state(
    untrack(() => (characters.length === 1 ? characters[0].id : null)),
  );
  const isLastStep = $derived(step === TOTAL_STEPS - 1);

  onMount(() => {
    step = storedStep(localStorage);
  });

  function setStep(value: number) {
    step = value;
    localStorage.setItem(STORAGE_KEY, String(value));
  }

  function next() {
    if (!isLastStep) {
      setStep(step + 1);
    }
  }

  function back() {
    if (step > 0) {
      setStep(step - 1);
    }
  }

  function close() {
    open = false;
    step = 0;
    localStorage.removeItem(STORAGE_KEY);
  }

  async function skip() {
    if ((await send('POST', '/setup/complete')).ok) {
      close();
    }
  }

  async function finish() {
    if (selectedMainId === null) {
      await skip();
      return;
    }
    if ((await send('PUT', `/auth/character/${selectedMainId}/main`)).ok) {
      close();
    }
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="sm:max-w-lg">
    <Dialog.Header>
      <Dialog.Title>
        {#if step === 0}Welcome, pilot{:else if step === 1}Add all your characters{:else if step === 2}Standings
          sync themselves{:else}Pick your main character{/if}
      </Dialog.Title>
      <Dialog.Description class="label-caps text-[0.6875rem] text-primary"
        >Step {step + 1} of {TOTAL_STEPS}</Dialog.Description
      >
    </Dialog.Header>

    {#if step === 0}
      <div class="space-y-4 text-sm">
        <p>
          This app keeps your in-game contacts in line with the alliance's official standings,
          automatically, for every character you register.
        </p>
        <ul class="space-y-3">
          <li class="flex items-start gap-3">
            <Users class="mt-0.5 h-4 w-4 shrink-0 text-muted-foreground" />
            <span>You add all your characters to one account.</span>
          </li>
          <li class="flex items-start gap-3">
            <RefreshCw class="mt-0.5 h-4 w-4 shrink-0 text-muted-foreground" />
            <span>
              The alliance's standings are copied onto each character's contact list and kept up to
              date.
            </span>
          </li>
          <li class="flex items-start gap-3">
            <MessageSquarePlus class="mt-0.5 h-4 w-4 shrink-0 text-muted-foreground" />
            <span>
              If one of your characters isn't blue yet, you can request a standing right from the
              dashboard.
            </span>
          </li>
        </ul>
      </div>
    {:else if step === 1}
      <div class="space-y-4 text-sm">
        <p>
          Every character you want to keep in sync needs to be on this account, alts included. Log
          in with each one once and it joins automatically.
        </p>
        <div>
          <p class="mb-2 text-xs text-muted-foreground">Currently on your account:</p>
          <ul class="max-h-40 space-y-1 overflow-y-auto">
            {#each characters as character (character.id)}
              <li class="flex items-center gap-3 border px-3 py-2">
                <Avatar.Root class="h-7 w-7 overflow-hidden border border-border">
                  <Avatar.Image src={eveImage('character', character.id)} alt={character.name} />
                  <Avatar.Fallback class="rounded-none text-xs">
                    {getInitials(character.name)}
                  </Avatar.Fallback>
                </Avatar.Root>
                <span class="text-sm font-medium">{character.name}</span>
              </li>
            {/each}
          </ul>
        </div>
        <Button href="/eve?add_to_account=1" data-sveltekit-reload variant="outline" class="w-full">
          <Plus class="h-4 w-4" />
          Add another character
        </Button>
        <p class="text-xs text-muted-foreground">
          You can always add more later from the character menu in the top right. Already added
          everyone? Continue.
        </p>
      </div>
    {:else if step === 2}
      <div class="space-y-4 text-sm">
        <p>
          Each character on the dashboard has a sync switch. Grant the character contact
          permissions, turn the switch on, and their in-game contacts mirror the alliance's
          standings within minutes.
        </p>
        <ul class="space-y-3">
          <li class="flex items-start gap-3">
            <Check class="mt-0.5 h-4 w-4 shrink-0 text-muted-foreground" />
            <span>
              Characters already in the alliance inherit the standings in-game, so there is nothing
              to do for those.
            </span>
          </li>
          <li class="flex items-start gap-3">
            <Check class="mt-0.5 h-4 w-4 shrink-0 text-muted-foreground" />
            <span>
              For everyone else, use
              <span class="font-medium">Request standing</span>. An admin reviews it, and once
              approved the standings unlock for you.
            </span>
          </li>
        </ul>
      </div>
    {:else}
      <div class="space-y-4 text-sm">
        <p>
          Your main is the character the admins will recognize you by. Your alts are grouped under
          it in the pilot roster.
        </p>
        <ul class="max-h-56 space-y-1 overflow-y-auto">
          {#each characters as character (character.id)}
            <li>
              <button
                type="button"
                class={[
                  'flex w-full items-center gap-3 border px-3 py-2 text-left transition-[background-color,border-color,box-shadow] hover:border-border-strong hover:bg-primary/5',
                  selectedMainId === character.id &&
                    'border-primary bg-primary/10 shadow-[inset_2px_0_0_var(--primary)]',
                ]}
                data-test={`wizard-character-${character.id}`}
                onclick={() => (selectedMainId = character.id)}
              >
                <Avatar.Root class="h-8 w-8 overflow-hidden border border-border">
                  <Avatar.Image src={eveImage('character', character.id)} alt={character.name} />
                  <Avatar.Fallback class="rounded-none text-xs">
                    {getInitials(character.name)}
                  </Avatar.Fallback>
                </Avatar.Root>
                <span class="flex-1 text-sm font-medium">{character.name}</span>
                {#if selectedMainId === character.id}
                  <Check class="h-4 w-4 text-primary" />
                {/if}
              </button>
            </li>
          {/each}
        </ul>
        <p class="text-xs text-muted-foreground">
          You can change this later with the star next to a character on the dashboard.
        </p>
      </div>
    {/if}

    <div class="flex items-center justify-between gap-2 pt-2">
      <Button type="button" variant="ghost" size="sm" class="text-muted-foreground" onclick={skip}>
        Skip for now
      </Button>
      <div class="flex items-center gap-2">
        {#if step > 0}
          <Button type="button" variant="outline" size="sm" onclick={back}>Back</Button>
        {/if}
        {#if !isLastStep}
          <Button type="button" size="sm" onclick={next}>Next</Button>
        {:else}
          <Button type="button" size="sm" disabled={selectedMainId === null} onclick={finish}>
            <Star class="h-4 w-4" />
            Set main & finish
          </Button>
        {/if}
      </div>
    </div>
  </Dialog.Content>
</Dialog.Root>
