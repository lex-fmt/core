import { test, expect, _electron as electron } from '@playwright/test';
import { openFixture } from './helpers';
import * as fs from 'fs/promises';
import * as path from 'path';

type LexTestWindow = Window & {
  lexTest?: {
    editor?: {
      getValue: () => string;
    };
    getActiveEditorValue: () => string;
  };
};

test.describe('Interop Features', () => {
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
