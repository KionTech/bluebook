// Screenshot baselines of the design: the dashboard (dark, the default,
// and light), the admin's requests and pilots, at desktop and phone width.
// Fixed data, a fixed browser clock and EVE images stubbed so only the
// layout can change; the fonts are self-hosted. Fonts and rendering differ between machines, so CI skips them;
// refresh with `npx playwright test design --update-snapshots`.
import { type Page, expect, test } from '@playwright/test';

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
} from './helpers.ts';

test.skip(Boolean(process.env.CI), 'Screenshot baselines only run locally.');

const NOW = new Date('2026-07-24T12:20:00Z');
const SYNCED_AT = '2026-07-24T12:18:44Z';

const VIEWPORTS = [
  { name: 'desktop', width: 1280, height: 900 },
  { name: 'phone', width: 390, height: 844 },
];

// A 1x1 grey PNG for every EVE portrait and logo.
const PIXEL = Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mOsrQcAAYMBAJ/2vVQAAAAASUVORK5CYII=',
  'base64',
);

function seed(): { admin: number; pilot: number } {
  const admin = createAdmin();
  createAlliance(3000, 'Home Alliance', 'HOME');
  createCorporation(2000, 'Home Corp', 'HOMC', 3000);
  createCorporation(4000, 'Blue Corp', 'BLUE');
  createCorporation(4001, 'Kion Trading Inc.', 'KION');
  createAlliance(5000, 'Red Alliance', 'RED');
  createSource('alliance', 3000);
  psql(`update standings_sources set last_synced_at = '${SYNCED_AT}'`);
  createSourceContact('corporation', 4000, 10);
  createSourceContact('corporation', 4001, 5);
  createSourceContact('alliance', 5000, -10);

  const pilot = createUser({ name: 'Nicolas' });
  const main = createCharacter({
    id: 95_100_001,
    user: pilot,
    name: 'Nicolas Pilot',
    corporationId: 2000,
    allianceId: 3000,
  });
  const trader = createCharacter({
    id: 95_100_002,
    user: pilot,
    name: 'Tina Trader',
    corporationId: 4001,
    shouldSync: true,
  });
  grantScopes(trader, ['esi-characters.read_contacts.v1', 'esi-characters.write_contacts.v1']);
  createCharacter({ id: 95_100_003, user: pilot, name: 'Lapsed Alt', shouldSync: true });
  createCharacter({ id: 95_100_004, user: pilot, name: 'Quiet Alt' });
  setMain(pilot, main);

  // Fixed account names: the helpers number their defaults across files.
  psql(`update users set name = 'Admin Account' where id = ${admin}`);
  createStandingRequest(
    createCharacter({ id: 95_100_010, name: 'Asking Pilot', user: createUser({ name: 'Asker' }) }),
  );
  createStandingRequest(
    createCharacter({ id: 95_100_011, name: 'Other Pilot', user: createUser({ name: 'Other' }) }),
  );
  psql(`update standing_requests set created_at = '2026-07-24T09:30:00Z'`);
  return { admin, pilot };
}

async function shoot(page: Page, path: string, name: string) {
  for (const viewport of VIEWPORTS) {
    await page.setViewportSize({ width: viewport.width, height: viewport.height });
    await page.goto(path);
    await page.waitForLoadState('networkidle');
    // One tick of the page clock settles the relative times on the
    // browser's fixed time.
    await page.clock.runFor(1000);
    await expect(page).toHaveScreenshot(`${name}-${viewport.name}.png`, {
      fullPage: true,
      mask: [page.locator('img[src*="images.evetech.net"]')],
    });
  }
}

test.beforeEach(async ({ page }) => {
  resetDatabase();
  await page.clock.install({ time: NOW });
  await page.route('https://images.evetech.net/**', (route) =>
    route.fulfill({ status: 200, contentType: 'image/png', body: PIXEL }),
  );
});

test('dashboard', async ({ page, context, baseURL }) => {
  const { pilot } = seed();
  await signIn(context, pilot, baseURL!);
  await shoot(page, '/dashboard', 'dashboard');
});

test('dashboard in the light theme', async ({ page, context, baseURL }) => {
  const { pilot } = seed();
  const url = new URL(baseURL!);
  await context.addCookies([
    { name: 'appearance', value: 'light', domain: url.hostname, path: '/' },
  ]);
  await context.addInitScript(() => localStorage.setItem('appearance', 'light'));
  await signIn(context, pilot, baseURL!);
  await shoot(page, '/dashboard', 'dashboard-light');
});

test('admin requests', async ({ page, context, baseURL }) => {
  const { admin } = seed();
  await signIn(context, admin, baseURL!);
  await shoot(page, '/admin/standing-requests', 'admin-requests');
});

test('admin pilots', async ({ page, context, baseURL }) => {
  const { admin } = seed();
  await signIn(context, admin, baseURL!);
  await shoot(page, '/admin/pilots', 'admin-pilots');
});
