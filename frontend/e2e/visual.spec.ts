// Every page against the screenshot of the Laravel page
// (scripts/laravel-reference/capture.mjs), at desktop and phone width,
// within the 3% pixel tolerance of playwright.config.ts. The EVE images
// are masked. The baselines depend on the host's fonts, so CI skips them.
import { expect, test } from '@playwright/test';

import { psql, resetDatabase, signIn } from './helpers.ts';
import { VIEWPORTS, VISUAL_NOW, VISUAL_PAGES, datasetSql } from './visual-pages.ts';

test.skip(Boolean(process.env.CI), 'screenshots are host specific');

for (const visual of VISUAL_PAGES) {
  for (const [device, viewport] of Object.entries(VIEWPORTS)) {
    test(`${visual.name} looks like Laravel's (${device})`, async ({ page, context, baseURL }) => {
      resetDatabase();
      psql(datasetSql());
      await page.setViewportSize(viewport);
      await page.clock.setFixedTime(new Date(VISUAL_NOW));
      if (visual.user) {
        await signIn(context, visual.user, baseURL!);
      }
      await page.goto(visual.path);
      await page.waitForLoadState('networkidle');
      // The demo login exists only on local stacks (DEMO_ENABLED), not
      // on the site the baselines were taken from.
      await page.addStyleTag({ content: 'a[href="/demo-login"] { display: none !important; }' });
      await page.evaluate(() => document.fonts.ready);
      await expect(page).toHaveScreenshot(`${visual.name}-${device}.png`, {
        fullPage: false,
        mask: [page.locator('img')],
      });
    });
  }
}
