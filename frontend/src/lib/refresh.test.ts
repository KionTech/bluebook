import { describe, expect, it } from 'vitest';

import { nextRefresh } from './refresh.ts';

describe('nextRefresh', () => {
  it('counts down five minutes from the last sync', () => {
    const synced = '2026-06-25T13:58:00+00:00';
    expect(nextRefresh(synced, Date.parse('2026-06-25T13:59:00Z'))).toBe('4:00');
    expect(nextRefresh(synced, Date.parse('2026-06-25T14:03:00Z'))).toBe('due now');
    expect(nextRefresh(null, Date.now())).toBeNull();
  });
});
