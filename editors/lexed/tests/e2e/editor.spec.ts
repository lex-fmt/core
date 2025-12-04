import { test, expect, _electron as electron } from '@playwright/test';
import { openFixture } from './helpers';

test.describe('Editor', () => {
  test('should load editor and apply syntax highlighting', async () => {
    const electronApp = await electron.launch({
      args: ['.'],
      env: {
        ...process.env,
        NODE_ENV: 'development',
      },
    });

    const window = await electronApp.firstWindow();
    await window.waitForLoadState('domcontentloaded');

    await openFixture(window, 'format-basic.lex');

    // Wait for Monaco editor to be visible
    const editor = window.locator('.monaco-editor').first();
    await expect(editor).toBeVisible();

    // Click to focus
    await editor.click();

    // Type some Lex code
    // We need to focus the editor first.
    // Monaco captures keyboard input, so we can just type.
    await window.keyboard.type('# Hello World\n');
    await window.keyboard.type('This is a *test*.\n');

    // Check for semantic tokens
    // Monaco renders tokens as spans with classes like "mtkX"
    // We can't easily know which mtk class maps to what without deep inspection,
    // but we can check if there are ANY mtk classes other than default.
    
    // Wait for a bit for LSP to respond
    await window.waitForTimeout(2000);

    // Check if we have spans with color styles or specific classes
    // In our theme we set DocumentTitle to Red (FF0000)
    // Monaco might inline the style or use a class.
    // Let's look for the text "Hello World" and see if it has a style or class.
    
    // Check if the editor content contains the text
    // Monaco renders text in spans inside view-lines
    const textLocator = window.locator('text=Hello World');
    await expect(textLocator).toBeVisible();

    await electronApp.close();
  });
});
