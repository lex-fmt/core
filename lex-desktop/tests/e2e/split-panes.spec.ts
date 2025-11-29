import { test, expect, _electron as electron } from '@playwright/test';

test.describe('Split Panes', () => {
  test('opens files per pane and syncs outline and explorer', async () => {
    const electronApp = await electron.launch({
      args: ['.'],
      env: {
        ...process.env,
        NODE_ENV: 'development',
      },
    });

    const window = await electronApp.firstWindow();
    window.on('console', (msg) => console.log('renderer:', msg.text()));
    await window.waitForLoadState('domcontentloaded');

    await window.waitForSelector('[data-testid="editor-pane"]');
    const panes = window.locator('[data-testid="editor-pane"]');
    await expect(panes).toHaveCount(2, { timeout: 15000 });

    const fileTree = window.locator('[data-testid="file-tree"]');
    await expect(fileTree).toBeVisible();

    const openTreeItem = async (label: string) => {
      const item = window.locator('[data-testid="file-tree-item"]', { hasText: label }).first();
      await item.click();
      return item;
    };

    // Open general.lex in the first pane
    await panes.nth(0).click();
    await openTreeItem('general.lex');
    const firstPaneTabs = panes.nth(0).locator('[data-testid="editor-tab"]', { hasText: 'general.lex' });
    await expect(firstPaneTabs).toHaveCount(1);
    await expect(panes.nth(1).locator('[data-testid="editor-tab"]', { hasText: 'general.lex' })).toHaveCount(0);

    // Open 20-ideas-naked.lex in the second pane
    await panes.nth(1).click();
    await openTreeItem('20-ideas-naked.lex');
    await expect(panes.nth(1).locator('[data-testid="editor-tab"]', { hasText: '20-ideas-naked.lex' })).toHaveCount(1);

    // File tree selection should follow the active pane
    const generalEntry = window.locator('[data-testid="file-tree-item"][data-path$="general.lex"]');
    const ideasEntry = window.locator('[data-testid="file-tree-item"][data-path$="20-ideas-naked.lex"]');

    await expect(ideasEntry).toHaveAttribute('data-selected', 'true');
    await expect(generalEntry).toHaveAttribute('data-selected', 'false');

    // Switching back to the first pane should update the selection and outline
    await panes.nth(0).click();
    await expect(generalEntry).toHaveAttribute('data-selected', 'true');
    await expect(ideasEntry).toHaveAttribute('data-selected', 'false');

    const outline = window.locator('[data-testid="outline-view"]');
    await window.waitForTimeout(1500);
    await expect(outline.locator('text="1. General"')).toBeVisible();

    await panes.nth(1).click();
    await window.waitForTimeout(1500);
    await expect(outline.locator('text="Ideas, Naked"')).toBeVisible();

    const splitVertical = window.locator('button[title="Split vertically"]');
    await splitVertical.click();
    await expect(window.locator('[data-testid="editor-pane"]')).toHaveCount(3);

    const splitHorizontal = window.locator('button[title="Split horizontally"]');
    await splitHorizontal.click();
    await expect(window.locator('[data-testid="pane-row"]')).toHaveCount(2);

    await electronApp.close();
  });
});
