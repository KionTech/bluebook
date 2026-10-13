// The redesigned interface: the top bar, the main flows, the prompts and
// empty states, phone width and accessibility in both themes.
import AxeBuilder from '@axe-core/playwright';
import { type BrowserContext, type Page, expect, test } from '@playwright/test';

import {
  createAdmin,
  createAlliance,
  createCharacter,
  createCorporation,
  createSource,
  createSourceContact,
  createStandingRequest,
  createUser,
  grantScopes,
  psql,
  resetDatabase,
  setMain,
  signIn,
  watchForErrors,
} from './helpers.ts';

const WRITE_SCOPES = ['esi-characters.read_contacts.v1', 'esi-characters.write_contacts.v1'];

test.beforeEach(() => resetDatabase());

/** An admin with a pilot, a source with standings and a pending request:
 * every page has something to show. */
function seedEverything(): number {
  const admin = createAdmin();
  createAlliance(3000, 'Home Alliance', 'HOME');
  createCorporation(2000, 'Home Corp', 'HOMC', 3000);
  createCorporation(4000, 'Blue Corp', 'BLUE');
  createSource('alliance', 3000, 1);
  createSourceContact('corporation', 4000, 10);
  const pilot = createUser({ name: 'Pilot Account' });
  const main = createCharacter({ user: pilot, name: 'Pilot Main', corporationId: 2000 });
  setMain(pilot, main);
  createStandingRequest(createCharacter({ name: 'Asking Pilot' }));
  return admin;
}

async function useTheme(context: BrowserContext, baseURL: string, theme: 'light' | 'dark') {
  const url = new URL(baseURL);
  await context.addCookies([{ name: 'appearance', value: theme, domain: url.hostname, path: '/' }]);
  await context.addInitScript((value) => localStorage.setItem('appearance', value), theme);
}

async function hasHorizontalScroll(page: Page): Promise<boolean> {
  return page.evaluate(
    () => document.documentElement.scrollWidth > document.documentElement.clientWidth,
  );
}

const PAGES = [
  '/',
  '/dashboard',
  '/settings/appearance',
  '/admin',
  '/admin/standing-requests',
  '/admin/pilots',
  '/admin/pilots?view=corporations',
  '/admin/settings',
];

test('switches character from the top bar', async ({ page, context, baseURL }) => {
  const user = createUser();
  const main = createCharacter({ user, name: 'First Pilot' });
  createCharacter({ user, name: 'Second Pilot' });
  setMain(user, main);
  await signIn(context, user, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/dashboard');
  await page.getByRole('button', { name: 'Character menu, First Pilot' }).click();
  await page.getByRole('menuitem', { name: 'Second Pilot' }).click();

  await expect(page.getByText('Switched to Second Pilot.')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Character menu, Second Pilot' })).toBeVisible();
  noErrors();
});

test('offers add character, appearance and log out in the character menu', async ({
  page,
  context,
  baseURL,
}) => {
  const user = createUser();
  createCharacter({ user, name: 'Menu Pilot' });
  await signIn(context, user, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/dashboard');
  await page.getByRole('button', { name: 'Character menu, Menu Pilot' }).click();

  await expect(page.getByRole('menuitem', { name: 'Add character' })).toHaveAttribute(
    'href',
    '/eve?add_to_account=1',
  );
  await page.getByRole('menuitem', { name: 'Appearance' }).click();
  await expect(page).toHaveURL(/\/settings\/appearance$/);

  await page.getByRole('button', { name: 'Character menu, Menu Pilot' }).click();
  await page.getByRole('menuitem', { name: 'Log out' }).click();
  await expect(page).toHaveURL(/\/$/);
  await expect(page.getByRole('link', { name: 'Sign in with EVE Online' })).toBeVisible();
  noErrors();
});

test('toggles sync for a character', async ({ page, context, baseURL }) => {
  const user = createUser();
  const character = createCharacter({ user, name: 'Switch Pilot' });
  grantScopes(character, WRITE_SCOPES);
  await signIn(context, user, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/dashboard');
  const toggle = page.getByRole('switch', { name: 'Sync standings to Switch Pilot' });
  await expect(toggle).toHaveAttribute('aria-checked', 'false');
  await toggle.click();

  await expect(page.getByText('Switch Pilot will now sync standings.')).toBeVisible();
  await expect(toggle).toHaveAttribute('aria-checked', 'true');
  expect(psql(`select should_sync from characters where id = ${character}`)).toBe('t');
  noErrors();
});

test('requests a standing from the dialog', async ({ page, context, baseURL }) => {
  const user = createUser();
  const character = createCharacter({ user, name: 'Outsider' });
  createSource('character', 999);
  await signIn(context, user, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/dashboard');
  await page.getByRole('button', { name: 'Request standing' }).click();
  await page
    .getByRole('dialog')
    .getByRole('button', { name: /Outsider/ })
    .click();

  await expect(page.getByText('Standing request submitted.')).toBeVisible();
  await expect(page.getByText('Waiting for an admin')).toBeVisible();
  expect(
    psql(`select status from standing_requests where requested_by_character_id = ${character}`),
  ).toBe('pending');
  noErrors();
});

test('rejects a standing request after confirming', async ({ page, context, baseURL }) => {
  const admin = createAdmin();
  const request = createStandingRequest(createCharacter({ name: 'Unwanted Pilot' }));
  await signIn(context, admin, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/admin/standing-requests');
  await page.getByRole('button', { name: 'Reject', exact: true }).click();
  await expect(page.getByText('Reject request?')).toBeVisible();
  await page.getByRole('dialog').getByRole('button', { name: 'Confirm' }).click();

  await expect(page.getByText('Standing request updated.')).toBeVisible();
  await expect(page.getByText('No pending requests')).toBeVisible();
  expect(psql(`select status from standing_requests where id = ${request}`)).toBe('rejected');

  await page.getByRole('button', { name: /Rejected/ }).click();
  await expect(page).toHaveURL(/status=rejected/);
  await expect(page.getByText('Unwanted Pilot').first()).toBeVisible();
  noErrors();
});

test('switches the pilot views and searches them', async ({ page, context, baseURL }) => {
  const admin = createAdmin();
  createCorporation(2000, 'Search Corp', 'SRCH');
  const pilot = createUser({ name: 'Searcher' });
  createCharacter({ user: pilot, name: 'Findable Pilot', corporationId: 2000 });
  createCharacter({ user: pilot, name: 'Other Pilot' });
  await signIn(context, admin, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/admin/pilots');
  const search = page.getByRole('searchbox', { name: 'Search pilots and characters' });
  await search.fill('findable');
  await expect(page.getByText('Findable Pilot')).toBeVisible();
  await expect(page.getByText('Other Pilot')).toHaveCount(0);
  await search.fill('nobody at all');
  await expect(page.getByText('No characters match your search')).toBeVisible();

  await search.fill('');
  await page.getByRole('button', { name: 'Corporations' }).click();
  await expect(page).toHaveURL(/view=corporations/);
  await expect(page.getByRole('button', { name: /Search Corp/ })).toBeVisible();

  await search.fill('zzz');
  await expect(page).toHaveURL(/search=zzz/);
  await expect(page.getByText('No corporations match your search')).toBeVisible();

  await page.getByRole('button', { name: 'Alliances' }).click();
  await expect(page).toHaveURL(/view=alliances/);
  noErrors();
});

test('is dark until the user picks another theme', async ({ page, request }) => {
  const response = await request.get('/');
  expect(await response.text()).toContain('<html lang="en" class="dark">');
  await page.goto('/');
  await expect(page.locator('html')).toHaveClass(/dark/);
});

test('switches the appearance to dark', async ({ page, context, baseURL }) => {
  const user = createUser();
  createCharacter({ user });
  await signIn(context, user, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/settings/appearance');
  await page.getByRole('button', { name: 'Light' }).click();
  await expect(page.locator('html')).not.toHaveClass(/dark/);
  await page.getByRole('button', { name: 'Dark' }).click();

  await expect(page.locator('html')).toHaveClass(/dark/);
  await expect(page.getByRole('button', { name: 'Dark' })).toHaveAttribute('aria-pressed', 'true');
  expect(await page.evaluate(() => localStorage.getItem('appearance'))).toBe('dark');
  expect((await context.cookies()).find((cookie) => cookie.name === 'appearance')?.value).toBe(
    'dark',
  );

  await page.reload();
  await expect(page.locator('html')).toHaveClass(/dark/);
  noErrors();
});

test('asks a character with a lapsed token to log in again', async ({ page, context, baseURL }) => {
  const user = createUser();
  const lapsed = createCharacter({ user, name: 'Lapsed Pilot', shouldSync: true });
  grantScopes(lapsed, ['esi-characters.read_contacts.v1']);
  const fine = createCharacter({ user, name: 'Fine Pilot', shouldSync: true });
  grantScopes(fine, WRITE_SCOPES);
  await signIn(context, user, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/dashboard');
  const lapsedRow = page.locator(`[data-test="character-${lapsed}"]`);
  const prompt = lapsedRow.getByRole('alert');
  await expect(prompt).toContainText('Lapsed Pilot needs to log in again.');
  await expect(prompt.getByRole('link', { name: 'Log in again' })).toHaveAttribute(
    'href',
    /\/eve\?add_to_account=1&scopes=.*write_contacts/,
  );
  await expect(lapsedRow.getByRole('switch')).toHaveCount(0);
  await expect(page.locator(`[data-test="character-${fine}"]`).getByRole('alert')).toHaveCount(0);
  await expect(page.locator(`[data-test="character-${fine}"]`).getByRole('switch')).toBeVisible();

  // Away from the dashboard one notice names every such character.
  await page.goto('/settings/appearance');
  await expect(page.getByRole('alert')).toContainText('Lapsed Pilot needs to log in again');
  noErrors();
});

test('asks an admin to re-authenticate an unreadable source', async ({
  page,
  context,
  baseURL,
}) => {
  const admin = createAdmin();
  createSource('alliance', 3000);
  await signIn(context, admin, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/admin');

  const prompt = page.getByRole('alert').filter({ hasText: 'cannot be read' });
  await expect(prompt).toBeVisible();
  await expect(prompt.getByRole('link', { name: 'Re-authenticate source' })).toHaveAttribute(
    'href',
    /\/eve\?add_to_account=1&scopes=/,
  );
  noErrors();
});

test('shows empty states', async ({ page, context, baseURL }) => {
  const admin = createAdmin();
  const user = createUser();
  createAlliance(3000, 'Lonely Alliance');
  createCharacter({ user, name: 'Lonely Pilot', allianceId: 3000 });
  await signIn(context, admin, baseURL!);
  const noErrors = watchForErrors(page);

  // No source yet.
  await page.goto('/admin');
  await expect(page.getByText('No source is set')).toBeVisible();
  await expect(page.getByText('Nothing waiting')).toBeVisible();

  await page.goto('/admin/standing-requests');
  await expect(page.getByText('No pending requests')).toBeVisible();
  await page.goto('/admin/standing-requests?status=done');
  await expect(page.getByText('No done requests')).toBeVisible();

  await page.goto('/admin/pilots?view=corporations');
  await expect(page.getByText('No corporations yet')).toBeVisible();

  await signIn(context, user, baseURL!);
  await page.goto('/dashboard');
  await expect(page.getByText('No standings source yet')).toBeVisible();
  await expect(page.getByText('No standings yet')).toBeVisible();

  // The pilot's alliance is the source, which has no standings so far.
  createSource('alliance', 3000);
  await page.goto('/dashboard');
  await expect(page.getByText('The source has no standings yet')).toBeVisible();

  // Someone else's source: the standings stay hidden until one is granted.
  psql('delete from standings_sources');
  createSource('character', 999);
  await page.goto('/dashboard');
  await expect(page.getByText('Standings are hidden')).toBeVisible();
  noErrors();
});

test('has no horizontal scroll at phone width', async ({ page, context, baseURL }) => {
  const admin = seedEverything();
  await signIn(context, admin, baseURL!);
  await page.setViewportSize({ width: 390, height: 844 });
  const noErrors = watchForErrors(page);

  for (const path of PAGES) {
    await page.goto(path);
    await page.waitForLoadState('networkidle');
    expect(await hasHorizontalScroll(page), path).toBe(false);
  }

  // The open character menu stays on screen too.
  await page.goto('/dashboard');
  await page.getByRole('button', { name: /Character menu/ }).click();
  await expect(page.getByRole('menuitem', { name: 'Log out' })).toBeInViewport();
  expect(await hasHorizontalScroll(page)).toBe(false);
  noErrors();
});

for (const theme of ['light', 'dark'] as const) {
  test(`has no serious accessibility violations in the ${theme} theme`, async ({
    page,
    context,
    baseURL,
  }) => {
    const admin = seedEverything();
    await useTheme(context, baseURL!, theme);
    await signIn(context, admin, baseURL!);

    for (const path of [...PAGES, '/missing-page']) {
      await page.goto(path);
      await page.waitForLoadState('networkidle');
      if (theme === 'dark') {
        await expect(page.locator('html')).toHaveClass(/dark/);
      } else {
        await expect(page.locator('html')).not.toHaveClass(/dark/);
      }
      const results = await new AxeBuilder({ page })
        .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
        .analyze();
      const serious = results.violations
        .filter((violation) => violation.impact === 'serious' || violation.impact === 'critical')
        .map((violation) => ({
          id: violation.id,
          nodes: violation.nodes.map((node) => node.target.join(' ')),
        }));
      expect(serious, path).toEqual([]);
    }
  });
}
