// The appearance setting (light, dark or system), as useAppearance.ts kept
// it: in localStorage for the browser and in a cookie for the server's
// first paint, applied as the `dark` class on the root element.
export type Appearance = 'light' | 'dark' | 'system';

export const COOKIE_DAYS = 365;

export function appearanceCookie(value: Appearance, days = COOKIE_DAYS): string {
  return `appearance=${value};path=/;max-age=${days * 24 * 60 * 60};SameSite=Lax`;
}

/** Whether the dark theme applies for a setting. */
export function isDark(value: Appearance, systemIsDark: boolean): boolean {
  return value === 'system' ? systemIsDark : value === 'dark';
}

export function storedAppearance(storage: Pick<Storage, 'getItem'>): Appearance {
  const value = storage.getItem('appearance');
  return value === 'light' || value === 'dark' ? value : 'system';
}

function prefersDark(): boolean {
  return window.matchMedia('(prefers-color-scheme: dark)').matches;
}

export function applyTheme(value: Appearance): void {
  document.documentElement.classList.toggle('dark', isDark(value, prefersDark()));
}

export function updateAppearance(value: Appearance): void {
  localStorage.setItem('appearance', value);
  document.cookie = appearanceCookie(value);
  applyTheme(value);
}

/** Follows the system's theme while the setting is "system". */
export function initializeTheme(): () => void {
  applyTheme(storedAppearance(localStorage));
  const query = window.matchMedia('(prefers-color-scheme: dark)');
  const listener = () => applyTheme(storedAppearance(localStorage));
  query.addEventListener('change', listener);
  return () => query.removeEventListener('change', listener);
}
