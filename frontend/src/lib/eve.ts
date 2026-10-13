// EVE image server URLs and the colours and labels of standings.
/**
 * The EVE image-server URL for an entity's portrait/logo, or null for types
 * (e.g. factions) that have no image endpoint.
 */
export function eveImage(type: string, id: number, size = 64): string | null {
  switch (type) {
    case 'character':
      return `https://images.evetech.net/characters/${id}/portrait?size=${size}`;
    case 'corporation':
      return `https://images.evetech.net/corporations/${id}/logo?size=${size}`;
    case 'alliance':
      return `https://images.evetech.net/alliances/${id}/logo?size=${size}`;
    default:
      return null;
  }
}

/**
 * A character portrait URL (always available).
 */
export function characterPortrait(id: number, size = 64): string {
  return `https://images.evetech.net/characters/${id}/portrait?size=${size}`;
}

/**
 * A standing value formatted with an explicit sign (e.g. "+5", "-10", "0").
 */
export function standingLabel(standing: number): string {
  return standing > 0 ? `+${standing}` : `${standing}`;
}

/**
 * The Tailwind text-colour class for a standing, from blue (positive) through
 * muted (neutral) to red (negative).
 */
export function standingTextClass(standing: number): string {
  if (standing >= 5) {
    return 'text-blue-600 dark:text-blue-400';
  }
  if (standing > 0) {
    return 'text-sky-600 dark:text-sky-400';
  }
  if (standing === 0) {
    return 'text-muted-foreground';
  }
  if (standing > -5) {
    return 'text-orange-600 dark:text-orange-400';
  }
  return 'text-red-600 dark:text-red-400';
}

type EffectiveStanding = { value: number; source: string };

/**
 * Text colour for an effective standing; entities covered by the standings
 * source itself render green, like own corp/alliance in game.
 */
export function effectiveStandingTextClass(standing: EffectiveStanding): string {
  return standing.source === 'source'
    ? 'text-green-600 dark:text-green-400'
    : standingTextClass(standing.value);
}

/**
 * Chip classes for an effective standing; source-covered entities render
 * green, like own corp/alliance in game.
 */
export function effectiveStandingChipClass(standing: EffectiveStanding): string {
  return standing.source === 'source'
    ? 'bg-green-500/10 text-green-600 dark:text-green-400'
    : standingChipClass(standing.value);
}

/**
 * The Tailwind classes for a tinted standing chip (pill background plus text
 * colour), matching the standingTextClass buckets.
 */
export function standingChipClass(standing: number): string {
  if (standing >= 5) {
    return 'bg-blue-500/10 text-blue-600 dark:text-blue-400';
  }
  if (standing > 0) {
    return 'bg-sky-500/10 text-sky-600 dark:text-sky-400';
  }
  if (standing === 0) {
    return 'bg-muted text-muted-foreground';
  }
  if (standing > -5) {
    return 'bg-orange-500/10 text-orange-600 dark:text-orange-400';
  }
  return 'bg-red-500/10 text-red-600 dark:text-red-400';
}
