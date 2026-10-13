import { describe, expect, it } from 'vitest';

import type { PilotCharacter } from '#lib/types.ts';

import { pilotsUrl, viewOf, visibleCharacters } from './pilots.ts';

function pilot(id: number, name: string, extra: Partial<PilotCharacter> = {}): PilotCharacter {
  return {
    id,
    name,
    is_main: false,
    account: { id: 1, name: 'Account' },
    standing: null,
    corporation: null,
    alliance: null,
    ...extra,
  };
}

const rows = [
  pilot(1, 'Bravo', { standing: { value: -5, source: 'corporation' } }),
  pilot(2, 'alpha', {
    corporation: { id: 5, name: 'Jita Corp', ticker: null },
  }),
  pilot(3, 'Charlie', { standing: { value: 10, source: 'source' } }),
];

describe('visibleCharacters', () => {
  it('sorts by name either way', () => {
    expect(visibleCharacters(rows, '', 'name', true).map((row) => row.id)).toEqual([2, 1, 3]);
    expect(visibleCharacters(rows, '', 'name', false).map((row) => row.id)).toEqual([3, 1, 2]);
  });

  it('sorts by standing with the ones without last', () => {
    expect(visibleCharacters(rows, '', 'standing', false).map((row) => row.id)).toEqual([3, 1, 2]);
    expect(visibleCharacters(rows, '', 'standing', true).map((row) => row.id)).toEqual([1, 3, 2]);
  });

  it('searches the name, account, corporation and alliance', () => {
    expect(visibleCharacters(rows, 'jita', 'name', true).map((row) => row.id)).toEqual([2]);
    expect(visibleCharacters(rows, 'ACCOUNT', 'name', true)).toHaveLength(3);
    expect(visibleCharacters(rows, 'nobody', 'name', true)).toEqual([]);
  });
});

describe('the pilots page URL', () => {
  it('keeps the view and the search', () => {
    expect(pilotsUrl('characters', '')).toBe('/admin/pilots');
    expect(pilotsUrl('corporations', 'kion')).toBe('/admin/pilots?view=corporations&search=kion');
    expect(pilotsUrl('characters', 'a b')).toBe('/admin/pilots?search=a+b');
    expect(viewOf('alliances')).toBe('alliances');
    expect(viewOf('maybe')).toBe('characters');
  });
});
