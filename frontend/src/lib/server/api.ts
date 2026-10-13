import { error, redirect } from '@sveltejs/kit';

import { errorMessage } from '#lib/errors.ts';

/** The status codes SvelteKit's `redirect` accepts. */
type RedirectStatus = 300 | 301 | 302 | 303 | 304 | 305 | 306 | 307 | 308;

/**
 * Loads a page's props from the Axum API for an SSR load function. The
 * api answers like the pages always did: a redirect (a guest on a page
 * behind the login goes to `/eve`) becomes the same redirect here, any
 * other non-OK status the error page for that status, with the api's
 * message where the page shows one.
 */
export async function apiGet<T>(fetch: typeof globalThis.fetch, path: string): Promise<T> {
  const response = await fetch(path, { redirect: 'manual' });

  if (response.status >= 300 && response.status < 400) {
    redirect(response.status as RedirectStatus, sitePath(response.headers.get('location')));
  }

  if (!response.ok) {
    const body: { message?: string } = await response.json().catch(() => ({}));
    error(response.status, errorMessage(response.status, body.message));
  }

  return response.json();
}

/** The api redirects within the site, or to EVE's login; a URL on the site
 * becomes its path. */
export function sitePath(location: string | null): string {
  if (!location) {
    return '/';
  }
  if (location.startsWith('/')) {
    return location;
  }
  return location;
}

/** The props endpoint of a page URL: `/api/pages` plus its path, the home
 * page at `/api/pages/home`. */
export function pagesPath(url: URL): string {
  const path = url.pathname === '/' ? '/home' : url.pathname;
  return `/api/pages${path}${url.search}`;
}

/** The load of every page: its props from the api. */
export function loadPage<T>({ fetch, url }: { fetch: typeof globalThis.fetch; url: URL }) {
  return apiGet<T>(fetch, pagesPath(url));
}
