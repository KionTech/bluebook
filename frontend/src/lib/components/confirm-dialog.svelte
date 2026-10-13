<script lang="ts">
  // The one dialog behind `confirm()`; closing it any other way counts
  // as "no".
  import * as AlertDialog from '#lib/components/ui/alert-dialog/index.ts';
  import { buttonVariants } from '#lib/components/ui/button/index.ts';
  import { confirmState, settleConfirm } from '#lib/confirm.svelte.ts';
</script>

<AlertDialog.Root
  open={confirmState.open}
  onOpenChange={(open) => {
    if (!open) settleConfirm(false);
  }}
>
  <AlertDialog.Content class="pointer-events-auto">
    <AlertDialog.Header>
      <AlertDialog.Title>{confirmState.title}</AlertDialog.Title>
      {#if confirmState.description}
        <AlertDialog.Description>{confirmState.description}</AlertDialog.Description>
      {/if}
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel data-testid="confirm-cancel" onclick={() => settleConfirm(false)}>
        {confirmState.cancelLabel}
      </AlertDialog.Cancel>
      <AlertDialog.Action
        data-testid="confirm-accept"
        class={confirmState.destructive ? buttonVariants({ variant: 'destructive' }) : undefined}
        onclick={() => settleConfirm(true)}
      >
        {confirmState.confirmLabel}
      </AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
