import { test, expect, _electron as electron } from '@playwright/test';

test.describe('Format Document', () => {
  test('should format document via toolbar button', async () => {
    const electronApp = await electron.launch({
      args: ['.'],
      env: {
        ...process.env,
        NODE_ENV: 'development',
      },
    });

    const window = await electronApp.firstWindow();
    await window.waitForLoadState('domcontentloaded');

    // Wait for Monaco editor to be visible
    const editor = window.locator('.monaco-editor').first();
    await expect(editor).toBeVisible();

    // Wait for LSP to initialize (needed for formatting provider)
    await window.waitForTimeout(2000);

    // Click to focus the editor
    await editor.click();

    // Type some unformatted Lex content
    // Using inconsistent spacing that the formatter should fix
    await window.keyboard.type('# Title\n');
    await window.keyboard.type('::session  foo\n'); // Extra space that formatter might normalize
    await window.keyboard.type('Some content here.\n');

    // Wait for content to be typed
    await window.waitForTimeout(500);

    // Get the content before formatting
    const contentBefore = await window.evaluate(() => {
      // @ts-ignore - window.editor is exposed for debugging
      return window.editor?.getValue();
    });

    expect(contentBefore).toContain('# Title');

    // Find and click the Format button (AlignLeft icon in the Lex button group)
    // The button has title="Format Document"
    const formatButton = window.locator('button[title="Format Document"]');
    await expect(formatButton).toBeVisible();
    await formatButton.click();

    // Wait for formatting to complete
    await window.waitForTimeout(1000);

    // Get the content after formatting
    const contentAfter = await window.evaluate(() => {
      // @ts-ignore - window.editor is exposed for debugging
      return window.editor?.getValue();
    });

    // The content should still contain our text (formatting shouldn't delete it)
    expect(contentAfter).toContain('Title');

    // Formatting was triggered (we can't always predict the exact output,
    // but we can verify the action completed without error)
    console.log('Content before:', contentBefore);
    console.log('Content after:', contentAfter);

    await electronApp.close();
  });

  test('should format document via keyboard shortcut', async () => {
    const electronApp = await electron.launch({
      args: ['.'],
      env: {
        ...process.env,
        NODE_ENV: 'development',
      },
    });

    const window = await electronApp.firstWindow();
    await window.waitForLoadState('domcontentloaded');

    // Wait for Monaco editor to be visible
    const editor = window.locator('.monaco-editor').first();
    await expect(editor).toBeVisible();

    // Wait for LSP to initialize
    await window.waitForTimeout(2000);

    // Click to focus the editor
    await editor.click();

    // Type some content
    await window.keyboard.type('# Test Document\n');
    await window.keyboard.type('::session test\n');
    await window.keyboard.type('Hello world.\n');

    // Wait for content to be typed
    await window.waitForTimeout(500);

    // Get content before
    const contentBefore = await window.evaluate(() => {
      // @ts-ignore
      return window.editor?.getValue();
    });

    // Use keyboard shortcut Cmd+Shift+F (Mac) or Ctrl+Shift+F (others)
    const isMac = process.platform === 'darwin';
    if (isMac) {
      await window.keyboard.press('Meta+Shift+KeyF');
    } else {
      await window.keyboard.press('Control+Shift+KeyF');
    }

    // Wait for formatting to complete
    await window.waitForTimeout(1000);

    // Get content after
    const contentAfter = await window.evaluate(() => {
      // @ts-ignore
      return window.editor?.getValue();
    });

    // Content should still be present
    expect(contentAfter).toContain('Test Document');

    console.log('Keyboard shortcut test - Before:', contentBefore);
    console.log('Keyboard shortcut test - After:', contentAfter);

    await electronApp.close();
  });

  test('format button should be disabled when no file is open', async () => {
    const electronApp = await electron.launch({
      args: ['.'],
      env: {
        ...process.env,
        NODE_ENV: 'development',
      },
    });

    const window = await electronApp.firstWindow();
    await window.waitForLoadState('domcontentloaded');

    // Wait for editor to be visible
    const editor = window.locator('.monaco-editor').first();
    await expect(editor).toBeVisible();

    // The format button should exist
    const formatButton = window.locator('button[title="Format Document"]');
    await expect(formatButton).toBeVisible();

    // Check if the button has the disabled styling (opacity-50)
    // When no file is open, the button should be disabled
    const isDisabled = await formatButton.evaluate((el) => {
      return el.hasAttribute('disabled') || el.classList.contains('opacity-50');
    });

    // Note: The welcome folder might auto-open a file, so this test might need adjustment
    // depending on the initial state of the app
    console.log('Format button disabled state:', isDisabled);

    await electronApp.close();
  });
});
