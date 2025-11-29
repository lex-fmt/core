import { ReactNode, useEffect, useState } from 'react';
import { cn } from '@/lib/utils';
import { FileTree } from './FileTree';
import { FolderOpen, Settings, PanelLeftClose, PanelLeft, FileText, Save } from 'lucide-react';

interface LayoutProps {
  children: ReactNode;
  panel?: ReactNode;
  rootPath?: string;
  currentFile?: string | null;
  onFileSelect: (path: string) => void;
  onOpenFolder?: () => void;
  onOpenFile?: () => void;
  onSave?: () => void;
}

export function Layout({ children, panel, rootPath, currentFile, onFileSelect, onOpenFolder, onOpenFile, onSave }: LayoutProps) {
  const [leftPanelCollapsed, setLeftPanelCollapsed] = useState(false);

  useEffect(() => {
    const applyTheme = (mode: 'dark' | 'light') => {
      // Set data-theme attribute on document root for CSS variable switching
      document.documentElement.setAttribute('data-theme', mode);
    };

    window.ipcRenderer.getNativeTheme().then(applyTheme);

    const unsubscribe = window.ipcRenderer.onNativeThemeChanged(applyTheme);

    return unsubscribe;
  }, []);

  return (
    <div className="flex flex-col w-screen h-screen overflow-hidden bg-background text-foreground">
      {/* Top Toolbar */}
      <div className="h-10 flex items-center px-3 bg-panel border-b border-border shrink-0 gap-1">
        <button
          onClick={() => setLeftPanelCollapsed(!leftPanelCollapsed)}
          className={cn(
            "p-1.5 rounded",
            "hover:bg-panel-hover transition-colors"
          )}
          title={leftPanelCollapsed ? "Show sidebar" : "Hide sidebar"}
        >
          {leftPanelCollapsed ? <PanelLeft size={16} /> : <PanelLeftClose size={16} />}
        </button>

        <div className="w-px h-5 bg-border mx-1" />

        <button
          onClick={onOpenFolder}
          className={cn(
            "flex items-center gap-2 px-2 py-1.5 rounded text-sm",
            "hover:bg-panel-hover transition-colors"
          )}
          title="Open Folder"
        >
          <FolderOpen size={16} />
        </button>
        <button
          onClick={onOpenFile}
          className={cn(
            "flex items-center gap-2 px-2 py-1.5 rounded text-sm",
            "hover:bg-panel-hover transition-colors"
          )}
          title="Open File"
        >
          <FileText size={16} />
        </button>
        <button
          onClick={onSave}
          disabled={!currentFile}
          className={cn(
            "flex items-center gap-2 px-2 py-1.5 rounded text-sm",
            "hover:bg-panel-hover transition-colors",
            !currentFile && "opacity-50 cursor-not-allowed"
          )}
          title="Save"
        >
          <Save size={16} />
        </button>

        <div className="flex-1 min-w-0 px-2">
          <span className="text-sm text-muted-foreground truncate block">
            {currentFile ? currentFile.split('/').pop() : 'Untitled'}
          </span>
        </div>

        <button
          className={cn(
            "p-1.5 rounded",
            "hover:bg-panel-hover transition-colors"
          )}
          title="Settings"
        >
          <Settings size={16} />
        </button>
      </div>

      {/* Main Content Area */}
      <div className="flex flex-1 min-h-0">
        {/* Left Panel - File Tree & Outline */}
        <div
          className={cn(
            "flex flex-col border-r border-border bg-panel transition-all",
            leftPanelCollapsed ? "w-0" : "w-64"
          )}
        >
          {!leftPanelCollapsed && (
            <>
              {/* File Tree Section */}
              <div className="flex-1 min-h-0 overflow-auto">
                <FileTree rootPath={rootPath} selectedFile={currentFile} onFileSelect={onFileSelect} />
              </div>

              {/* Outline Section */}
              {panel && (
                <div className="h-64 border-t border-border overflow-auto shrink-0">
                  {panel}
                </div>
              )}
            </>
          )}
        </div>

        {/* Editor Area */}
        <div className="flex-1 flex flex-col min-w-0">
          {children}
        </div>
      </div>
    </div>
  );
}
