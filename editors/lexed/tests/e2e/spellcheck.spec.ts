import { test, expect, _electron as electron } from '@playwright/test';
import * as path from 'path';

test.describe('Spellcheck', () => {
  test('should show diagnostics for misspelled words', async () => {
    const appPath = path.join(
      process.cwd(),
      'release/mac-arm64/LexEd.app/Contents/MacOS/LexEd'
    );
    console.log(`Launching app from: ${appPath}`);
    
    const electronApp = await electron.launch({
      executablePath: appPath,
      env: {
        ...process.env,
        NODE_ENV: 'production', // Use production env
      },
    });

    const window = await electronApp.firstWindow();
    await window.waitForLoadState('domcontentloaded');

    // Wait for editor
    const editor = window.locator('.monaco-editor').first();
    await expect(editor).toBeVisible({ timeout: 20000 });

    // Type a misspelled word
    await window.keyboard.type('Helllo World');

    // Wait for diagnostics
    // The marker title or aria-label often contains the message.
    
    // Let's wait for the squiggly first
    const squiggly = window.locator('.squiggly-error');
    await expect(squiggly).toBeVisible({ timeout: 20000 });

    // Let's try to hover
    await squiggly.hover();
    
    // The hover widget should appear. Class usually `.monaco-hover-content`
    const hover = window.locator('.monaco-hover-content');
    await expect(hover).toBeVisible();
    await expect(hover).toContainText('Unknown word: Helllo');

    await electronApp.close();
  });
});
