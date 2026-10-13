// A confirmation dialog in place of `window.confirm` (Laravel's
// useConfirm): `confirm()` opens the one ConfirmDialog of the layout and
// resolves to the visitor's choice.

export type ConfirmOptions = {
  title: string;
  description?: string;
  /** Defaults to "Bestätigen". */
  confirmLabel?: string;
  /** Defaults to "Abbrechen". */
  cancelLabel?: string;
  destructive?: boolean;
};

class ConfirmState {
  open = $state(false);
  title = $state('');
  description: string | undefined = $state();
  confirmLabel = $state('Bestätigen');
  cancelLabel = $state('Abbrechen');
  destructive = $state(false);
  resolve: ((value: boolean) => void) | null = null;
}

export const confirmState = new ConfirmState();

/** Closes the dialog with the answer; once settled, further calls only
 * keep it closed. */
export function settleConfirm(value: boolean): void {
  confirmState.open = false;
  const { resolve } = confirmState;
  confirmState.resolve = null;
  resolve?.(value);
}

export function confirm(options: ConfirmOptions): Promise<boolean> {
  settleConfirm(false);
  confirmState.title = options.title;
  confirmState.description = options.description;
  confirmState.confirmLabel = options.confirmLabel ?? 'Bestätigen';
  confirmState.cancelLabel = options.cancelLabel ?? 'Abbrechen';
  confirmState.destructive = options.destructive ?? false;
  confirmState.open = true;
  return new Promise((resolve) => {
    confirmState.resolve = resolve;
  });
}
