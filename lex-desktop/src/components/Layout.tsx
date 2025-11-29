import { ReactNode, useEffect, useState, useRef, useCallback } from 'react';
import { cn } from '@/lib/utils';
import { FileTree } from './FileTree';
import { FolderOpen, Settings, PanelLeftClose, PanelLeft, FileText, FilePlus, Save, ChevronDown, ChevronRight } from 'lucide-react';

interface LayoutProps {
  children: ReactNode;
  panel?: ReactNode;
  rootPath?: string;
  currentFile?: string | null;
  onFileSelect: (path: string) => void;
  onNewFile?: () => void;
  onOpenFolder?: () => void;
  onOpenFile?: () => void;
  onSave?: () => void;
}

const MIN_OUTLINE_HEIGHT = 100;
const DEFAULT_OUTLINE_HEIGHT = 200;

export function Layout({ children, panel, rootPath, currentFile, onFileSelect, onNewFile, onOpenFolder, onOpenFile, onSave }: LayoutProps) {
  const [leftPanelCollapsed, setLeftPanelCollapsed] = useState(false);
  const [outlineCollapsed, setOutlineCollapsed] = useState(false);
  const [outlineHeight, setOutlineHeight] = useState(DEFAULT_OUTLINE_HEIGHT);
  const [isDragging, setIsDragging] = useState(false);
  const sidebarRef = useRef<HTMLDivElement>(null);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    setIsDragging(true);
  }, []);

  const handleMouseMove = useCallback((e: MouseEvent) => {
    if (!isDragging || !sidebarRef.current) return;

    const sidebarRect = sidebarRef.current.getBoundingClientRect();
    const newHeight = sidebarRect.bottom - e.clientY;
    const maxHeight = sidebarRect.height - MIN_OUTLINE_HEIGHT;

    setOutlineHeight(Math.max(MIN_OUTLINE_HEIGHT, Math.min(newHeight, maxHeight)));
  }, [isDragging]);

  const handleMouseUp = useCallback(() => {
    setIsDragging(false);
  }, []);

  useEffect(() => {
    if (isDragging) {
      document.addEventListener('mousemove', handleMouseMove);
      document.addEventListener('mouseup', handleMouseUp);
      document.body.style.cursor = 'ns-resize';
      document.body.style.userSelect = 'none';
    } else {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
    }

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
    };
  }, [isDragging, handleMouseMove, handleMouseUp]);

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
          onClick={onNewFile}
          className={cn(
            "flex items-center gap-2 px-2 py-1.5 rounded text-sm",
            "hover:bg-panel-hover transition-colors"
          )}
          title="New File"
        >
          <FilePlus size={16} />
        </button>
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

        <div className="flex-1" />

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
          ref={sidebarRef}
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
                <div className="shrink-0 flex flex-col">
                  {/* Drag Handle */}
                  <div
                    className="h-1 cursor-ns-resize hover:bg-accent/50 active:bg-accent border-t border-border"
                    onMouseDown={handleMouseDown}
                  />
                  {/* Outline Header (collapsible) */}
                  <div
                    className="flex items-center gap-1 px-2.5 py-2 text-xs font-semibold border-b border-border cursor-pointer hover:bg-panel-hover"
                    onClick={() => setOutlineCollapsed(!outlineCollapsed)}
                  >
                    {outlineCollapsed ? <ChevronRight size={12} /> : <ChevronDown size={12} />}
                    <span>Outline</span>
                  </div>
                  {/* Outline Content */}
                  {!outlineCollapsed && (
                    <div
                      className="overflow-auto"
                      style={{ height: outlineHeight }}
                    >
                      {panel}
                    </div>
                  )}
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
