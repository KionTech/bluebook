import { describe, expect, it } from 'vitest';

import { confirm, confirmState, settleConfirm } from './confirm.svelte.ts';

describe('confirm', () => {
  it('opens the dialog with the defaults and resolves to the answer', async () => {
    const answer = confirm({ title: 'Artikel löschen', destructive: true });

    expect(confirmState.open).toBe(true);
    expect(confirmState.title).toBe('Artikel löschen');
    expect(confirmState.confirmLabel).toBe('Bestätigen');
    expect(confirmState.cancelLabel).toBe('Abbrechen');
    expect(confirmState.destructive).toBe(true);

    settleConfirm(true);
    expect(await answer).toBe(true);
    expect(confirmState.open).toBe(false);
  });

  it('answers no to a dialog replaced by another', async () => {
    const first = confirm({ title: 'Eins' });
    const second = confirm({ title: 'Zwei', confirmLabel: 'Löschen' });

    expect(await first).toBe(false);
    expect(confirmState.confirmLabel).toBe('Löschen');
    settleConfirm(false);
    expect(await second).toBe(false);
  });
});
