<script lang="ts">
  // A quiet message where a list has nothing to show, with an optional
  // next step below it.
  import type { Component, Snippet } from 'svelte';

  let {
    title,
    icon: Icon,
    children,
  }: { title: string; icon?: Component<{ class?: string }>; children?: Snippet } = $props();
</script>

<div class="frame flex flex-col items-center gap-2 px-6 py-10 text-center">
  {#if Icon}
    <Icon class="size-5 text-primary" />
  {:else}
    <span aria-hidden="true" class="pip text-border-strong"></span>
  {/if}
  <p class="label-caps text-sm">{title}</p>
  {#if children}
    <div class="max-w-md text-sm text-muted-foreground">{@render children()}</div>
  {/if}
</div>
