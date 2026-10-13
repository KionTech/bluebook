// The setup wizard's step, kept in localStorage so it resumes after the
// EVE login that adds a character.
export const TOTAL_STEPS = 4;
export const STORAGE_KEY = 'standings.setup-wizard-step';

export function storedStep(storage: Pick<Storage, 'getItem'>): number {
  const step = Number(storage.getItem(STORAGE_KEY) ?? 0);
  return Number.isInteger(step) && step >= 0 && step < TOTAL_STEPS ? step : 0;
}
