import { test, expect } from '@playwright/test';
import { openFixture, launchApp } from './helpers';

test.describe('Spellcheck', () => {
  test.skip('should show diagnostics for misspelled words', async () => {
    const electronApp = await launchApp();

    const page = await electronApp.firstWindow();
    await page.waitForLoadState('domcontentloaded');
    
    await openFixture(page, 'empty.lex');

    // Wait for editor
    const editor = page.locator('.monaco-editor').first();
    await expect(editor).toBeVisible({ timeout: 20000 });
    
    // Explicitly focus
    await editor.click();

    // Type a misspelled word
    await page.keyboard.type('Helllo World');

    // Wait for diagnostics
    // The marker title or aria-label often contains the message.
    
    // Let's wait for the squiggly first
    const squiggly = page.locator('.squiggly-error');
    await expect(squiggly).toBeVisible({ timeout: 20000 });

    // Let's try to hover
    await squiggly.hover();
    
    // The hover widget should appear. Class usually `.monaco-hover-content`
    const hover = page.locator('.monaco-hover-content');
    await expect(hover).toBeVisible();
    await expect(hover).toContainText('Unknown word: Helllo');

    await electronApp.close();
  });
});
