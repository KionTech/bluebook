// The heads of the pages: the title (`<page> - <app name>`),
// the icons, the appearance class and robots.txt.
import { expect, test } from '@playwright/test';

import { createAdmin, resetDatabase, signIn } from './helpers.ts';

test.beforeEach(() => resetDatabase());

test('every page has its title', async ({ page, context, baseURL }) => {
  await page.goto('/');
  await expect(page).toHaveTitle('Bluebook - Bluebook');

  await signIn(context, createAdmin(), baseURL!);
  for (const [path, title] of [
    ['/dashboard', 'Standings - Bluebook'],
    ['/settings/appearance', 'Appearance settings - Bluebook'],
    ['/admin', 'Administration - Bluebook'],
    ['/admin/standing-requests', 'Standing requests - Bluebook'],
    ['/admin/pilots', 'Pilots - Bluebook'],
    ['/admin/settings', 'Admin settings - Bluebook'],
  ]) {
    await page.goto(path);
    await expect(page, path).toHaveTitle(title);
  }
});

test('the icons and fonts are linked', async ({ page }) => {
  await page.goto('/');
  await expect(page.locator('link[rel="icon"][href$="/favicon.ico"]')).toHaveCount(1);
  await expect(page.locator('link[rel="icon"][href$="/favicon.svg"]')).toHaveCount(1);
  await expect(
    page.locator('link[rel="apple-touch-icon"][href$="/apple-touch-icon.png"]'),
  ).toHaveCount(1);
  for (const file of ['/favicon.ico', '/favicon.svg', '/apple-touch-icon.png']) {
    const response = await page.request.get(file);
    expect(response.status(), file).toBe(200);
  }
});

test('the appearance cookie decides the theme of the first paint', async ({
  page,
  context,
  baseURL,
}) => {
  const url = new URL(baseURL!);
  await context.addCookies([
    { name: 'appearance', value: 'dark', domain: url.hostname, path: '/' },
  ]);
  const response = await page.request.get('/');
  expect(await response.text()).toContain('<html lang="en" class="dark">');
  // The browser keeps the choice in localStorage too, as the appearance
  // settings store it; without it the theme follows the system.
  await page.goto('/');
  await page.evaluate(() => localStorage.setItem('appearance', 'dark'));
  await page.goto('/');
  await expect(page.locator('html')).toHaveClass(/dark/);
});

test('robots may crawl everything', async ({ request }) => {
  const response = await request.get('/robots.txt');
  expect(await response.text()).toBe('User-agent: *\nDisallow:\n');
});
