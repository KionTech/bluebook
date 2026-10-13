// Screenshots of the Laravel pages, the baselines of
// frontend/e2e/visual.spec.ts: the pages of frontend/e2e/visual-pages.ts
// at desktop and phone width, tests/fixtures/dataset.json seeded at
// VISUAL_NOW before every shot and the browser's clock at that moment.
//
//   scripts/laravel-reference/run.sh start
//   cd frontend && node ../scripts/laravel-reference/capture.mjs
import { execFileSync } from 'node:child_process';
import { mkdirSync } from 'node:fs';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';

import { VIEWPORTS, VISUAL_NOW, VISUAL_PAGES } from '../../frontend/e2e/visual-pages.ts';

// Playwright from the frontend's dependencies; nothing is installed here.
const { chromium } = createRequire(new URL('../../frontend/package.json', import.meta.url))(
  'playwright',
);

const BASE = 'http://127.0.0.1:8315';
const OUT = new URL('../../frontend/e2e/__screenshots__/visual.spec.ts/', import.meta.url);
mkdirSync(OUT, { recursive: true });

function seed() {
  const record = fileURLToPath(new URL('./record.py', import.meta.url));
  execFileSync('python3', ['-I', record, 'seed'], {
    env: { ...process.env, RECORD_NOW: VISUAL_NOW },
    stdio: 'ignore',
  });
}

const browser = await chromium.launch();
for (const visual of VISUAL_PAGES) {
  for (const [device, viewport] of Object.entries(VIEWPORTS)) {
    seed();
    const context = await browser.newContext({ viewport, locale: 'en-US', timezoneId: 'UTC' });
    const page = await context.newPage();
    await page.clock.setFixedTime(new Date(VISUAL_NOW));
    if (visual.user) {
      await page.request.get(`${BASE}/_reference/login/${visual.user}`);
    }
    await page.goto(BASE + visual.path);
    await page.waitForLoadState('networkidle');
    await page.evaluate(() => document.fonts.ready);
    await page.screenshot({
      path: fileURLToPath(new URL(`${visual.name}-${device}.png`, OUT)),
      animations: 'disabled',
      mask: [page.locator('img')],
    });
    await context.close();
    console.log(visual.name, device);
  }
}
await browser.close();
