import { afterEach, describe, expect, it, vi } from 'vitest';

import { DURATION_MS, Toaster, flashMessages } from './toaster.svelte.ts';

describe('flashMessages', () => {
  it('lists the messages of a flash in order', () => {
    expect(flashMessages({ success: 'Saved.', error: null, info: 'Note.' })).toEqual([
      ['success', 'Saved.'],
      ['info', 'Note.'],
    ]);
    expect(flashMessages({ success: null, error: null, info: null })).toEqual([]);
    expect(flashMessages(undefined)).toEqual([]);
  });
});

describe('Toaster', () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it('shows a flash for four seconds', () => {
    vi.useFakeTimers();
    const toaster = new Toaster();
    toaster.showFlash({
      success: 'Standing request updated.',
      error: null,
      info: null,
    });
    expect(toaster.items).toEqual([
      { id: 1, type: 'success', message: 'Standing request updated.' },
    ]);
    vi.advanceTimersByTime(DURATION_MS - 1);
    expect(toaster.items).toHaveLength(1);
    vi.advanceTimersByTime(1);
    expect(toaster.items).toEqual([]);
  });

  it('closes a toast on demand', () => {
    const toaster = new Toaster();
    toaster.add('error', 'This action is unauthorized.');
    toaster.add('info', 'No characters are set to sync.');
    toaster.dismiss(1);
    expect(toaster.items).toEqual([
      { id: 2, type: 'info', message: 'No characters are set to sync.' },
    ]);
  });
});
