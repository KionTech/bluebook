import { describe, expect, it } from 'vitest';

import {
  characterPortrait,
  effectiveStandingChipClass,
  effectiveStandingTextClass,
  eveImage,
  standingChipClass,
  standingLabel,
  standingTextClass,
} from './eve.ts';

describe('eveImage', () => {
  it('links portraits and logos, and nothing for factions', () => {
    expect(eveImage('character', 1)).toBe(
      'https://images.evetech.net/characters/1/portrait?size=64',
    );
    expect(eveImage('corporation', 2, 128)).toBe(
      'https://images.evetech.net/corporations/2/logo?size=128',
    );
    expect(eveImage('alliance', 3)).toBe('https://images.evetech.net/alliances/3/logo?size=64');
    expect(eveImage('faction', 500001)).toBeNull();
    expect(characterPortrait(4)).toBe('https://images.evetech.net/characters/4/portrait?size=64');
  });
});

describe('standings', () => {
  it('labels a standing with its sign', () => {
    expect(standingLabel(10)).toBe('+10');
    expect(standingLabel(7.5)).toBe('+7.5');
    expect(standingLabel(0)).toBe('0');
    expect(standingLabel(-5)).toBe('-5');
  });

  it('colours a standing from blue to red', () => {
    expect(standingTextClass(10)).toBe('text-blue-600 dark:text-blue-400');
    expect(standingTextClass(1)).toBe('text-sky-600 dark:text-sky-400');
    expect(standingTextClass(0)).toBe('text-muted-foreground');
    expect(standingTextClass(-1)).toBe('text-orange-600 dark:text-orange-400');
    expect(standingTextClass(-5)).toBe('text-red-600 dark:text-red-400');
    expect(standingChipClass(5)).toBe('bg-blue-500/10 text-blue-600 dark:text-blue-400');
    expect(standingChipClass(-10)).toBe('bg-red-500/10 text-red-600 dark:text-red-400');
  });

  it('shows a source covered entity green', () => {
    expect(effectiveStandingTextClass({ value: 10, source: 'source' })).toBe(
      'text-green-600 dark:text-green-400',
    );
    expect(effectiveStandingTextClass({ value: -5, source: 'corporation' })).toBe(
      'text-red-600 dark:text-red-400',
    );
    expect(effectiveStandingChipClass({ value: 10, source: 'source' })).toBe(
      'bg-green-500/10 text-green-600 dark:text-green-400',
    );
  });
});
