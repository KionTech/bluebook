// The welcome page and the pilot's dashboard (standings, characters,
// refresh times, sync now, standing requests).
import { expect, test } from '@playwright/test';

import {
  createAlliance,
  createCharacter,
  createCorporation,
  createSource,
  createSourceContact,
  createUser,
  resetDatabase,
  signIn,
  watchForErrors,
} from './helpers.ts';

test.beforeEach(() => resetDatabase());

test('renders the welcome page without errors', async ({ page }) => {
  const noErrors = watchForErrors(page);
  await page.goto('/');
  await expect(page.getByText('Bluebook').first()).toBeVisible();
  noErrors();
});

test('shows a grant-access banner for synced characters missing scopes', async ({
  page,
  context,
  baseURL,
}) => {
  const user = createUser();
  createCharacter({ user, name: 'Tokenless', shouldSync: true });
  await signIn(context, user, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/dashboard');

  const prompt = page.getByRole('alert').filter({ hasText: 'Tokenless needs to log in again.' });
  await expect(prompt).toBeVisible();
  await expect(prompt.getByRole('link', { name: 'Log in again' })).toBeVisible();
  noErrors();
});

test('shows the standings hub with the user characters', async ({ page, context, baseURL }) => {
  const user = createUser();
  createCharacter({ user, name: 'Test Pilot' });
  await signIn(context, user, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/dashboard');

  await expect(page.getByText('Current standings')).toBeVisible();
  await expect(page.getByText('Your characters', { exact: true })).toBeVisible();
  await expect(page.getByText('Test Pilot').first()).toBeVisible();
  noErrors();
});

test('shows the source standings overview with resolved names', async ({
  page,
  context,
  baseURL,
}) => {
  const user = createUser();
  createAlliance(3000, 'Home Alliance');
  createCharacter({ user, allianceId: 3000 });
  createSource('alliance', 3000);
  createCorporation(4000, 'Goonswarm Federation');
  createSourceContact('corporation', 4000, 10);
  await signIn(context, user, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/dashboard');

  await expect(page.getByText('Goonswarm Federation')).toBeVisible();
  await expect(page.getByText('+10')).toBeVisible();
  noErrors();
});

test('lists standings alphabetically within the same standing and type', async ({
  page,
  context,
  baseURL,
}) => {
  const user = createUser();
  createAlliance(3000, 'Home Alliance');
  createCharacter({ user, allianceId: 3000 });
  createSource('alliance', 3000);
  createCorporation(4000, 'Zebra Corp');
  createCorporation(4001, 'Alpha Corp');
  createCorporation(4002, 'Mid Corp');
  createSourceContact('corporation', 4000, 10);
  createSourceContact('corporation', 4001, 10);
  createSourceContact('corporation', 4002, 10);
  await signIn(context, user, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/dashboard');
  await expect(page.getByText('Alpha Corp')).toBeVisible();

  const names = await page
    .locator('[data-test="standing-name"]')
    .evaluateAll((elements) => elements.map((element) => element.textContent?.trim()));
  expect(names).toEqual(['Alpha Corp', 'Mid Corp', 'Zebra Corp']);
  noErrors();
});

test('filters the standings list through the search bar', async ({ page, context, baseURL }) => {
  const user = createUser();
  createAlliance(3000, 'Home Alliance');
  createCharacter({ user, allianceId: 3000 });
  createSource('alliance', 3000);
  createCorporation(4000, 'Blue Corp');
  createCorporation(4001, 'Red Corp');
  createSourceContact('corporation', 4000, 10);
  createSourceContact('corporation', 4001, -10);
  await signIn(context, user, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/dashboard');
  await expect(page.getByText('Blue Corp')).toBeVisible();
  await expect(page.getByText('Red Corp')).toBeVisible();

  const search = page.locator('input[name="standings-search"]');
  await search.fill('blue');
  await expect(page.getByText('Blue Corp')).toBeVisible();
  await expect(page.getByText('Red Corp')).toHaveCount(0);

  await search.fill('Nobody Home');
  await expect(page.getByText('No standings match your search.')).toBeVisible();
  noErrors();
});

test('marks a character that inherits the source as inherited', async ({
  page,
  context,
  baseURL,
}) => {
  const user = createUser();
  createCorporation(2000, 'Source Corp', 'SRCC');
  createCharacter({ user, name: 'Inheritor', corporationId: 2000 });
  createSource('corporation', 2000);
  await signIn(context, user, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/dashboard');

  await expect(page.getByText('Inherited', { exact: true })).toBeVisible();
  await expect(page.getByText('Inherits the source')).toBeVisible();
  noErrors();
});

test('shows each character corporation and alliance', async ({ page, context, baseURL }) => {
  const user = createUser();
  createCorporation(2000, 'Test Corporation', 'TSTC');
  createAlliance(3000, 'Test Alliance', 'TSTA');
  createCharacter({ user, name: 'Test Pilot', corporationId: 2000, allianceId: 3000 });
  await signIn(context, user, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/dashboard');

  await expect(page.getByText('Test Corporation').first()).toBeVisible();
  await expect(page.getByText('Test Alliance').first()).toBeVisible();
  noErrors();
});

test('shows the live last-refreshed time and a next-refresh countdown', async ({
  page,
  context,
  baseURL,
}) => {
  const user = createUser();
  createCharacter({ user });
  createSource('alliance', 3000, 1);
  await signIn(context, user, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/dashboard');

  await expect(page.getByText('Last refreshed')).toBeVisible();
  await expect(page.getByText(/ago/)).toBeVisible();
  await expect(page.getByText('Next refresh')).toBeVisible();
  noErrors();
});

test('lets a user trigger a manual sync from the dashboard', async ({ page, context, baseURL }) => {
  const user = createUser();
  // No write scope: nothing to queue, so the action is ESI-free.
  createCharacter({ user, name: 'Pilot', shouldSync: true });
  await signIn(context, user, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/dashboard');
  await expect(page.getByRole('button', { name: 'Sync now' })).toBeVisible();
  await page.getByRole('button', { name: 'Sync now' }).click();

  await expect(page.getByText('No characters are set to sync.')).toBeVisible();
  noErrors();
});

test('opens a request-standing dialog with the eligible entities', async ({
  page,
  context,
  baseURL,
}) => {
  const user = createUser();
  createCharacter({ user, name: 'Outsider' });
  createSource('character', 999);
  await signIn(context, user, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/dashboard');
  await page.getByRole('button', { name: 'Request standing' }).click();

  await expect(page.getByText('Ask the admins to add')).toBeVisible();
  await expect(page.getByRole('dialog').getByText('Outsider')).toBeVisible();
  noErrors();
});

test('marks a request option that is already in the standings list', async ({
  page,
  context,
  baseURL,
}) => {
  const user = createUser();
  createCorporation(2000, 'Blue Corp');
  createCharacter({ user, name: 'Pilot', corporationId: 2000 });
  createSource('character', 999);
  createSourceContact('corporation', 2000, 5);
  await signIn(context, user, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/dashboard');
  await page.getByRole('button', { name: 'Request standing' }).click();

  await expect(page.getByText('On the list')).toBeVisible();
  noErrors();
});

test('does not offer a request for a character that inherits the source', async ({
  page,
  context,
  baseURL,
}) => {
  const user = createUser();
  createCorporation(2000, 'Source Corp');
  createCharacter({ user, name: 'Member', corporationId: 2000 });
  createSource('corporation', 2000);
  await signIn(context, user, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/dashboard');
  await expect(page.getByText('Your characters', { exact: true })).toBeVisible();

  await expect(page.getByText('Request standing')).toHaveCount(0);
  noErrors();
});
