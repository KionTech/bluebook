// Dates as the pages show them: formatted in the browser's locale, with a
// relative "ago" and the countdown to the next refresh.
const DIVISIONS: { amount: number; unit: Intl.RelativeTimeFormatUnit }[] = [
  { amount: 60, unit: 'seconds' },
  { amount: 60, unit: 'minutes' },
  { amount: 24, unit: 'hours' },
  { amount: 7, unit: 'days' },
  { amount: 4.34524, unit: 'weeks' },
  { amount: 12, unit: 'months' },
  { amount: Number.POSITIVE_INFINITY, unit: 'years' },
];

export function formatDateTime(value: string | null | undefined): string | null {
  if (!value) {
    return null;
  }

  return new Intl.DateTimeFormat(undefined, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(new Date(value));
}

export function timeAgo(
  value: string | null | undefined,
  nowMs: number = Date.now(),
): string | null {
  if (!value) {
    return null;
  }

  const formatter = new Intl.RelativeTimeFormat(undefined, {
    numeric: 'auto',
  });
  let duration = (new Date(value).getTime() - nowMs) / 1000;

  for (const division of DIVISIONS) {
    if (Math.abs(duration) < division.amount) {
      return formatter.format(Math.round(duration), division.unit);
    }

    duration /= division.amount;
  }

  return null;
}

/**
 * A formatted date string with a relative "how long ago" suffix, e.g.
 * "18 Jun 2026, 18:30 (2 hours ago)". Returns "Never" for empty values.
 */
export function dateWithAgo(value: string | null | undefined, nowMs: number = Date.now()): string {
  const formatted = formatDateTime(value);

  if (!formatted) {
    return 'Never';
  }

  const ago = timeAgo(value, nowMs);

  return ago ? `${formatted} (${ago})` : formatted;
}

/**
 * A "M:SS" countdown to a target timestamp, or "due now" once it has passed.
 */
export function countdown(targetMs: number, nowMs: number = Date.now()): string {
  const remaining = targetMs - nowMs;

  if (remaining <= 0) {
    return 'due now';
  }

  const totalSeconds = Math.ceil(remaining / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;

  return `${minutes}:${seconds.toString().padStart(2, '0')}`;
}
