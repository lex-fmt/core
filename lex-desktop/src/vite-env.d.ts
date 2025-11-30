/// <reference types="vite/client" />

interface Window {
  ipcRenderer: {
    on(channel: string, func: (...args: any[]) => void): () => void;
    off(channel: string, func: (...args: any[]) => void): void;
    send(channel: string, ...args: any[]): void;
    invoke(channel: string, ...args: any[]): Promise<any>;
    fileNew(defaultPath?: string): Promise<{ filePath: string, content: string } | null>;
    fileOpen(): Promise<{ filePath: string, content: string } | null>;
    fileSave(filePath: string, content: string): Promise<boolean>;
    fileReadDir(dirPath: string): Promise<Array<{ name: string, isDirectory: boolean, path: string }>>;
    fileRead(filePath: string): Promise<string | null>;
    fileChecksum(filePath: string): Promise<string | null>;
    folderOpen(): Promise<string | null>;
    getInitialFolder: () => Promise<string>;
    setLastFolder: (folderPath: string) => Promise<boolean>;
    loadTestFixture: (fixtureName: string) => Promise<{ path: string; content: string }>;
    getNativeTheme: () => Promise<'dark' | 'light'>;
    onNativeThemeChanged: (callback: (theme: 'dark' | 'light') => void) => () => void;
    getOpenTabs: () => Promise<{
      panes: Array<{ id: string; tabs: string[]; activeTab: string | null }>;
      activePaneId: string | null;
      rows: Array<{ id: string; paneIds: string[]; size?: number; paneSizes?: Record<string, number> }>;
    }>;
    setOpenTabs: (
      panes: Array<{ id: string; tabs: string[]; activeTab: string | null }>,
      rows: Array<{ id: string; paneIds: string[]; size?: number; paneSizes?: Record<string, number> }>,
      activePaneId: string | null
    ) => Promise<boolean>;
    onMenuNewFile: (callback: () => void) => () => void;
    onMenuOpenFile: (callback: () => void) => () => void;
    onMenuOpenFolder: (callback: () => void) => () => void;
    onMenuSave: (callback: () => void) => () => void;
    onMenuFormat: (callback: () => void) => () => void;
    fileExport: (sourcePath: string, format: string) => Promise<string>;
    lexPreview: (sourcePath: string) => Promise<string>;
    onMenuExport: (callback: (format: string) => void) => () => void;
    shareWhatsApp: (content: string) => Promise<void>;
    showItemInFolder: (fullPath: string) => Promise<void>;
    onMenuFind: (callback: () => void) => () => void;
    onMenuReplace: (callback: () => void) => () => void;
    onMenuSplitVertical: (callback: () => void) => () => void;
    onMenuSplitHorizontal: (callback: () => void) => () => void;
    onMenuPreview: (callback: () => void) => () => void;
  }
  lexTest?: {
    openFixture: (fixtureName: string, paneId?: string | null) => Promise<{ path: string; content: string }>;
    readFixture: (fixtureName: string) => Promise<{ path: string; content: string }>;
    getActiveEditorValue: () => string;
  };
}
