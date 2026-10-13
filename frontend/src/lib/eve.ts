// EVE image server URLs and the colours and labels of standings.
/**
 * The EVE image-server URL for an entity's portrait/logo, or null for types
 * (e.g. factions) that have no image endpoint.
 */
export function eveImage(type: string, id: number, size = 64): string | null {
  switch (type) {
    case "character":
      return `https://images.evetech.net/characters/${id}/portrait?size=${size}`;
    case "corporation":
      return `https://images.evetech.net/corporations/${id}/logo?size=${size}`;
    case "alliance":
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
 * The text-colour class for a standing, in EVE's colours: excellent (dark
 * blue), good (light blue), neutral (grey), bad (orange), terrible (red).
 */
export function standingTextClass(standing: number): string {
  if (standing >= 5) {
    return "text-standing-excellent";
  }
  if (standing > 0) {
    return "text-standing-good";
  }
  if (standing === 0) {
    return "text-standing-neutral";
  }
  if (standing > -5) {
    return "text-standing-bad";
  }
  return "text-standing-terrible";
}

type EffectiveStanding = { value: number; source: string };

/**
 * Text colour for an effective standing; entities covered by the standings
 * source itself render green, like own corp/alliance in game.
 */
export function effectiveStandingTextClass(
  standing: EffectiveStanding,
): string {
  return standing.source === "source"
    ? "text-standing-own"
    : standingTextClass(standing.value);
}

/**
 * Chip classes for an effective standing; source-covered entities render
 * green, like own corp/alliance in game.
 */
export function effectiveStandingChipClass(
  standing: EffectiveStanding,
): string {
  return standing.source === "source"
    ? "bg-standing-own/12 text-standing-own"
    : standingChipClass(standing.value);
}

/**
 * The Tailwind classes for a tinted standing chip (pill background plus text
 * colour), matching the standingTextClass buckets.
 */
export function standingChipClass(standing: number): string {
  if (standing >= 5) {
    return "bg-standing-excellent/12 text-standing-excellent";
  }
  if (standing > 0) {
    return "bg-standing-good/12 text-standing-good";
  }
  if (standing === 0) {
    return "bg-standing-neutral/12 text-standing-neutral";
  }
  if (standing > -5) {
    return "bg-standing-bad/12 text-standing-bad";
  }
  return "bg-standing-terrible/12 text-standing-terrible";
}
