import { describe, expect, it } from 'vitest';

import type { RequestOption, Standing } from '#lib/types.ts';

import {
  canRequest,
  filterStandings,
  hasStanding,
  optionStatusLabel,
  requestedOptions,
  sortStandings,
  typeRank,
} from './standings.ts';

function standing(contact_id: number, contact_type: string, value: number, name: string | null) {
  return { contact_id, contact_type, standing: value, name, redundant_via: null } as Standing;
}

function option(type: string, id: number, status: string | null = null): RequestOption {
  return { type, id, name: null, via_character_id: 1, status };
}

describe('sortStandings', () => {
  it('orders by standing, type and name', () => {
    const sorted = sortStandings([
      standing(1, 'character', 10, 'Zed'),
      standing(2, 'corporation', 10, 'zebra Corp'),
      standing(3, 'corporation', 10, 'Alpha Corp'),
      standing(4, 'alliance', 10, 'Blue Alliance'),
      standing(5, 'faction', -5, 'Caldari State'),
      standing(6, 'corporation', 5, null),
      standing(7, 'corporation', 10, 'Écho Corp'),
    ]);
    expect(sorted.map((item) => item.contact_id)).toEqual([3, 7, 2, 4, 1, 6, 5]);
    expect(typeRank('unknown')).toBe(4);
  });
});

describe('filterStandings', () => {
  it('matches the name, or the id without one, ignoring case', () => {
    const all = [
      standing(1, 'corporation', 10, 'Blue Corp'),
      standing(98000099, 'corporation', 5, null),
    ];
    expect(filterStandings(all, ' blue ')).toEqual([all[0]]);
    expect(filterStandings(all, '980000')).toEqual([all[1]]);
    expect(filterStandings(all, '')).toEqual(all);
    expect(filterStandings(all, 'Nobody Home')).toEqual([]);
  });
});

describe('request options', () => {
  const standings = [standing(2000, 'corporation', 5, 'Blue Corp')];

  it('is on the list when the standing exists', () => {
    expect(hasStanding(option('corporation', 2000), standings)).toBe(true);
    expect(hasStanding(option('alliance', 2000), standings)).toBe(false);
    expect(optionStatusLabel(option('corporation', 2000), standings)).toBe('On the list');
    expect(canRequest(option('corporation', 2000), standings)).toBe(false);
  });

  it('labels the request status and allows asking again after a rejection', () => {
    expect(optionStatusLabel(option('character', 1, 'pending'), null)).toBe('Requested');
    expect(optionStatusLabel(option('character', 1, 'done'), null)).toBe('Approved');
    expect(optionStatusLabel(option('character', 1, 'rejected'), null)).toBe('Rejected');
    expect(optionStatusLabel(option('character', 1), null)).toBeNull();
    expect(canRequest(option('character', 1, 'pending'), null)).toBe(false);
    expect(canRequest(option('character', 1, 'done'), null)).toBe(false);
    expect(canRequest(option('character', 1, 'rejected'), null)).toBe(true);
    expect(canRequest(option('character', 1), null)).toBe(true);
  });
});

describe('requestedOptions', () => {
  it('keeps the requested options, pending first', () => {
    const options = [
      option('character', 1),
      option('corporation', 2, 'rejected'),
      option('alliance', 3, 'pending'),
      option('character', 4, 'done'),
    ];
    expect(requestedOptions(options).map((item) => item.id)).toEqual([3, 4, 2]);
  });
});
