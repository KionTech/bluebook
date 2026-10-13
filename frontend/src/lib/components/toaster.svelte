<script lang="ts">
  // The toasts at the top of the page: each flash message a page arrives
  // with, and the errors of refused forms.
  import { page } from '$app/state';
  import { untrack } from 'svelte';
  import { fly } from 'svelte/transition';
  import CircleCheck from '@lucide/svelte/icons/circle-check';
  import CircleX from '@lucide/svelte/icons/circle-x';
  import Info from '@lucide/svelte/icons/info';
  import X from '@lucide/svelte/icons/x';

  import { toaster } from '#lib/toaster.svelte.ts';

  const icon = { success: CircleCheck, error: CircleX, info: Info };
  const accent = {
    success: 'text-emerald-500',
    error: 'text-destructive',
    info: 'text-sky-500',
  };

  // Every page load brings its own flash; showing it must not make the
  // effect depend on the toasts themselves.
  $effect(() => {
    const flash = page.data.flash;
    untrack(() => toaster.showFlash(flash));
  });
</script>

<div
  class="pointer-events-none fixed top-4 left-1/2 z-[100] flex w-full max-w-sm -translate-x-1/2 flex-col items-center gap-2"
>
  {#each toaster.items as toast (toast.id)}
    {@const Icon = icon[toast.type]}
    <div
      transition:fly={{ y: -8, duration: 200 }}
      class="pointer-events-auto flex items-start gap-3 rounded-lg border bg-background p-3 shadow-lg"
      role="status"
    >
      <Icon class={['mt-0.5 h-5 w-5 shrink-0', accent[toast.type]]} />
      <p class="flex-1 text-sm">{toast.message}</p>
      <button
        type="button"
        class="text-muted-foreground transition-colors hover:text-foreground"
        aria-label="Dismiss"
        onclick={() => toaster.dismiss(toast.id)}
      >
        <X class="h-4 w-4" />
      </button>
    </div>
  {/each}
</div>
