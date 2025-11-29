import { useState, useEffect, useRef, useCallback, useMemo } from 'react'
import { toast } from 'sonner'
import { EditorPane, EditorPaneHandle } from './components/EditorPane'
import { Layout } from './components/Layout'
import { Outline } from './components/Outline'
import { ExportStatus } from './components/StatusBar'
import { initDebugMonaco } from './debug-monaco'
import type { Tab } from './components/TabBar'

initDebugMonaco();

interface PaneState {
  id: string;
  tabs: Tab[];
  activeTabId: string | null;
  currentFile: string | null;
  cursorLine: number;
}

const createPaneId = () => {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) {
    return crypto.randomUUID();
  }
  return `pane-${Math.random().toString(36).slice(2, 9)}`;
};

const createTabFromPath = (path: string): Tab => ({
  id: path,
  path,
  name: path.split('/').pop() || path,
});

const createEmptyPane = (id?: string): PaneState => ({
  id: id || createPaneId(),
  tabs: [],
  activeTabId: null,
  currentFile: null,
  cursorLine: 0,
});

function App() {
  const defaultLayoutRef = useRef<{ panes: PaneState[]; activePaneId: string } | null>(null);
  if (!defaultLayoutRef.current) {
    const first = createEmptyPane();
    const second = createEmptyPane();
    defaultLayoutRef.current = { panes: [first, second], activePaneId: first.id };
  }

  const [panes, setPanes] = useState<PaneState[]>(() => defaultLayoutRef.current!.panes);
  const [activePaneId, setActivePaneId] = useState<string>(() => defaultLayoutRef.current!.activePaneId);
  const [rootPath, setRootPath] = useState<string | undefined>(undefined);
  const [exportStatus, setExportStatus] = useState<ExportStatus>({ isExporting: false, format: null });
  const [layoutInitialized, setLayoutInitialized] = useState(false);
  const paneHandles = useRef(new Map<string, EditorPaneHandle | null>());

  const resolvedActivePane = useMemo(() => {
    return panes.find(pane => pane.id === activePaneId) ?? panes[0] ?? null;
  }, [panes, activePaneId]);

  const activePaneIdValue = resolvedActivePane?.id ?? null;
  const activePaneFile = resolvedActivePane?.currentFile ?? null;
  const activeCursorLine = resolvedActivePane?.cursorLine ?? 0;
  const activeEditor = activePaneIdValue
    ? paneHandles.current.get(activePaneIdValue)?.getEditor() ?? null
    : null;

  const registerPaneHandle = useCallback(
    (paneId: string) => (instance: EditorPaneHandle | null) => {
      if (!instance) {
        return;
      }
      const currentInstance = paneHandles.current.get(paneId) ?? null;
      if (currentInstance === instance) {
        return;
      }
      paneHandles.current.set(paneId, instance);
    },
    []
  );

  useEffect(() => {
    const ids = new Set(panes.map(pane => pane.id));
    for (const [paneId] of paneHandles.current) {
      if (!ids.has(paneId)) {
        paneHandles.current.delete(paneId);
      }
    }
  }, [panes]);


  useEffect(() => {
    const loadLayout = async () => {
      try {
        const layout = await window.ipcRenderer.getOpenTabs();
        if (layout && Array.isArray(layout.panes) && layout.panes.length > 0) {
          let hydrated = layout.panes.map<PaneState>((pane) => ({
            id: pane.id || createPaneId(),
            tabs: pane.tabs.map(createTabFromPath),
            activeTabId: pane.activeTab && pane.tabs.includes(pane.activeTab)
              ? pane.activeTab
              : pane.tabs[0] || null,
            currentFile: null,
            cursorLine: 0,
          }));
          if (hydrated.length === 1) {
            hydrated = [...hydrated, createEmptyPane()];
          }
          setPanes(hydrated);
          const savedActiveId = layout.activePaneId && hydrated.some(p => p.id === layout.activePaneId)
            ? layout.activePaneId
            : hydrated[0]?.id;
          if (savedActiveId) {
            setActivePaneId(savedActiveId);
          }
        }
      } catch (error) {
        console.error('Failed to load pane layout:', error);
      } finally {
        setLayoutInitialized(true);
      }
    };
    loadLayout();
  }, []);

  useEffect(() => {
    if (!layoutInitialized) return;
    const persist = async () => {
      try {
        const payload = panes.map(pane => ({
          id: pane.id,
          tabs: pane.tabs.map(tab => tab.path),
          activeTab: pane.activeTabId,
        }));
        await window.ipcRenderer.setOpenTabs(payload, resolvedActivePane?.id ?? null);
      } catch (error) {
        console.error('Failed to persist pane layout:', error);
      }
    };
    persist();
  }, [panes, resolvedActivePane?.id, layoutInitialized]);

  useEffect(() => {
    if (!panes.length) return;
    if (!panes.some(pane => pane.id === activePaneId)) {
      setActivePaneId(panes[0].id);
    }
  }, [panes, activePaneId]);

  useEffect(() => {
    const loadInitialFolder = async () => {
      try {
        const folder = await window.ipcRenderer.getInitialFolder();
        if (folder) {
          setRootPath(folder);
        }
      } catch (e) {
        console.error('App: Error loading initial folder:', e);
      }
    };
    loadInitialFolder();
  }, []);

  const focusPane = useCallback((paneId: string) => {
    setActivePaneId(paneId);
  }, []);

  const openFileInPane = useCallback((paneId: string, path: string) => {
    let resolvedId: string | null = null;
    setPanes(prev => {
      if (prev.length === 0) {
        const newPane = createEmptyPane();
        const newTab = createTabFromPath(path);
        resolvedId = newPane.id;
        return [{ ...newPane, tabs: [newTab], activeTabId: newTab.id }];
      }
      resolvedId = prev.some(pane => pane.id === paneId) ? paneId : prev[0].id;
      return prev.map(pane => {
        if (pane.id !== resolvedId) return pane;
        const existingTab = pane.tabs.find(tab => tab.path === path);
        if (existingTab) {
          return { ...pane, activeTabId: existingTab.id };
        }
        const newTab = createTabFromPath(path);
        return { ...pane, tabs: [...pane.tabs, newTab], activeTabId: newTab.id };
      });
    });
    if (resolvedId) {
      setActivePaneId(resolvedId);
    }
  }, []);

  const handleTabSelect = useCallback((paneId: string, tabId: string) => {
    setPanes(prev => prev.map(pane => (
      pane.id === paneId ? { ...pane, activeTabId: tabId } : pane
    )));
    setActivePaneId(paneId);
  }, []);

  const handleTabClose = useCallback((paneId: string, tabId: string) => {
    setPanes(prev => {
      let removePane = false;
      const next = prev.map(pane => {
        if (pane.id !== paneId) return pane;
        const tabIndex = pane.tabs.findIndex(tab => tab.id === tabId);
        if (tabIndex === -1) return pane;
        const remainingTabs = pane.tabs.filter(tab => tab.id !== tabId);
        let nextActiveId = pane.activeTabId;
        if (pane.activeTabId === tabId) {
          nextActiveId = remainingTabs.length > 0
            ? remainingTabs[Math.min(tabIndex, remainingTabs.length - 1)].id
            : null;
        }
        const updatedPane: PaneState = {
          ...pane,
          tabs: remainingTabs,
          activeTabId: nextActiveId,
          currentFile: remainingTabs.length === 0 ? null : pane.currentFile,
          cursorLine: remainingTabs.length === 0 ? 0 : pane.cursorLine,
        };
        if (remainingTabs.length === 0 && prev.length > 1) {
          removePane = true;
        }
        return updatedPane;
      });
      if (removePane) {
        return next.filter(pane => pane.id !== paneId);
      }
      return next;
    });
  }, []);

  const handlePaneFileLoaded = useCallback((paneId: string, path: string | null) => {
    setPanes(prev => prev.map(pane => (
      pane.id === paneId ? { ...pane, currentFile: path } : pane
    )));
  }, []);

  const handlePaneCursorChange = useCallback((paneId: string, line: number) => {
    setPanes(prev => prev.map(pane => (
      pane.id === paneId ? { ...pane, cursorLine: line } : pane
    )));
  }, []);

  const handleNewFile = useCallback(async () => {
    if (!activePaneIdValue) return;
    const result = await window.ipcRenderer.fileNew(rootPath);
    if (result) {
      openFileInPane(activePaneIdValue, result.filePath);
    }
  }, [rootPath, activePaneIdValue, openFileInPane]);

  const handleOpenFolder = useCallback(async () => {
    const result = await window.ipcRenderer.invoke('folder-open');
    if (result) {
      setRootPath(result);
      await window.ipcRenderer.setLastFolder(result);
    }
  }, []);

  const handleOpenFile = useCallback(async () => {
    if (!activePaneIdValue) return;
    const result = await window.ipcRenderer.fileOpen();
    if (result) {
      openFileInPane(activePaneIdValue, result.filePath);
    }
  }, [activePaneIdValue, openFileInPane]);

  const handleSave = useCallback(async () => {
    if (!activePaneIdValue) return;
    const handle = paneHandles.current.get(activePaneIdValue);
    await handle?.save();
  }, [activePaneIdValue]);

  const handleFormat = useCallback(async () => {
    if (!activePaneIdValue) return;
    const handle = paneHandles.current.get(activePaneIdValue);
    await handle?.format();
  }, [activePaneIdValue]);

  const handleFind = useCallback(() => {
    if (!activePaneIdValue) return;
    paneHandles.current.get(activePaneIdValue)?.find();
  }, [activePaneIdValue]);

  const handleReplace = useCallback(() => {
    if (!activePaneIdValue) return;
    paneHandles.current.get(activePaneIdValue)?.replace();
  }, [activePaneIdValue]);

  const handleShareWhatsApp = useCallback(async () => {
    if (!activeEditor) {
      toast.error('No document to share');
      return;
    }
    const content = activeEditor.getValue();
    if (!content.trim()) {
      toast.error('Document is empty');
      return;
    }
    await window.ipcRenderer.shareWhatsApp(content);
  }, [activeEditor]);

  const handleConvertToLex = useCallback(async () => {
    if (!activePaneFile || !activePaneIdValue) {
      toast.error('No file open to convert');
      return;
    }

    const handle = paneHandles.current.get(activePaneIdValue);
    await handle?.save();

    setExportStatus({ isExporting: true, format: 'lex' });

    try {
      const outputPath = await window.ipcRenderer.fileExport(activePaneFile, 'lex');
      const fileName = outputPath.split('/').pop() || outputPath;
      toast.success(`Converted to ${fileName}`);
      openFileInPane(activePaneIdValue, outputPath);
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Conversion failed';
      toast.error(message);
    } finally {
      setExportStatus({ isExporting: false, format: null });
    }
  }, [activePaneFile, activePaneIdValue, openFileInPane]);

  const handleExport = useCallback(async (format: string) => {
    if (!activePaneFile) {
      toast.error('No file open to export');
      return;
    }

    if (!activePaneIdValue) return;
    const handle = paneHandles.current.get(activePaneIdValue);
    await handle?.save();

    setExportStatus({ isExporting: true, format });

    try {
      const outputPath = await window.ipcRenderer.fileExport(activePaneFile, format);
      const fileName = outputPath.split('/').pop() || outputPath;
      toast.success(`Exported to ${fileName}`);
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Export failed';
      toast.error(message);
    } finally {
      setExportStatus({ isExporting: false, format: null });
    }
  }, [activePaneFile, activePaneIdValue]);

  const handleFileSelect = useCallback((path: string) => {
    if (!activePaneIdValue) return;
    openFileInPane(activePaneIdValue, path);
  }, [activePaneIdValue, openFileInPane]);

  useEffect(() => {
    const unsubNewFile = window.ipcRenderer.onMenuNewFile(handleNewFile);
    const unsubOpenFile = window.ipcRenderer.onMenuOpenFile(handleOpenFile);
    const unsubOpenFolder = window.ipcRenderer.onMenuOpenFolder(handleOpenFolder);
    const unsubSave = window.ipcRenderer.onMenuSave(handleSave);
    const unsubFormat = window.ipcRenderer.onMenuFormat(handleFormat);
    const unsubExport = window.ipcRenderer.onMenuExport(handleExport);
    const unsubFind = window.ipcRenderer.onMenuFind(handleFind);
    const unsubReplace = window.ipcRenderer.onMenuReplace(handleReplace);

    return () => {
      unsubNewFile();
      unsubOpenFile();
      unsubOpenFolder();
      unsubSave();
      unsubFormat();
      unsubExport();
      unsubFind();
      unsubReplace();
    };
  }, [handleNewFile, handleOpenFile, handleOpenFolder, handleSave, handleFormat, handleExport, handleFind, handleReplace]);

  const renderPanes = () => (
    <div className="flex flex-1 min-h-0">
      {panes.map((pane, index) => (
        <div
          key={pane.id}
          data-testid="editor-pane"
          data-pane-index={index}
          data-pane-id={pane.id}
          data-active={pane.id === activePaneIdValue}
          className={`flex flex-col flex-1 min-w-0 ${index > 0 ? 'border-l border-border' : ''}`}
          onMouseDown={() => focusPane(pane.id)}
        >
          <EditorPane
            ref={registerPaneHandle(pane.id)}
            tabs={pane.tabs}
            activeTabId={pane.activeTabId}
            onTabSelect={(tabId) => handleTabSelect(pane.id, tabId)}
            onTabClose={(tabId) => handleTabClose(pane.id, tabId)}
            onFileLoaded={(path) => handlePaneFileLoaded(pane.id, path)}
            onCursorChange={(line) => handlePaneCursorChange(pane.id, line)}
            onActivate={() => focusPane(pane.id)}
            exportStatus={exportStatus}
          />
        </div>
      ))}
    </div>
  );

  return (
    <Layout
      rootPath={rootPath}
      onFileSelect={handleFileSelect}
      onNewFile={handleNewFile}
      onOpenFolder={handleOpenFolder}
      onOpenFile={handleOpenFile}
      onSave={handleSave}
      onFormat={handleFormat}
      onExport={handleExport}
      onShareWhatsApp={handleShareWhatsApp}
      onConvertToLex={handleConvertToLex}
      onFind={handleFind}
      onReplace={handleReplace}
      currentFile={activePaneFile}
      panel={
        <Outline
          currentFile={activePaneFile}
          editor={activeEditor}
          cursorLine={activeCursorLine}
        />
      }
    >
      {renderPanes()}
    </Layout>
  )
}

export default App
