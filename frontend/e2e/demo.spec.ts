// The local demo login: the stack seeds the demo pilots, the home page
// links the login, and it signs in without EVE SSO.
import { execFileSync } from 'node:child_process';

import { expect, test } from '@playwright/test';

import { resetDatabase } from './helpers.ts';

test.beforeEach(() => {
  resetDatabase();
  execFileSync('docker', ['exec', 'bluebook-api', 'seed_demo'], { encoding: 'utf8' });
});

test('the demo login signs in the demo pilot', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('link', { name: 'Demo login' }).click();

  await expect(page).toHaveURL(/\/dashboard$/);
  await expect(page.getByText('Demo Pilot').first()).toBeVisible();
  await expect(page.getByText('Current standings')).toBeVisible();
});
