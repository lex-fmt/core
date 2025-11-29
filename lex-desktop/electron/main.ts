import { app, BrowserWindow, ipcMain, dialog, nativeTheme, Menu, shell } from 'electron'
import { fileURLToPath } from 'node:url'
import path from 'node:path'
import * as fs from 'fs/promises';
import { spawn } from 'child_process';
import { LspManager } from './lsp-manager'

const __dirname = path.dirname(fileURLToPath(import.meta.url))

// Settings persistence
const SETTINGS_FILE = 'settings.json';

interface AppSettings {
  lastFolder?: string;
  openTabs?: string[];
  activeTab?: string;
}

async function getSettingsPath(): Promise<string> {
  return path.join(app.getPath('userData'), SETTINGS_FILE);
}

async function loadSettings(): Promise<AppSettings> {
  try {
    const settingsPath = await getSettingsPath();
    const data = await fs.readFile(settingsPath, 'utf-8');
    return JSON.parse(data);
  } catch {
    return {};
  }
}

async function saveSettings(settings: AppSettings): Promise<void> {
  const settingsPath = await getSettingsPath();
  await fs.writeFile(settingsPath, JSON.stringify(settings, null, 2), 'utf-8');
}

function getWelcomeFolderPath(): string {
  if (app.isPackaged) {
    return path.join(process.resourcesPath, 'welcome');
  }
  return path.join(process.env.APP_ROOT!, 'welcome');
}

function getLexCliPath(): string {
  if (app.isPackaged) {
    return path.join(process.resourcesPath, 'lex');
  }
  // Hardcoded path for dev environment (same pattern as lsp-manager.ts)
  return '/private/tmp/lex/desktop-app/target/debug/lex';
}

/**
 * Maps export format names to file extensions.
 */
const FORMAT_EXTENSIONS: Record<string, string> = {
  markdown: 'md',
  html: 'html',
};

// The built directory structure
//
// ├─┬─┬ dist
// │ │ └── index.html
// │ │
// │ ├─┬ dist-electron
// │ │ ├── main.js
// │ │ └── preload.mjs
// │
process.env.APP_ROOT = path.join(__dirname, '..')

// 🚧 Use ['ENV_NAME'] avoid vite:define plugin - Vite@2.x
export const VITE_DEV_SERVER_URL = process.env['VITE_DEV_SERVER_URL']
export const MAIN_DIST = path.join(process.env.APP_ROOT, 'dist-electron')
export const RENDERER_DIST = path.join(process.env.APP_ROOT, 'dist')

process.env.VITE_PUBLIC = VITE_DEV_SERVER_URL ? path.join(process.env.APP_ROOT, 'public') : RENDERER_DIST

let win: BrowserWindow | null
const lspManager = new LspManager()

function createWindow() {
  win = new BrowserWindow({
    title: 'Lex Editor',
    icon: path.join(process.env.VITE_PUBLIC, 'icon.png'),
    webPreferences: {
      preload: path.join(__dirname, 'preload.mjs'),
    },
  })

  lspManager.setWebContents(win.webContents)
  lspManager.start()

  // Test active push message to Renderer-process.
  win.webContents.on('did-finish-load', () => {
    win?.webContents.send('main-process-message', (new Date).toLocaleString())
  })

  if (VITE_DEV_SERVER_URL) {
    win.loadURL(VITE_DEV_SERVER_URL)
  } else {
    // win.loadFile('dist/index.html')
    win.loadFile(path.join(RENDERER_DIST, 'index.html'))
  }
}

ipcMain.handle('file-new', async (_, defaultPath?: string) => {
  if (!win) return null;
  const { canceled, filePath } = await dialog.showSaveDialog(win, {
    defaultPath: defaultPath || undefined,
    filters: [{ name: 'Lex Files', extensions: ['lex'] }]
  });
  if (canceled || !filePath) {
    return null;
  }
  // Create empty file
  await fs.writeFile(filePath, '', 'utf-8');
  return { filePath, content: '' };
});

ipcMain.handle('file-open', async () => {
  if (!win) return null;
  const { canceled, filePaths } = await dialog.showOpenDialog(win, {
    properties: ['openFile'],
    filters: [{ name: 'Lex Files', extensions: ['lex'] }]
  });
  if (canceled || filePaths.length === 0) {
    return null;
  }
  const filePath = filePaths[0];
  const content = await fs.readFile(filePath, 'utf-8');
  return { filePath, content };
});

ipcMain.handle('file-save', async (_, filePath: string, content: string) => {
  await fs.writeFile(filePath, content, 'utf-8');
  return true;
});

ipcMain.handle('file-read-dir', async (_, dirPath: string) => {
  try {
    const entries = await fs.readdir(dirPath, { withFileTypes: true });
    return entries.map(entry => ({
      name: entry.name,
      isDirectory: entry.isDirectory(),
      path: path.join(dirPath, entry.name)
    }));
  } catch (error) {
    console.error('Failed to read directory:', error);
    return [];
  }
});

ipcMain.handle('file-read', async (_, filePath: string) => {
  try {
    return await fs.readFile(filePath, 'utf-8');
  } catch (error) {
    console.error('Failed to read file:', error);
    return null;
  }
});

/**
 * Computes a checksum of a file's content for auto-save conflict detection.
 *
 * Used by the auto-save system to detect if a file was modified externally
 * (by another editor or process) since the last save.
 *
 * Algorithm: Simple djb2-style hash - fast and collision-resistant enough
 * for this use case. Same algorithm is used in EditorPane.tsx for consistency.
 *
 * @returns Hex string checksum, or null if file doesn't exist/can't be read
 */
ipcMain.handle('file-checksum', async (_, filePath: string) => {
  try {
    const content = await fs.readFile(filePath, 'utf-8');
    let hash = 0;
    for (let i = 0; i < content.length; i++) {
      hash = ((hash << 5) - hash + content.charCodeAt(i)) | 0;
    }
    return hash.toString(16);
  } catch (error) {
    console.error('Failed to compute checksum:', error);
    return null;
  }
});

ipcMain.handle('folder-open', async () => {
  if (!win) return null;
  const { canceled, filePaths } = await dialog.showOpenDialog(win, {
    properties: ['openDirectory']
  });
  if (canceled || filePaths.length === 0) {
    return null;
  }
  return filePaths[0];
});

ipcMain.handle('get-initial-folder', async () => {
  const settings = await loadSettings();
  if (settings.lastFolder) {
    // Verify the folder still exists
    try {
      await fs.access(settings.lastFolder);
      return settings.lastFolder;
    } catch {
      // Folder no longer exists, fall through to welcome folder
    }
  }
  return getWelcomeFolderPath();
});

ipcMain.handle('set-last-folder', async (_, folderPath: string) => {
  const settings = await loadSettings();
  settings.lastFolder = folderPath;
  await saveSettings(settings);
  return true;
});

ipcMain.handle('get-open-tabs', async () => {
  const settings = await loadSettings();
  const tabs = settings.openTabs || [];
  const activeTab = settings.activeTab;

  // Filter out tabs whose files no longer exist
  const existingTabs: string[] = [];
  for (const tab of tabs) {
    try {
      await fs.access(tab);
      existingTabs.push(tab);
    } catch {
      // File no longer exists, skip it
    }
  }

  return {
    tabs: existingTabs,
    activeTab: activeTab && existingTabs.includes(activeTab) ? activeTab : existingTabs[0] || null
  };
});

ipcMain.handle('set-open-tabs', async (_, tabs: string[], activeTab: string | null) => {
  const settings = await loadSettings();
  settings.openTabs = tabs;
  settings.activeTab = activeTab || undefined;
  await saveSettings(settings);
  return true;
});

/**
 * Exports a lex document to another format using the lex CLI.
 *
 * The export process:
 * 1. Takes the source lex file path and target format
 * 2. Computes the output path by replacing .lex extension with format extension
 * 3. Spawns `lex convert <source> --to <format> -o <output>`
 * 4. Returns the output path on success, or throws on error
 *
 * @param sourcePath - Path to the .lex file to export
 * @param format - Target format ('markdown' or 'html')
 * @returns The path to the exported file
 */
ipcMain.handle('file-export', async (_, sourcePath: string, format: string): Promise<string> => {
  const ext = FORMAT_EXTENSIONS[format];
  if (!ext) {
    throw new Error(`Unsupported export format: ${format}`);
  }

  // Compute output path: replace .lex extension with target format extension
  const outputPath = sourcePath.replace(/\.lex$/, `.${ext}`);
  const lexPath = getLexCliPath();

  return new Promise((resolve, reject) => {
    const proc = spawn(lexPath, ['convert', sourcePath, '--to', format, '-o', outputPath]);

    let stderr = '';
    proc.stderr.on('data', (data: Buffer) => {
      stderr += data.toString();
    });

    proc.on('error', (err) => {
      reject(new Error(`Failed to spawn lex CLI: ${err.message}`));
    });

    proc.on('close', (code) => {
      if (code === 0) {
        resolve(outputPath);
      } else {
        reject(new Error(stderr || `lex CLI exited with code ${code}`));
      }
    });
  });
});

/**
 * Shares document content via WhatsApp using the URL scheme.
 * Opens WhatsApp with the document text pre-filled in a new message.
 */
ipcMain.handle('share-whatsapp', async (_, content: string): Promise<void> => {
  const encodedText = encodeURIComponent(content);
  const whatsappUrl = `whatsapp://send?text=${encodedText}`;
  await shell.openExternal(whatsappUrl);
});

// Show item in system file manager
ipcMain.handle('show-item-in-folder', (_, fullPath: string) => {
  shell.showItemInFolder(fullPath);
});

// Theme detection
ipcMain.handle('get-native-theme', () => {
  return nativeTheme.shouldUseDarkColors ? 'dark' : 'light';
});

// Listen for OS theme changes and notify renderer
nativeTheme.on('updated', () => {
  const theme = nativeTheme.shouldUseDarkColors ? 'dark' : 'light';
  if (win && !win.isDestroyed()) {
    win.webContents.send('native-theme-changed', theme);
  }
});

// Quit when all windows are closed, except on macOS. There, it's common
// for applications and their menu bar to stay active until the user quits
// explicitly with Cmd + Q.
app.on('window-all-closed', () => {
  lspManager.stop()
  if (process.platform !== 'darwin') {
    app.quit()
    win = null
  }
})

app.on('activate', () => {
  // On OS X it's common to re-create a window in the app when the
  // dock icon is clicked and there are no other windows open.
  if (BrowserWindow.getAllWindows().length === 0) {
    createWindow()
  }
})

function createMenu() {
  const isMac = process.platform === 'darwin';

  const template: Electron.MenuItemConstructorOptions[] = [
    ...(isMac ? [{
      label: app.name,
      submenu: [
        { role: 'about' as const },
        { type: 'separator' as const },
        { role: 'services' as const },
        { type: 'separator' as const },
        { role: 'hide' as const },
        { role: 'hideOthers' as const },
        { role: 'unhide' as const },
        { type: 'separator' as const },
        { role: 'quit' as const }
      ]
    }] : []),
    {
      label: 'File',
      submenu: [
        {
          label: 'New File',
          accelerator: 'CmdOrCtrl+N',
          click: () => win?.webContents.send('menu-new-file')
        },
        {
          label: 'Open File...',
          accelerator: 'CmdOrCtrl+O',
          click: () => win?.webContents.send('menu-open-file')
        },
        {
          label: 'Open Folder...',
          accelerator: 'CmdOrCtrl+Shift+O',
          click: () => win?.webContents.send('menu-open-folder')
        },
        { type: 'separator' },
        {
          label: 'Save',
          accelerator: 'CmdOrCtrl+S',
          click: () => win?.webContents.send('menu-save')
        },
        { type: 'separator' },
        {
          label: 'Format Document',
          accelerator: 'CmdOrCtrl+Shift+F',
          click: () => win?.webContents.send('menu-format')
        },
        { type: 'separator' },
        {
          label: 'Export',
          submenu: [
            {
              label: 'Export to Markdown',
              click: () => win?.webContents.send('menu-export', 'markdown')
            },
            {
              label: 'Export to HTML',
              click: () => win?.webContents.send('menu-export', 'html')
            }
          ]
        },
        { type: 'separator' },
        isMac ? { role: 'close' } : { role: 'quit' }
      ]
    },
    {
      label: 'Edit',
      submenu: [
        { role: 'undo' },
        { role: 'redo' },
        { type: 'separator' },
        { role: 'cut' },
        { role: 'copy' },
        { role: 'paste' },
        { role: 'selectAll' }
      ]
    },
    {
      label: 'View',
      submenu: [
        { role: 'reload' },
        { role: 'forceReload' },
        { role: 'toggleDevTools' },
        { type: 'separator' },
        { role: 'resetZoom' },
        { role: 'zoomIn' },
        { role: 'zoomOut' },
        { type: 'separator' },
        { role: 'togglefullscreen' }
      ]
    },
    {
      label: 'Window',
      submenu: [
        { role: 'minimize' },
        { role: 'zoom' },
        ...(isMac ? [
          { type: 'separator' as const },
          { role: 'front' as const }
        ] : [
          { role: 'close' as const }
        ])
      ]
    }
  ];

  const menu = Menu.buildFromTemplate(template);
  Menu.setApplicationMenu(menu);
}

app.whenReady().then(() => {
  createMenu();
  createWindow();
})
