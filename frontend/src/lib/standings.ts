// The dashboard's standings list and request options: the order within a
// standing, the search, and what an option's state is.
import type { RequestOption, Standing } from '#lib/types.ts';

/** Within a standing level, entity types come in this order. */
const TYPE_ORDER = ['corporation', 'alliance', 'character', 'faction'];

export function typeRank(type: string): number {
  const index = TYPE_ORDER.indexOf(type);
  return index === -1 ? TYPE_ORDER.length : index;
}

function label(standing: Standing): string {
  return standing.name ?? `${standing.contact_id}`;
}

/** Highest standing first, then by type, then alphabetically. */
export function sortStandings(standings: Standing[]): Standing[] {
  return [...standings].sort(
    (a, b) =>
      b.standing - a.standing ||
      typeRank(a.contact_type) - typeRank(b.contact_type) ||
      label(a).localeCompare(label(b), undefined, { sensitivity: 'base' }),
  );
}

/** The standings whose name contains the search, ignoring case. */
export function filterStandings(standings: Standing[], search: string): Standing[] {
  const query = search.trim().toLowerCase();
  if (query === '') {
    return standings;
  }
  return standings.filter((standing) => label(standing).toLowerCase().includes(query));
}

/** Whether the entity is already in the standings shown on the page. */
export function hasStanding(option: RequestOption, standings: Standing[] | null): boolean {
  return (standings ?? []).some(
    (standing) => standing.contact_type === option.type && standing.contact_id === option.id,
  );
}

export function canRequest(option: RequestOption, standings: Standing[] | null): boolean {
  return !hasStanding(option, standings) && option.status !== 'pending' && option.status !== 'done';
}

export function optionStatusLabel(
  option: RequestOption,
  standings: Standing[] | null,
): string | null {
  if (hasStanding(option, standings)) {
    return 'On the list';
  }
  if (option.status === 'pending') {
    return 'Requested';
  }
  if (option.status === 'done') {
    return 'Approved';
  }
  if (option.status === 'rejected') {
    return 'Rejected';
  }
  return null;
}
