// What the dashboard shows beside each character: the sync switch, an
// inherited mark, a first grant of the contact permissions, or a re-login
// prompt once a syncing character's token lapsed.
import type { SyncCharacter } from '#lib/types.ts';

export type SyncState = 'inherited' | 'reauth' | 'grant' | 'toggle';

export function syncState(character: SyncCharacter, reauthIds: ReadonlySet<number>): SyncState {
  if (character.inherits_source) {
    return 'inherited';
  }
  if (reauthIds.has(character.id) || (character.should_sync && !character.has_write_scope)) {
    return 'reauth';
  }
  if (!character.has_write_scope) {
    return 'grant';
  }
  return 'toggle';
}

/** One line on what a character's sync is doing. */
export function syncSummary(character: SyncCharacter, state: SyncState): string {
  switch (state) {
    case 'inherited':
      return 'Inherits the source';
    case 'reauth':
      return 'Not syncing';
    case 'grant':
      return 'No contact permissions';
    default:
      return character.should_sync
        ? `${character.synced_contacts_count} contacts synced`
        : 'Sync off';
  }
}

/** The characters of the account other than the active one. */
export function otherCharacters<T extends { id: number }>(
  characters: T[] | undefined,
  activeId: number | undefined,
): T[] {
  return (characters ?? []).filter((character) => character.id !== activeId);
}
