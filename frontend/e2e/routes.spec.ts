// The routes: the pages a guest is sent away from, the pages a pilot and
// the admin see, the error pages, `/settings`, the health page and the trailing slash.
import { expect, test } from '@playwright/test';

import { createAdmin, createCharacter, createUser, resetDatabase, signIn } from './helpers.ts';

test.beforeEach(() => resetDatabase());

const PAGES = [
  '/dashboard',
  '/settings/appearance',
  '/admin',
  '/admin/standing-requests',
  '/admin/pilots',
  '/admin/settings',
];

test('a guest is sent to the EVE login from every page behind it', async ({ request, baseURL }) => {
  for (const path of [...PAGES, '/settings']) {
    const response = await request.get(path, { maxRedirects: 0 });
    expect(response.status(), path).toBe(302);
    expect(new URL(response.headers().location, baseURL).pathname, path).toBe('/eve');
  }
});

test('the EVE login sends the browser to EVE SSO', async ({ request }) => {
  const response = await request.get('/eve', { maxRedirects: 0 });
  expect(response.status()).toBe(302);
  expect(response.headers().location).toMatch(
    /^https:\/\/login\.eveonline\.com\/v2\/oauth\/authorize\/\?client_id=.*&scope=&response_type=code&state=[A-Za-z0-9]{40}$/,
  );
});

test('a pilot sees the pilot pages and is refused the admin pages', async ({
  page,
  context,
  baseURL,
}) => {
  const user = createUser();
  createCharacter({ user });
  await signIn(context, user, baseURL!);

  for (const path of ['/', '/dashboard', '/settings/appearance']) {
    const response = await page.goto(path);
    expect(response?.status(), path).toBe(200);
  }
  for (const path of PAGES.filter((candidate) => candidate.startsWith('/admin'))) {
    const response = await page.goto(path);
    expect(response?.status(), path).toBe(403);
    await expect(page.getByText('This action is unauthorized.')).toBeVisible();
  }
  await page.goto('/settings');
  await expect(page).toHaveURL(/\/settings\/appearance$/);
});

test('the admin sees every page', async ({ page, context, baseURL }) => {
  await signIn(context, createAdmin(), baseURL!);
  for (const path of ['/', ...PAGES]) {
    const response = await page.goto(path);
    expect(response?.status(), path).toBe(200);
  }
});

test('an unknown page is the not found page', async ({ page }) => {
  const response = await page.goto('/nope');
  expect(response?.status()).toBe(404);
  await expect(page.getByText('Not Found')).toBeVisible();
  await expect(page).toHaveTitle('Not Found');
});

test('the health page answers', async ({ request }) => {
  const response = await request.get('/up');
  expect(response.status()).toBe(200);
  expect(await response.text()).toContain('Application up');
});

test('a trailing slash redirects to the page without it', async ({ request, baseURL }) => {
  const response = await request.get('/dashboard/', { maxRedirects: 0 });
  expect(response.status()).toBe(308);
  // SvelteKit's Location is relative to the URL with the slash.
  const target = new URL(response.headers().location, new URL('/dashboard/', baseURL));
  expect(target.pathname).toBe('/dashboard');
});
