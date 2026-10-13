<script lang="ts" module>
  import { type VariantProps, tv } from 'tailwind-variants';

  export const badgeVariants = tv({
    base: 'inline-flex items-center justify-center gap-1.5 border px-1.5 py-0.5 font-display text-[0.6875rem] font-semibold uppercase leading-none tracking-[0.12em] w-fit whitespace-nowrap shrink-0 [&>svg]:size-3 [&>svg]:pointer-events-none aria-invalid:border-destructive transition-[color,box-shadow] overflow-hidden',
    variants: {
      variant: {
        default: 'border-primary bg-primary/15 text-primary [a&]:hover:bg-primary/25',
        secondary:
          'border-border-strong bg-secondary text-secondary-foreground [a&]:hover:bg-secondary/70',
        destructive:
          'border-destructive/60 bg-destructive/10 text-destructive [a&]:hover:bg-destructive/20',
        outline:
          'border-border-strong text-foreground [a&]:hover:bg-accent [a&]:hover:text-accent-foreground',
      },
    },
    defaultVariants: {
      variant: 'default',
    },
  });

  export type BadgeVariant = VariantProps<typeof badgeVariants>['variant'];
</script>

<script lang="ts">
  import { cn, type WithElementRef } from '#lib/utils.ts';
  import type { HTMLAnchorAttributes } from 'svelte/elements';

  let {
    ref = $bindable(null),
    href,
    class: className,
    variant = 'default',
    children,
    ...restProps
  }: WithElementRef<HTMLAnchorAttributes> & {
    variant?: BadgeVariant;
  } = $props();
</script>

<svelte:element
  this={href ? 'a' : 'span'}
  bind:this={ref}
  data-slot="badge"
  {href}
  class={cn(badgeVariants({ variant }), className)}
  {...restProps}
>
  {@render children?.()}
</svelte:element>
