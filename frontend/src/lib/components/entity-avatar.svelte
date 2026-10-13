<script lang="ts">
  // The portrait or logo of an EVE entity in a square frame with corner
  // brackets, with its initials while the image server is slow or for
  // types without an image.
  import * as Avatar from '#lib/components/ui/avatar/index.ts';
  import { eveImage } from '#lib/eve.ts';
  import { getInitials } from '#lib/initials.ts';
  import { cn } from '#lib/utils.ts';

  let {
    type,
    id,
    name,
    class: className = 'size-8',
  }: { type: string; id: number; name: string | null; class?: string } = $props();

  const src = $derived(eveImage(type, id));
</script>

<span class={cn('bracketed', className)}>
  <Avatar.Root class="size-full shrink-0 overflow-hidden border border-border bg-muted">
    {#if src}
      <Avatar.Image {src} alt="" />
    {/if}
    <Avatar.Fallback class="font-mono text-[10px] text-muted-foreground uppercase">
      {name ? getInitials(name) : type.slice(0, 2)}
    </Avatar.Fallback>
  </Avatar.Root>
</span>
