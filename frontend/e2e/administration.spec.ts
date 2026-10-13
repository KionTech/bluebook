// The admin's overview, settings, standing requests and pilots.
import { expect, test } from '@playwright/test';

import {
  createAdmin,
  createAlliance,
  createCharacter,
  createCorporation,
  createSourceContact,
  createStandingRequest,
  createUser,
  psql,
  resetDatabase,
  setMain,
  signIn,
  watchForErrors,
} from './helpers.ts';

test.beforeEach(() => resetDatabase());

test('shows the overview with stats and the section tabs', async ({ page, context, baseURL }) => {
  const admin = createAdmin();
  const requester = createCharacter({ name: 'Requester Bob' });
  createStandingRequest(requester);
  await signIn(context, admin, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/admin');

  await expect(page.getByText('Overview').first()).toBeVisible();
  await expect(page.getByText('Pending requests').first()).toBeVisible();
  await expect(page.getByText('Standings source').first()).toBeVisible();
  await expect(page.getByText('Requester Bob').first()).toBeVisible();
  noErrors();
});

test('shows the read-only source with confirm-to-change on the settings page', async ({
  page,
  context,
  baseURL,
}) => {
  const admin = createAdmin();
  await signIn(context, admin, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/admin/settings');
  await expect(page.getByText('Standings source').first()).toBeVisible();
  await expect(page.getByText('Discord notifications')).toBeVisible();
  await page.getByRole('button', { name: 'Set source' }).click();

  await expect(page.getByRole('button', { name: 'Save source' })).toBeVisible();
  noErrors();
});

test('confirms and resolves a standing request with a toast', async ({
  page,
  context,
  baseURL,
}) => {
  const admin = createAdmin();
  const pilot = createCharacter({ name: 'Pending Pilot' });
  const request = createStandingRequest(pilot);
  await signIn(context, admin, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/admin/standing-requests');
  await expect(page.getByText('Pending Pilot').first()).toBeVisible();
  await page.getByRole('button', { name: 'Mark done' }).click();
  await expect(page.getByText('Mark request as done?')).toBeVisible();
  await page.getByRole('button', { name: 'Confirm' }).click();

  await expect(page.getByText('Standing request updated.')).toBeVisible();
  noErrors();
  expect(psql(`select status from standing_requests where id = ${request}`)).toBe('done');
});

test('shows a success toast after saving discord settings', async ({ page, context, baseURL }) => {
  const admin = createAdmin();
  await signIn(context, admin, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/admin/settings');
  await page.locator('[name="webhook_url"]').fill('https://discord.com/api/webhooks/1/abc');
  await page.getByRole('button', { name: 'Save Discord settings' }).click();

  await expect(page.getByText('Discord settings updated.')).toBeVisible();
  noErrors();
});

test('lists pilots with their main character and alts', async ({ page, context, baseURL }) => {
  const admin = createAdmin();
  const pilot = createUser();
  const main = createCharacter({ user: pilot, name: 'Main Pilot' });
  createCharacter({ user: pilot, name: 'Alt Pilot' });
  setMain(pilot, main);
  await signIn(context, admin, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/admin/pilots');

  await expect(page.getByText('Main Pilot').filter({ visible: true }).first()).toBeVisible();
  await expect(page.getByText('Alt Pilot').filter({ visible: true }).first()).toBeVisible();
  noErrors();
});

// The pilots

test('renders the pilots corporation view with standings and member characters', async ({
  page,
  context,
  baseURL,
}) => {
  const admin = createAdmin();
  createAlliance(99014466, 'Strix Ridens.', 'STRIX');
  createCorporation(98630705, 'Strix Ridens', 'STRX', 99014466);
  createCorporation(98792349, 'Kion Trading Inc.', 'KION');
  createSourceContact('alliance', 99014466, 10);
  createSourceContact('corporation', 98792349, -5);
  const pilot = createUser();
  const main = createCharacter({
    user: pilot,
    name: 'Nicolas Pilot',
    corporationId: 98630705,
    allianceId: 99014466,
  });
  createCharacter({ user: pilot, name: 'Tina Pilot', corporationId: 98792349 });
  setMain(pilot, main);
  await signIn(context, admin, baseURL!);
  const noErrors = watchForErrors(page);

  // Groups list their member characters directly; collapsing a group
  // hides its rows.
  await page.goto('/admin/pilots?view=corporations');
  await expect(page.getByText('Kion Trading Inc.').first()).toBeVisible();
  await expect(page.getByText('via alliance').first()).toBeVisible();
  await expect(page.getByText('via corporation').first()).toBeVisible();
  await expect(page.getByText('Tina Pilot')).toBeVisible();
  await page.getByText('Kion Trading Inc.').first().click();

  await expect(page.getByText('Tina Pilot')).toHaveCount(0);
  noErrors();
});

test('renders the flat character list with standings', async ({ page, context, baseURL }) => {
  const admin = createAdmin();
  createCorporation(98792349, 'Kion Trading Inc.', 'KION');
  createSourceContact('corporation', 98792349, -5);
  const pilot = createUser();
  const main = createCharacter({ user: pilot, name: 'Nicolas Pilot', corporationId: 98792349 });
  setMain(pilot, main);
  await signIn(context, admin, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/admin/pilots');

  await expect(page.getByText('Nicolas Pilot').first()).toBeVisible();
  await expect(page.getByText('Kion Trading Inc.').first()).toBeVisible();
  await expect(page.getByText('-5').first()).toBeVisible();
  noErrors();
});
