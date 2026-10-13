import { describe, expect, it } from 'vitest';

import { pageTitle } from './title.ts';

describe('pageTitle', () => {
  it('suffixes the app name', () => {
    expect(pageTitle('Standings', 'Bluebook')).toBe('Standings - Bluebook');
    expect(pageTitle('Bluebook', 'Bluebook')).toBe('Bluebook - Bluebook');
    expect(pageTitle(undefined, 'Bluebook')).toBe('Bluebook');
  });
});
