import { test, expect, _electron as electron } from '@playwright/test';
import { openFixture } from './helpers';

test.describe('Diagnostics', () => {
  test('should show mock diagnostics', async () => {
    const electronApp = await electron.launch({
      args: ['.'],
      env: {
        ...process.env,
        NODE_ENV: 'development',
      },
    });

    const window = await electronApp.firstWindow();
    await window.waitForLoadState('domcontentloaded');
    await openFixture(window, 'diagnostics.lex');

    // Wait for editor
    const editor = window.locator('.monaco-editor').first();
    await expect(editor).toBeVisible();

    await window.evaluate(() => window.lexTest?.triggerMockDiagnostics());

    // Check for squiggly error
    // Monaco renders squigglies with class .cdr-error or .squiggly-error
    // Let's check for .squiggly-error
    const squiggly = window.locator('.squiggly-error');
    await expect(squiggly).toBeVisible();

    await electronApp.close();
  });
});
