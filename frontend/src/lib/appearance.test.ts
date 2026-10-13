import { describe, expect, it } from 'vitest';

import { appearanceCookie, isDark, storedAppearance } from './appearance.ts';

describe('appearance', () => {
  it('writes the cookie the server reads, for a year', () => {
    expect(appearanceCookie('dark')).toBe('appearance=dark;path=/;max-age=31536000;SameSite=Lax');
  });

  it('applies dark for dark, or for system when the system prefers it', () => {
    expect(isDark('dark', false)).toBe(true);
    expect(isDark('light', true)).toBe(false);
    expect(isDark('system', true)).toBe(true);
    expect(isDark('system', false)).toBe(false);
  });

  it('reads the stored setting, system by default', () => {
    const storage = (value: string | null) => ({ getItem: () => value });
    expect(storedAppearance(storage('light'))).toBe('light');
    expect(storedAppearance(storage('dark'))).toBe('dark');
    expect(storedAppearance(storage(null))).toBe('system');
    expect(storedAppearance(storage('nonsense'))).toBe('system');
  });
});
