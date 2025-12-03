import { test, expect, _electron as electron } from '@playwright/test';
import { openFixture } from './helpers';
import * as fs from 'fs/promises';

type LexTestWindow = Window & {
  lexTest?: {
    editor?: {
      getValue: () => string;
    };
    getActiveEditorValue: () => string;
  };
};

test.describe('Interop Features', () => {
  test('should convert markdown to lex via convert button', async () => {
    const electronApp = await electron.launch({
      args: ['.'],
      env: { ...process.env, NODE_ENV: 'development' },
    });
    const window = await electronApp.firstWindow();
    await window.waitForLoadState('domcontentloaded');

    // Open a markdown fixture
    const fixture = await openFixture(window, 'sample.md');
    const editor = window.locator('.monaco-editor').first();
    await expect(editor).toBeVisible();
    await window.waitForTimeout(2000); // Wait for LSP

    // Find and click the convert to lex button
    const convertButton = window.locator('button[title="Convert to Lex"]');
    await expect(convertButton).toBeVisible();
    await expect(convertButton).toBeEnabled();
    await convertButton.click();

    // Wait for conversion to complete - look for success toast
    const toast = window.locator('[data-sonner-toast]');
    await expect(toast).toBeVisible({ timeout: 10000 });

    // Check the toast contains success message
    const toastText = await toast.textContent();
    expect(toastText).toContain('Converted');

    // Verify the output file exists
    const outputPath = fixture.path.replace(/\.md$/, '.lex');
    const outputExists = await fs.access(outputPath).then(() => true).catch(() => false);
    expect(outputExists).toBe(true);

    // Verify content is lex (should contain the title)
    const content = await fs.readFile(outputPath, 'utf-8');
    expect(content).toContain('Sample Markdown');

    // Cleanup
    await fs.unlink(outputPath).catch(() => {});
    await electronApp.close();
  });

  test('should export lex to markdown via menu', async () => {
    const electronApp = await electron.launch({
      args: ['.'],
      env: { ...process.env, NODE_ENV: 'development' },
    });
    const window = await electronApp.firstWindow();
    await window.waitForLoadState('domcontentloaded');

    // Open a lex fixture
    const fixture = await openFixture(window, 'format-basic.lex');
    const editor = window.locator('.monaco-editor').first();
    await expect(editor).toBeVisible();
    await window.waitForTimeout(2000); // Wait for LSP

    // Trigger export via menu
    await electronApp.evaluate(({ BrowserWindow }) => {
      const win = BrowserWindow.getAllWindows()[0];
      win.webContents.send('menu-export', 'markdown');
    });

    // Wait for export to complete - look for success toast
    const toast = window.locator('[data-sonner-toast]');
    await expect(toast).toBeVisible({ timeout: 10000 });

    // Check the toast contains success message
    const toastText = await toast.textContent();
    expect(toastText).toContain('Exported');

    // Verify the output file exists
    const outputPath = fixture.path.replace(/\.lex$/, '.md');
    const outputExists = await fs.access(outputPath).then(() => true).catch(() => false);
    expect(outputExists).toBe(true);

    // Verify content is markdown
    const content = await fs.readFile(outputPath, 'utf-8');
    expect(content.length).toBeGreaterThan(0);

    // Cleanup
    await fs.unlink(outputPath).catch(() => {});
    await electronApp.close();
  });

  test('should export lex to html via menu', async () => {
    const electronApp = await electron.launch({
      args: ['.'],
      env: { ...process.env, NODE_ENV: 'development' },
    });
    const window = await electronApp.firstWindow();
    await window.waitForLoadState('domcontentloaded');

    // Open a lex fixture
    const fixture = await openFixture(window, 'format-basic.lex');
    const editor = window.locator('.monaco-editor').first();
    await expect(editor).toBeVisible();
    await window.waitForTimeout(2000); // Wait for LSP

    // Trigger export via menu
    await electronApp.evaluate(({ BrowserWindow }) => {
      const win = BrowserWindow.getAllWindows()[0];
      win.webContents.send('menu-export', 'html');
    });

    // Wait for export to complete
    const toast = window.locator('[data-sonner-toast]');
    await expect(toast).toBeVisible({ timeout: 10000 });

    // Check the toast contains success message
    const toastText = await toast.textContent();
    expect(toastText).toContain('Exported');

    // Verify the output file exists
    const outputPath = fixture.path.replace(/\.lex$/, '.html');
    const outputExists = await fs.access(outputPath).then(() => true).catch(() => false);
    expect(outputExists).toBe(true);

    // Verify content is HTML
    const content = await fs.readFile(outputPath, 'utf-8');
    expect(content).toContain('<');

    // Cleanup
    await fs.unlink(outputPath).catch(() => {});
    await electronApp.close();
  });
});
