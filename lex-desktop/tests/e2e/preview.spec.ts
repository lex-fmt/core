import { test, expect, _electron as electron } from '@playwright/test';

test.describe('Preview Feature', () => {
  test('opens HTML preview for lex file', async () => {
    const electronApp = await electron.launch({
      args: ['.'],
      env: {
        ...process.env,
        NODE_ENV: 'development',
      },
    });

    const window = await electronApp.firstWindow();
    await window.waitForLoadState('domcontentloaded');

    // Wait for editor pane to appear
    await window.waitForSelector('[data-testid="editor-pane"]', { timeout: 15000 });
    const panes = window.locator('[data-testid="editor-pane"]');

    // Open a lex file from the file tree
    const fileTree = window.locator('[data-testid="file-tree"]');
    await expect(fileTree).toBeVisible();

    // Click on a .lex file
    const lexFile = window.locator('[data-testid="file-tree-item"]', { hasText: 'general.lex' }).first();
    await lexFile.click();
    await window.waitForTimeout(1000);

    // Check the tab is open (use first() since there may be multiple panes with same file)
    const tab = window.locator('[data-testid="editor-tab"]', { hasText: 'general.lex' }).first();
    await expect(tab).toBeVisible();

    // Find and click the preview button
    const previewButton = window.locator('button[title="Preview"]');
    await expect(previewButton).toBeVisible();
    await previewButton.click();

    // Wait for preview to open
    await window.waitForTimeout(2000);

    // Check for preview tab with "Preview:" prefix
    const previewTab = window.locator('[data-testid="editor-tab"]', { hasText: 'Preview:' });
    await expect(previewTab).toBeVisible({ timeout: 5000 });

    // Check for iframe (PreviewPane)
    const iframe = window.locator('iframe[title="Preview"]');
    await expect(iframe).toBeVisible({ timeout: 5000 });

    // Verify the preview tab name includes the filename
    await expect(previewTab).toContainText('Preview: general.lex');

    await electronApp.close();
  });

  test('preview button only shows for lex files', async () => {
    const electronApp = await electron.launch({
      args: ['.'],
      env: {
        ...process.env,
        NODE_ENV: 'development',
      },
    });

    const window = await electronApp.firstWindow();
    await window.waitForLoadState('domcontentloaded');

    // Wait for editor pane to appear
    await window.waitForSelector('[data-testid="editor-pane"]', { timeout: 15000 });

    const fileTree = window.locator('[data-testid="file-tree"]');
    await expect(fileTree).toBeVisible();

    // Open a non-lex file (look for any markdown file)
    const mdFile = window.locator('[data-testid="file-tree-item"]', { hasText: '.md' }).first();
    const mdFileExists = await mdFile.count() > 0;

    if (mdFileExists) {
      await mdFile.click();
      await window.waitForTimeout(500);

      // Preview button should not be visible (it's in the Lex button group which only shows for .lex files)
      const previewButton = window.locator('button[title="Preview"]');
      await expect(previewButton).not.toBeVisible();
    }

    // Now open a .lex file
    const lexFile = window.locator('[data-testid="file-tree-item"]', { hasText: '.lex' }).first();
    await lexFile.click();
    await window.waitForTimeout(500);

    // Preview button should be visible
    const previewButton = window.locator('button[title="Preview"]');
    await expect(previewButton).toBeVisible();

    await electronApp.close();
  });
});
