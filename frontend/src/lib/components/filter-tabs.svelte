<script lang="ts" generics="T extends string">
  // A row of mutually exclusive filters, such as a request status or a
  // pilots view.
  let {
    label,
    items,
    value,
    onselect,
  }: {
    label: string;
    items: { value: T; label: string; count?: number }[];
    value: T;
    onselect: (value: T) => void;
  } = $props();
</script>

<div class="inline-flex flex-wrap border border-border bg-card" role="group" aria-label={label}>
  {#each items as item (item.value)}
    <button
      type="button"
      aria-pressed={value === item.value}
      class={[
        'label-caps inline-flex h-8 items-center gap-1.5 border-r border-border px-3 text-xs transition-colors last:border-r-0',
        value === item.value
          ? 'bg-primary/12 text-primary shadow-[inset_0_-2px_0_var(--primary)]'
          : 'text-muted-foreground hover:bg-primary/5 hover:text-foreground',
      ]}
      onclick={() => onselect(item.value)}
    >
      {item.label}
      {#if item.count !== undefined}
        <span class="font-mono text-[11px] tabular-nums opacity-80">{item.count}</span>
      {/if}
    </button>
  {/each}
</div>
