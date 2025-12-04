import { test, expect, _electron as electron } from '@playwright/test';

test.describe('Settings', () => {
  test('should persist editor settings and apply to Monaco', async () => {
    const electronApp = await electron.launch({
      args: ['.'],
      env: {
        ...process.env,
        NODE_ENV: 'development',
        LEX_DISABLE_PERSISTENCE: '0', // Ensure persistence is enabled
      },
    });

    const window = await electronApp.firstWindow();
    await window.waitForLoadState('domcontentloaded');

    // Open settings
    await window.click('button[title="Settings"]');
    await expect(window.locator('h2:has-text("Settings")')).toBeVisible();

    // Check "Show vertical col width ruler"
    const showRulerCheckbox = window.locator('input#show-ruler');
    await showRulerCheckbox.check();

    // Set ruler width to 120
    const rulerWidthInput = window.locator('input#ruler-width');
    await expect(rulerWidthInput).toBeEnabled();
    await rulerWidthInput.fill('120');

    // Save
    await window.click('text=Save Changes');
    await expect(window.locator('h2:has-text("Settings")')).toBeHidden();

    // Verify settings via IPC
    const settings = await window.evaluate(async () => {
      return await (window as any).ipcRenderer.getAppSettings();
    });
    expect(settings.editor.showRuler).toBe(true);
    expect(settings.editor.rulerWidth).toBe(120);

    // Verify Monaco editor options
    const rulerOption = await window.evaluate(() => {
      const editor = (window as any).editor;
      if (!editor) return null;
      const monaco = (window as any).monaco;
      return editor.getOptions().get(monaco.editor.EditorOption.rulers);
    });
    expect(rulerOption).toEqual([{ column: 120, color: null }]);

    await electronApp.close();
  });
});
