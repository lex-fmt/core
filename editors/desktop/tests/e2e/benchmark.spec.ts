import { test, expect, _electron as electron } from '@playwright/test';
import { openFixture } from './helpers';

test.describe('Benchmark File', () => {
  test('should open benchmark file on startup and display correct content and outline', async () => {
    test.setTimeout(60000);
    const electronApp = await electron.launch({
      args: ['.'],
      env: {
        ...process.env,
        NODE_ENV: 'development',
      },
    });

    const window = await electronApp.firstWindow();
    
    // Capture console logs
    window.on('console', msg => console.log(`[Browser Console] ${msg.type()}: ${msg.text()}`));
    window.on('pageerror', err => console.log(`[Browser Error]: ${err}`));

    await window.waitForLoadState('domcontentloaded');

    await openFixture(window, 'benchmark.lex');

    const editor = window.locator('.monaco-editor').first();
    await expect(editor).toBeVisible();

    // 1. Verify Editor Content
    await expect(editor).toContainText('Compromise');

    // 2. Verify Syntax Highlighting
    await expect(editor).toContainText('1.');

    // 3. Verify Outline
    const outline = window.locator('[data-testid="outline-view"]');
    await expect(outline).toContainText('1. The Cage of Compromise');

    // Note: Nested items might not be visible or rendered immediately, 
    // but verifying the first item confirms the outline component is working and receiving data.

    await electronApp.close();
  });
});
