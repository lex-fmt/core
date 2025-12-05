import { test, expect } from '@playwright/test';
import { launchApp } from './helpers';
import * as fs from 'fs';
import * as path from 'path';

test.describe('Split Panes', () => {
  test.beforeAll(() => {
    fs.writeFileSync(path.join(process.cwd(), 'general.lex'), '# General\nContent');
    fs.writeFileSync(path.join(process.cwd(), '20-ideas-naked.lex'), '# Ideas, Naked\nContent');
  });

  test.afterAll(() => {
    try {
      fs.unlinkSync(path.join(process.cwd(), 'general.lex'));
      fs.unlinkSync(path.join(process.cwd(), '20-ideas-naked.lex'));
    } catch (e) {
      // Ignore
    }
  });
  test.skip('opens files per pane and syncs outline and explorer', async () => {
    test.setTimeout(60000);
    const electronApp = await launchApp();

    const page = await electronApp.firstWindow();
    page.on('console', (msg) => console.log('renderer:', msg.text()));
    await page.waitForLoadState('domcontentloaded');

    await page.waitForSelector('[data-testid="editor-pane"]');
    const panes = page.locator('[data-testid="editor-pane"]');
    // If we start with 1 pane, split it manually to ensure 2 panes state for test compatibility
    if (await panes.count() === 1) {
       console.log('Only 1 pane found, splitting...');
       await page.click('button[title="Split vertically"]');
    }
    await expect(panes).toHaveCount(2, { timeout: 15000 });

    const fileTree = page.locator('[data-testid="file-tree"]');
    await expect(fileTree).toBeVisible();

    const openTreeItem = async (label: string) => {
      const item = page.locator('[data-testid="file-tree-item"]', { hasText: label }).first();
      await item.click();
      return item;
    };

    // Open general.lex in the first pane
    await panes.nth(0).click();
    await openTreeItem('general.lex');
    const firstPaneTabs = panes.nth(0).locator('[data-testid="editor-tab"]', { hasText: /^general\.lex$/ });
    await expect(firstPaneTabs).toHaveCount(1);

    // Open 20-ideas-naked.lex in the second pane
    await panes.nth(1).click();
    await openTreeItem('20-ideas-naked.lex');
    await expect(panes.nth(1).locator('[data-testid="editor-tab"]', { hasText: /^20-ideas-naked\.lex$/ })).toHaveCount(1);

    // File tree selection should follow the active pane
    const generalEntry = page.locator('[data-testid="file-tree-item"][data-path$="general.lex"]');
    const ideasEntry = page.locator('[data-testid="file-tree-item"][data-path$="20-ideas-naked.lex"]');

    await expect(ideasEntry).toHaveAttribute('data-selected', 'true');
    await expect(generalEntry).toHaveAttribute('data-selected', 'false');

    // Switching back to the first pane should update the selection and outline
    await panes.nth(0).click();
    await expect(generalEntry).toHaveAttribute('data-selected', 'true');
    await expect(ideasEntry).toHaveAttribute('data-selected', 'false');

    const outline = page.locator('[data-testid="outline-view"]');
    await page.waitForTimeout(1500);
    await expect(outline.locator('text="1. General"')).toBeVisible();

    await panes.nth(1).click();
    await page.waitForTimeout(1500);
    await expect(outline.locator('text="Ideas, Naked"')).toBeVisible();

    const splitVertical = page.locator('button[title="Split vertically"]');
    await splitVertical.click();
    await expect(page.locator('[data-testid="editor-pane"]')).toHaveCount(3);

    const splitHorizontal = page.locator('button[title="Split horizontally"]');
    await splitHorizontal.click();
    await expect(page.locator('[data-testid="pane-row"]')).toHaveCount(2);

    const closeButtons = () => page.locator('[data-pane-id] button[title="Close pane"]');
    await closeButtons().last().click();
    await expect(page.locator('[data-testid="pane-row"]')).toHaveCount(1);
    await expect(page.locator('[data-testid="editor-pane"]')).toHaveCount(3);

    await closeButtons().last().click();
    await expect(page.locator('[data-testid="editor-pane"]')).toHaveCount(2);

    await electronApp.close();
  });
});
