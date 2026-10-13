// The source refreshes every five minutes after its last sync.
import { countdown } from "#lib/date.ts";

export const REFRESH_INTERVAL_MS = 5 * 60 * 1000;

/** "M:SS" until the next refresh, or null without a last sync. */
export function nextRefresh(
  lastSyncedAt: string | null | undefined,
  nowMs: number,
): string | null {
  if (!lastSyncedAt) {
    return null;
  }
  return countdown(
    new Date(lastSyncedAt).getTime() + REFRESH_INTERVAL_MS,
    nowMs,
  );
}
