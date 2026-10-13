<script lang="ts">
  import { Switch as SwitchPrimitive } from 'bits-ui';
  import { cn, type WithoutChildrenOrChild } from '#lib/utils.ts';

  let {
    ref = $bindable(null),
    class: className,
    checked = $bindable(false),
    size = 'default',
    ...restProps
  }: WithoutChildrenOrChild<SwitchPrimitive.RootProps> & {
    size?: 'sm' | 'default';
  } = $props();
</script>

<SwitchPrimitive.Root
  bind:ref
  bind:checked
  data-slot="switch"
  data-size={size}
  class={cn(
    'data-[state=checked]:border-primary data-[state=checked]:bg-primary/25 data-[state=checked]:shadow-[0_0_10px_var(--glow)] data-[state=unchecked]:border-border-strong data-[state=unchecked]:bg-muted aria-invalid:border-destructive shrink-0 border data-[size=default]:h-5 data-[size=default]:w-10 data-[size=sm]:h-[14px] data-[size=sm]:w-[26px] peer group/switch relative inline-flex items-center transition-[background-color,border-color,box-shadow] duration-150 after:absolute after:-inset-x-3 after:-inset-y-2 data-disabled:cursor-not-allowed data-disabled:opacity-50',
    className,
  )}
  {...restProps}
>
  <SwitchPrimitive.Thumb
    data-slot="switch-thumb"
    class={cn(
      'data-[state=checked]:bg-primary data-[state=unchecked]:bg-muted-foreground group-data-[size=default]/switch:size-3 group-data-[size=sm]/switch:size-2 data-[state=checked]:translate-x-[1.4375rem] data-[state=unchecked]:translate-x-[0.1875rem] pointer-events-none block transition-[translate,background-color] duration-150',
      'rtl:data-[state=checked]:translate-x-[calc(-100%)]',
    )}
  />
</SwitchPrimitive.Root>
