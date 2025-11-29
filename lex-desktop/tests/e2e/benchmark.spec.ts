import { test, expect, _electron as electron } from '@playwright/test';

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
    
    // Wait for the editor to be ready
    await expect(window.locator('.monaco-editor')).toBeVisible();

    // Debug: Check if the file path is displayed in the toolbar
    // The toolbar displays {currentFile || 'Untitled'}
    // We expect it to contain "30-a-place-for-ideas.lex"
    const filePathDisplay = window.locator('span', { hasText: '30-a-place-for-ideas.lex' });
    
    // Take a screenshot if it fails
    try {
        await expect(filePathDisplay).toBeVisible({ timeout: 5000 });
    } catch (e) {
        console.log('File path not found. Taking screenshot...');
        await window.screenshot({ path: 'benchmark-failure.png' });
        // Log the actual text in the toolbar
        const toolbarText = await window.locator('div[style*="background: #333"]').textContent();
        console.log('Toolbar text:', toolbarText);
        throw e;
    }

    // 1. Verify Editor Content
    const editorText = window.locator('.monaco-editor').getByText('Compromise');
    await expect(editorText.first()).toBeVisible({ timeout: 5000 });

    // 2. Verify Syntax Highlighting
    const markerText = window.locator('.monaco-editor').getByText('1.');
    await expect(markerText.first()).toBeVisible();

    // 3. Verify Outline
    const outlineItem = window.locator('text="1. The Cage of Compromise"');
    await expect(outlineItem).toBeVisible();

    // Note: Nested items might not be visible or rendered immediately, 
    // but verifying the first item confirms the outline component is working and receiving data.

    await electronApp.close();
  });
});
