// The `appearance` cookie (light, dark or system), set in the browser by
// the appearance settings and read here so the first paint already has
// the right theme. Without a choice the theme is dark.

export type Appearance = 'light' | 'dark' | 'system';

export function appearanceOf(cookie: string | undefined): Appearance {
  return cookie === 'light' || cookie === 'system' ? cookie : 'dark';
}

/** Fills the `%appearance.*%` placeholders of app.html. */
export function applyAppearance(html: string, appearance: Appearance): string {
  return html
    .replace('%appearance.class%', appearance === 'dark' ? 'dark' : '')
    .replace('%appearance.value%', appearance);
}
