import { app, BrowserWindow, ipcMain, dialog, nativeTheme, Menu } from 'electron'
import { fileURLToPath } from 'node:url'
import path from 'node:path'
import * as fs from 'fs/promises';
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
