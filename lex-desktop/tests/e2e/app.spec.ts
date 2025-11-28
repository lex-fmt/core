import { test, expect, _electron as electron } from '@playwright/test';
import path from 'path';

test.describe('Application Launch', () => {
  test('should launch the app and connect to LSP', async () => {
    const electronApp = await electron.launch({
      args: ['.'],
      env: {
        ...process.env,
        NODE_ENV: 'development',
      },
    });

    const window = await electronApp.firstWindow();
    await window.waitForLoadState('domcontentloaded');

    // Check for h1 to ensure React app is mounted
    const h1 = window.locator('h1');
    await expect(h1).toHaveText('Vite + React + LSP');

    // Check for LSP output in the UI
    // We added a pre tag with lspOutput in App.tsx
    const lspOutput = window.locator('pre');
    await expect(lspOutput).toBeVisible();
    
    // Wait for some LSP output (initialization response)
    // The "initialize" request is sent on mount, so we expect a response.
    // We look for "capabilities" which is part of the standard response.
    await expect(lspOutput).toContainText('capabilities', { timeout: 10000 });

    await electronApp.close();
  });
});
