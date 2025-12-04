import { test, expect, _electron as electron, Page } from '@playwright/test';

const DEFAULT_EDITOR_SETTINGS = {
  showRuler: false,
  rulerWidth: 100,
  vimMode: false,
};

const DEFAULT_FORMATTER_SETTINGS = {
  sessionBlankLinesBefore: 1,
  sessionBlankLinesAfter: 1,
  normalizeSeqMarkers: true,
  unorderedSeqMarker: '-',
  maxBlankLines: 2,
  indentString: '    ',
  preserveTrailingBlanks: false,
  normalizeVerbatimMarkers: true,
  formatOnSave: false,
};

async function launchApp() {
  const electronApp = await electron.launch({
    args: ['.'],
    env: {
      ...process.env,
      NODE_ENV: 'development',
      LEX_DISABLE_PERSISTENCE: '0',
    },
  });

  const window = await electronApp.firstWindow();
  await window.waitForLoadState('domcontentloaded');
  return { electronApp, window };
}

async function resetAppSettings(window: Page) {
  await window.evaluate(async ({ editor, formatter }) => {
    await window.ipcRenderer.setEditorSettings(editor);
    await window.ipcRenderer.setFormatterSettings(formatter);
  }, { editor: DEFAULT_EDITOR_SETTINGS, formatter: DEFAULT_FORMATTER_SETTINGS });
}

async function openSettings(window: Page) {
  await window.click('button[title="Settings"]');
  await expect(window.locator('h2:has-text("Settings")')).toBeVisible();
}

async function saveAndCloseSettings(window: Page) {
  await window.click('text=Save Changes');
  await expect(window.locator('h2:has-text("Settings")')).toBeHidden();
}

test.describe('Settings', () => {
  test('loads latest formatter values when switching tabs', async () => {
    const { electronApp, window } = await launchApp();

    try {
      await resetAppSettings(window);
      await openSettings(window);

      const customFormatter = {
        sessionBlankLinesBefore: 4,
        sessionBlankLinesAfter: 3,
        normalizeSeqMarkers: false,
        unorderedSeqMarker: '+',
        maxBlankLines: 5,
        indentString: ' '.repeat(6),
        preserveTrailingBlanks: true,
        normalizeVerbatimMarkers: false,
        formatOnSave: true,
      };

      await window.evaluate(async (formatter) => {
        await window.ipcRenderer.setFormatterSettings(formatter);
      }, customFormatter);

      await window.click('button:has-text("Formatter")');

      await expect(window.locator('input#session-before')).toHaveValue(String(customFormatter.sessionBlankLinesBefore));
      await expect(window.locator('input#session-after')).toHaveValue(String(customFormatter.sessionBlankLinesAfter));
      await expect(window.locator('input#max-blank-lines')).toHaveValue(String(customFormatter.maxBlankLines));
      await expect(window.locator('input#indent-spaces')).toHaveValue(String(customFormatter.indentString.length));
      await expect(window.locator('input#unordered-marker')).toHaveValue(customFormatter.unorderedSeqMarker);
      await expect(window.locator('label:has-text("Normalize list markers") input[type="checkbox"]')).not.toBeChecked();
      await expect(window.locator('label:has-text("Normalize verbatim markers") input[type="checkbox"]')).not.toBeChecked();
      await expect(window.locator('label:has-text("Preserve trailing blank lines") input[type="checkbox"]')).toBeChecked();
      await expect(window.locator('label:has-text("Format automatically on save") input[type="checkbox"]')).toBeChecked();
    } finally {
      await electronApp.close();
    }
  });

  test('persists UI and formatter settings after closing dialog', async () => {
    const { electronApp, window } = await launchApp();

    try {
      await resetAppSettings(window);
      await openSettings(window);

      const showRulerCheckbox = window.locator('input#show-ruler');
      await showRulerCheckbox.check();
      const rulerWidthInput = window.locator('input#ruler-width');
      await rulerWidthInput.fill('120');
      await window.locator('input#vim-mode').check();

      await window.click('button:has-text("Formatter")');
      await window.locator('input#session-before').fill('2');
      await window.locator('input#session-after').fill('3');
      await window.locator('input#max-blank-lines').fill('4');
      await window.locator('input#indent-spaces').fill('2');
      await window.locator('input#unordered-marker').fill('*');
      await window.locator('label:has-text("Normalize list markers") input[type="checkbox"]').uncheck();
      await window.locator('label:has-text("Normalize verbatim markers") input[type="checkbox"]').uncheck();
      await window.locator('label:has-text("Preserve trailing blank lines") input[type="checkbox"]').check();
      await window.locator('label:has-text("Format automatically on save") input[type="checkbox"]').check();

      await saveAndCloseSettings(window);

      await openSettings(window);
      await expect(window.locator('input#show-ruler')).toBeChecked();
      await expect(window.locator('input#ruler-width')).toHaveValue('120');
      await expect(window.locator('input#vim-mode')).toBeChecked();

      await window.click('button:has-text("Formatter")');
      await expect(window.locator('input#session-before')).toHaveValue('2');
      await expect(window.locator('input#session-after')).toHaveValue('3');
      await expect(window.locator('input#max-blank-lines')).toHaveValue('4');
      await expect(window.locator('input#indent-spaces')).toHaveValue('2');
      await expect(window.locator('input#unordered-marker')).toHaveValue('*');
      await expect(window.locator('label:has-text("Normalize list markers") input[type="checkbox"]')).not.toBeChecked();
      await expect(window.locator('label:has-text("Normalize verbatim markers") input[type="checkbox"]')).not.toBeChecked();
      await expect(window.locator('label:has-text("Preserve trailing blank lines") input[type="checkbox"]')).toBeChecked();
      await expect(window.locator('label:has-text("Format automatically on save") input[type="checkbox"]')).toBeChecked();

      const persistedSettings = await window.evaluate(async () => {
        return window.ipcRenderer.getAppSettings();
      });
      expect(persistedSettings.editor.showRuler).toBe(true);
      expect(persistedSettings.editor.rulerWidth).toBe(120);
      expect(persistedSettings.editor.vimMode).toBe(true);
      expect(persistedSettings.formatter.sessionBlankLinesBefore).toBe(2);
      expect(persistedSettings.formatter.sessionBlankLinesAfter).toBe(3);
      expect(persistedSettings.formatter.maxBlankLines).toBe(4);
      expect(persistedSettings.formatter.indentString).toBe(' '.repeat(2));
      expect(persistedSettings.formatter.unorderedSeqMarker).toBe('*');
      expect(persistedSettings.formatter.normalizeSeqMarkers).toBe(false);
      expect(persistedSettings.formatter.normalizeVerbatimMarkers).toBe(false);
      expect(persistedSettings.formatter.preserveTrailingBlanks).toBe(true);
      expect(persistedSettings.formatter.formatOnSave).toBe(true);
    } finally {
      await electronApp.close();
    }
  });
});
