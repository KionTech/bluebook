<script lang="ts">
  // The light, dark or system choice, kept in localStorage and the
  // appearance cookie.
  import { onMount } from 'svelte';
  import Monitor from '@lucide/svelte/icons/monitor';
  import Moon from '@lucide/svelte/icons/moon';
  import Sun from '@lucide/svelte/icons/sun';

  import { type Appearance, storedAppearance, updateAppearance } from '#lib/appearance.ts';

  let appearance: Appearance = $state('dark');

  onMount(() => {
    appearance = storedAppearance(localStorage);
  });

  const options = [
    { value: 'light', Icon: Sun, label: 'Light' },
    { value: 'dark', Icon: Moon, label: 'Dark' },
    { value: 'system', Icon: Monitor, label: 'System' },
  ] as const;

  function choose(value: Appearance) {
    appearance = value;
    updateAppearance(value);
  }
</script>

<div class="grid max-w-md grid-cols-3 gap-2" role="group" aria-label="Theme">
  {#each options as { value, Icon, label } (value)}
    <button
      type="button"
      aria-pressed={appearance === value}
      onclick={() => choose(value)}
      class={[
        'label-caps flex flex-col items-center gap-2 border px-3 py-4 text-xs transition-[background-color,border-color,box-shadow]',
        appearance === value
          ? 'border-primary bg-primary/10 text-primary shadow-[0_0_14px_var(--glow)]'
          : 'bg-card/60 text-muted-foreground hover:border-border-strong hover:text-foreground',
      ]}
    >
      <Icon class="size-5" />
      {label}
    </button>
  {/each}
</div>
