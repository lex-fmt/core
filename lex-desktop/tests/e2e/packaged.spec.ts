import { test, expect, _electron as electron } from '@playwright/test';
import * as path from 'path';
import { openFixture } from './helpers';

test.describe('Packaged Application', () => {
  let electronApp: any = null;
  let page: any = null;

  test.beforeAll(async () => {
    const appPath = path.join(process.cwd(), 'release/mac-arm64/Lex Editor.app/Contents/MacOS/Lex Editor');
    console.log(`Launching app from: ${appPath}`);
    electronApp = await electron.launch({
      executablePath: appPath,
      env: {
        ...process.env,
        LEX_TEST_FIXTURES: path.join(process.cwd(), 'tests/fixtures'),
      },
    });
    page = await electronApp.firstWindow();
    page.on('console', (msg: any) => console.log(`[Browser Console] ${msg.type()}: ${msg.text()}`));
    page.on('pageerror', (err: any) => console.log(`[Browser Page Error] ${err.message}`));
    await page.waitForLoadState('domcontentloaded');
  });

  test.afterAll(async () => {
    await electronApp?.close();
  });

  test('should open benchmark file and display outline', async () => {
    if (!page) throw new Error('Window not initialized');

    await openFixture(page, 'benchmark.lex');

    await page.waitForSelector('.monaco-editor', { timeout: 10000 });
    await expect(page.locator('.monaco-editor').getByText('Compromise').first()).toBeVisible();
    await page.waitForSelector('[data-testid="outline-view"]:has-text("1. The Cage of Compromise")', { timeout: 10000 });
  });
});
