// tests/Browser/SetupWizardBrowserTest.php: the first-login wizard.
import { expect, test } from '@playwright/test';

import {
  createCharacter,
  createUser,
  psql,
  resetDatabase,
  signIn,
  watchForErrors,
} from './helpers.ts';

test.beforeEach(() => resetDatabase());

test('walks a new user through the wizard and sets the main character', async ({
  page,
  context,
  baseURL,
}) => {
  const user = createUser({ setUp: false });
  const main = createCharacter({ user, name: 'First Pilot' });
  createCharacter({ user, name: 'Second Pilot' });
  await signIn(context, user, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/dashboard');
  await expect(page.getByText('Welcome, pilot')).toBeVisible();
  await page.getByRole('button', { name: 'Next' }).click();
  await expect(page.getByText('Add all your characters')).toBeVisible();
  await expect(page.getByText('Currently on your account:')).toBeVisible();
  await expect(page.getByRole('dialog').getByText('Second Pilot')).toBeVisible();
  await page.getByRole('button', { name: 'Next' }).click();
  await expect(page.getByText('Standings sync themselves')).toBeVisible();
  await page.getByRole('button', { name: 'Next' }).click();
  await expect(page.getByText('Pick your main character')).toBeVisible();
  await page.locator(`[data-test="wizard-character-${main}"]`).click();
  await page.getByRole('button', { name: 'Set main & finish' }).click();
  await expect(page.getByText('Welcome, pilot')).toHaveCount(0);
  noErrors();

  expect(psql(`select main_character_id from users where id = ${user}`)).toBe(String(main));
  expect(psql(`select setup_completed_at is not null from users where id = ${user}`)).toBe('t');
});

test('lets a new user skip the wizard', async ({ page, context, baseURL }) => {
  const user = createUser({ setUp: false });
  createCharacter({ user });
  await signIn(context, user, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/dashboard');
  await expect(page.getByText('Welcome, pilot')).toBeVisible();
  await page.getByRole('button', { name: 'Skip for now' }).click();
  await expect(page.getByText('Welcome, pilot')).toHaveCount(0);
  noErrors();

  await expect
    .poll(() => psql(`select setup_completed_at is not null from users where id = ${user}`))
    .toBe('t');
  expect(psql(`select main_character_id is null from users where id = ${user}`)).toBe('t');
});

test('does not show the wizard to a user who completed setup', async ({
  page,
  context,
  baseURL,
}) => {
  const user = createUser();
  createCharacter({ user });
  await signIn(context, user, baseURL!);
  const noErrors = watchForErrors(page);

  await page.goto('/dashboard');
  await expect(page.getByText('Current standings')).toBeVisible();

  await expect(page.getByText('Welcome, pilot')).toHaveCount(0);
  noErrors();
});
