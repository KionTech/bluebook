import { describe, expect, it } from 'vitest';

import { countdown, dateWithAgo, formatDateTime, timeAgo } from './date.ts';

const NOW = Date.parse('2026-06-25T14:00:00Z');

describe('countdown', () => {
  it('counts minutes and seconds down to the target', () => {
    expect(countdown(NOW + 5 * 60_000, NOW)).toBe('5:00');
    expect(countdown(NOW + 61_500, NOW)).toBe('1:02');
    expect(countdown(NOW + 9_000, NOW)).toBe('0:09');
    expect(countdown(NOW, NOW)).toBe('due now');
    expect(countdown(NOW - 1, NOW)).toBe('due now');
  });
});

describe('timeAgo', () => {
  it('says how long ago a moment was', () => {
    expect(timeAgo('2026-06-25T13:58:00+00:00', NOW)).toMatch(/2 minutes ago/);
    expect(timeAgo('2026-06-25T13:59:30+00:00', NOW)).toMatch(/30 seconds ago/);
    expect(timeAgo('2026-06-23T14:00:00+00:00', NOW)).toMatch(/2 days ago/);
    expect(timeAgo(null, NOW)).toBeNull();
  });
});

describe('dateWithAgo', () => {
  it('formats the date with its relative time, or Never', () => {
    const text = dateWithAgo('2026-06-25T13:58:00+00:00', NOW);
    expect(text).toContain(formatDateTime('2026-06-25T13:58:00+00:00')!);
    expect(text).toMatch(/\(2 minutes ago\)$/);
    expect(dateWithAgo(null, NOW)).toBe('Never');
  });
});
