<script lang="ts">
  import PanelLeftClose from '@lucide/svelte/icons/panel-left-close';
  import PanelLeftOpen from '@lucide/svelte/icons/panel-left-open';
  import { Button } from '#lib/components/ui/button/index.ts';
  import { cn } from '#lib/utils.ts';
  import { useSidebar } from './context.svelte.ts';
  import type { ComponentProps } from 'svelte';

  let {
    ref = $bindable(null),
    class: className,
    onclick,
    ...restProps
  }: ComponentProps<typeof Button> & {
    onclick?: (e: MouseEvent) => void;
  } = $props();

  const sidebar = useSidebar();
</script>

<Button
  bind:ref
  data-sidebar="trigger"
  data-slot="sidebar-trigger"
  variant="ghost"
  size="icon"
  class={cn('h-7 w-7', className)}
  type="button"
  onclick={(e) => {
    onclick?.(e);
    sidebar.toggle();
  }}
  {...restProps}
>
  {#if sidebar.isMobile || sidebar.state === 'collapsed'}
    <PanelLeftOpen />
  {:else}
    <PanelLeftClose />
  {/if}
  <span class="sr-only">Toggle sidebar</span>
</Button>
