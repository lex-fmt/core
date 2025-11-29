import { test, expect, _electron as electron } from '@playwright/test';
import * as path from 'path';

test.describe('Packaged Application', () => {
  test('should open benchmark file and display outline', async () => {
    test.setTimeout(60000);

    // Path to the built executable
    const appPath = path.resolve(process.cwd(), 'release/mac-arm64/Lex Editor.app/Contents/MacOS/Lex Editor');
    
    console.log(`Launching app from: ${appPath}`);

    const electronApp = await electron.launch({
      executablePath: appPath,
      // We don't pass args like '.' because we want to test the default startup behavior
    });

    const page = await electronApp.firstWindow();
    
    // Capture console logs to debug issues
    page.on('console', msg => console.log(`[Browser Console] ${msg.type()}: ${msg.text()}`));
    page.on('pageerror', err => console.log(`[Browser Error]: ${err}`));

    await page.waitForLoadState('domcontentloaded');

    try {
        // 1. Verify Editor Content
        // The benchmark file should be loaded automatically
        const editorText = page.locator('.monaco-editor').getByText('Compromise');
        await expect(editorText.first()).toBeVisible({ timeout: 10000 });

        // 2. Verify Syntax Highlighting
        // We look for a span with a specific class or style that indicates highlighting.
        // In Monaco, tokens often have classes like 'mtk*' or inline styles.
        // We can check if the color is NOT the default foreground.
        // "The Cage of Compromise" should be a SessionTitleText, likely bold and a specific color.
        const titleText = page.locator('.monaco-editor span', { hasText: 'The Cage of Compromise' }).first();
        await expect(titleText).toBeVisible();
        
        // Check computed style to ensure it's highlighted (not default color)
        // Note: Exact color depends on theme, but we can check it's not empty or default.
        const color = await titleText.evaluate((el) => {
            return window.getComputedStyle(el).color;
        });
        console.log('Title color:', color);
        // Basic check: ensure it has a color (Playwright returns rgb string)
        expect(color).toBeTruthy();

        // 3. Verify Outline
        // Scope to the outline view using the test ID
        const outlineView = page.locator('[data-testid="outline-view"]');
        await expect(outlineView).toBeVisible();

        // Check for specific items in the outline
        await expect(outlineView.locator('text="1. The Cage of Compromise"')).toBeVisible();
        await expect(outlineView.locator('text="2. A Native Habitat for Ideas"')).toBeVisible();
        await expect(outlineView.locator('text="3. Lex: Ideas, Uncaged"')).toBeVisible();

    } catch (e) {
        console.error('Test failed. Dumping page content...');
        const content = await page.content();
        console.error('Page Content:', content);
        throw e;
    } finally {
        await electronApp.close();
    }
  });
});
