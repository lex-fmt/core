import { test, expect, _electron as electron } from '@playwright/test';

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

    // Wait for editor
    const editor = window.locator('.monaco-editor').first();
    await expect(editor).toBeVisible();

    // Click Mock Diagnostics button
    const mockButton = window.locator('button:has-text("Mock Diagnostics")');
    await expect(mockButton).toBeVisible();
    await mockButton.click();

    // Check for squiggly error
    // Monaco renders squigglies with class .cdr-error or .squiggly-error
    // Let's check for .squiggly-error
    const squiggly = window.locator('.squiggly-error');
    await expect(squiggly).toBeVisible();

    await electronApp.close();
  });
});
