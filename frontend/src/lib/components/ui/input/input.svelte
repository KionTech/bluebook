<script lang="ts">
  import { cn, type WithElementRef } from '#lib/utils.ts';
  import type { HTMLInputAttributes, HTMLInputTypeAttribute } from 'svelte/elements';

  type InputType = Exclude<HTMLInputTypeAttribute, 'file'>;

  type Props = WithElementRef<
    Omit<HTMLInputAttributes, 'type'> &
      ({ type: 'file'; files?: FileList } | { type?: InputType; files?: undefined })
  >;

  let {
    ref = $bindable(null),
    value = $bindable(),
    type,
    files = $bindable(),
    class: className,
    'data-slot': dataSlot = 'input',
    ...restProps
  }: Props = $props();
</script>

{#if type === 'file'}
  <input
    bind:this={ref}
    data-slot={dataSlot}
    class={cn(
      'file:text-foreground placeholder:text-muted-foreground selection:bg-primary selection:text-primary-foreground border-input h-9 w-full min-w-0 border bg-card/60 px-3 py-1 text-base transition-[color,border-color,box-shadow] duration-150 hover:border-border-strong file:inline-flex file:h-7 file:border-0 file:bg-transparent file:text-sm file:font-medium disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm focus-visible:border-primary focus-visible:outline-1 focus-visible:outline-offset-0 focus-visible:shadow-[0_0_12px_var(--glow)] aria-invalid:border-destructive',
      className,
    )}
    type="file"
    bind:files
    bind:value
    {...restProps}
  />
{:else}
  <input
    bind:this={ref}
    data-slot={dataSlot}
    class={cn(
      'file:text-foreground placeholder:text-muted-foreground selection:bg-primary selection:text-primary-foreground border-input h-9 w-full min-w-0 border bg-card/60 px-3 py-1 text-base transition-[color,border-color,box-shadow] duration-150 hover:border-border-strong file:inline-flex file:h-7 file:border-0 file:bg-transparent file:text-sm file:font-medium disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm focus-visible:border-primary focus-visible:outline-1 focus-visible:outline-offset-0 focus-visible:shadow-[0_0_12px_var(--glow)] aria-invalid:border-destructive',
      className,
    )}
    {type}
    bind:value
    {...restProps}
  />
{/if}
