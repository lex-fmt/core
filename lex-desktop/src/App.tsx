import { useState, useEffect, useRef, useCallback } from 'react'
import { toast } from 'sonner'
import type { EditorPaneHandle } from './components/EditorPane'
import { Layout } from './components/Layout'
import { Outline } from './components/Outline'
import { ExportStatus } from './components/StatusBar'
import { initDebugMonaco } from './debug-monaco'
import { isLexFile } from './components/Editor'
import type { Tab } from './components/TabBar'
import { PaneWorkspace } from './components/PaneWorkspace'
import { MIN_PANE_SIZE, normalizePaneSizes, withRowDefaults } from '@/panes/layout'
import { createEmptyPane, createRowId, usePersistedPaneLayout } from '@/panes/usePersistedPaneLayout'
import { usePaneManager } from '@/panes/usePaneManager'

initDebugMonaco();

const createTabFromPath = (path: string): Tab => ({
  id: path,
  path,
  name: path.split('/').pop() || path,
});

const createPreviewTab = (sourceFile: string, content: string): Tab => {
  const fileName = sourceFile.split('/').pop() || sourceFile;
  const previewId = `preview:${sourceFile}`;
  return {
    id: previewId,
    path: previewId,
    name: `Preview: ${fileName}`,
    type: 'preview',
    previewContent: content,
    sourceFile,
  };
};

function App() {
  const {
    panes,
    paneRows,
    setPanes,
    setPaneRows,
    setActivePaneId,
    resolvedActivePane,
    resolvedActivePaneId,
  } = usePersistedPaneLayout(createTabFromPath);
  const [rootPath, setRootPath] = useState<string | undefined>(undefined);
  const [exportStatus, setExportStatus] = useState<ExportStatus>({ isExporting: false, format: null });
  const paneHandles = useRef(new Map<string, EditorPaneHandle | null>());
  const panesRef = useRef(panes);

  useEffect(() => {
    panesRef.current = panes;
  }, [panes]);

  const activePaneIdValue = resolvedActivePaneId;
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

  const {
    focusPane,
    handleSplitVertical,
    handleSplitHorizontal,
    handleClosePane,
    openFileInPane,
    handleTabSelect,
    handleTabClose,
    handleTabDrop,
    handlePaneFileLoaded,
    handlePaneCursorChange,
  } = usePaneManager({
    activePaneId: activePaneIdValue,
    setActivePaneId,
    setPanes,
    setPaneRows,
    createTabFromPath,
  });

  useEffect(() => {
    if (typeof window === 'undefined') return;
    if (!window.ipcRenderer?.loadTestFixture) return;

    const waitForPaneFile = async (paneId: string, filePath: string, timeoutMs = 5000) => {
      const start = Date.now();
      while (Date.now() - start < timeoutMs) {
        const pane = panesRef.current.find(p => p.id === paneId);
        if (pane?.currentFile === filePath) {
          return;
        }
        await new Promise(resolve => setTimeout(resolve, 50));
      }
      throw new Error(`Timed out opening fixture ${filePath}`);
    };

    const api = {
      openFixture: async (fixtureName: string, targetPaneId?: string | null) => {
        const fixture = await window.ipcRenderer.loadTestFixture(fixtureName);
        const target = targetPaneId ?? activePaneIdValue ?? panes[0]?.id ?? null;
        if (!target) {
          throw new Error('No pane available for fixture');
        }
        openFileInPane(target, fixture.path);
        await waitForPaneFile(target, fixture.path);
        return fixture;
      },
      readFixture: (fixtureName: string) => window.ipcRenderer.loadTestFixture(fixtureName),
      getActiveEditorValue: () => {
        const target = activePaneIdValue ?? panesRef.current[0]?.id ?? null;
        if (!target) {
          return '';
        }
        const editorInstance = paneHandles.current.get(target)?.getEditor();
        return editorInstance?.getValue() ?? '';
      },
      triggerMockDiagnostics: () => {
        const target = activePaneIdValue ?? panesRef.current[0]?.id ?? null;
        if (!target) {
          return false;
        }
        const editorInstance = paneHandles.current.get(target)?.getEditor();
        const model = editorInstance?.getModel?.();
        if (!model) {
          return false;
        }
        const monacoInstance = (window as any).monaco;
        if (!monacoInstance?.editor) {
          return false;
        }
        const lastColumn = model.getLineLength(1) + 1;
        monacoInstance.editor.setModelMarkers(model, 'lex-test', [
          {
            severity: monacoInstance.MarkerSeverity.Error,
            startLineNumber: 1,
            startColumn: 1,
            endLineNumber: 1,
            endColumn: lastColumn,
            message: 'Mock diagnostic for testing',
            source: 'lex-test',
          },
        ]);
        return true;
      },
    };
    window.lexTest = api;
    return () => {
      if (window.lexTest === api) {
        delete window.lexTest;
      }
    };
  }, [activePaneIdValue, openFileInPane, panes]);

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

  const handlePreview = useCallback(async () => {
    console.log('[Preview] handlePreview called');
    console.log('[Preview] activePaneFile:', activePaneFile);
    console.log('[Preview] activePaneIdValue:', activePaneIdValue);

    if (!activePaneFile || !isLexFile(activePaneFile)) {
      console.log('[Preview] ABORT: not a lex file or no file');
      toast.error('Preview requires a .lex file');
      return;
    }

    if (!activePaneIdValue) {
      console.log('[Preview] ABORT: no active pane');
      return;
    }

    // Save the file first
    const handle = paneHandles.current.get(activePaneIdValue);
    console.log('[Preview] Saving file first...');
    await handle?.save();

    try {
      // Convert to HTML in-memory (no file written to disk)
      console.log('[Preview] Calling lexPreview IPC...');
      const htmlContent = await window.ipcRenderer.lexPreview(activePaneFile);
      console.log('[Preview] Got HTML content, length:', htmlContent?.length);

      const previewTab = createPreviewTab(activePaneFile, htmlContent);
      console.log('[Preview] Created preview tab:', previewTab.id, previewTab.name);

      // If only one pane, split vertically first
      console.log('[Preview] panes.length:', panes.length);
      if (panes.length === 1) {
        console.log('[Preview] Creating new pane for preview (single pane mode)');
        const newPane = createEmptyPane();
        console.log('[Preview] New pane id:', newPane.id);
        setPanes(prev => [...prev, { ...newPane, tabs: [previewTab], activeTabId: previewTab.id }]);
        setPaneRows(prevRows => {
          if (prevRows.length === 0) {
            return [withRowDefaults({ id: createRowId(), paneIds: [activePaneIdValue, newPane.id] })];
          }
          return prevRows.map(row => {
            if (!row.paneIds.includes(activePaneIdValue)) return row;
            const paneIds = [...row.paneIds];
            const insertIndex = paneIds.indexOf(activePaneIdValue);
            paneIds.splice(insertIndex + 1, 0, newPane.id);
            const paneSizes = normalizePaneSizes(row, paneIds);
            const currentWeight = paneSizes[activePaneIdValue];
            const splitWeight = Math.max(currentWeight / 2, MIN_PANE_SIZE);
            paneSizes[activePaneIdValue] = splitWeight;
            paneSizes[newPane.id] = splitWeight;
            return { ...row, paneIds, paneSizes };
          });
        });
        setActivePaneId(newPane.id);
        console.log('[Preview] Done - new pane created and activated');
      } else {
        // Open preview in the next pane (not the active one)
        console.log('[Preview] Using existing pane for preview (multi-pane mode)');
        const activeIndex = panes.findIndex(p => p.id === activePaneIdValue);
        const targetIndex = activeIndex === panes.length - 1 ? 0 : activeIndex + 1;
        const targetPaneId = panes[targetIndex].id;
        console.log('[Preview] Target pane id:', targetPaneId);

        setPanes(prev => prev.map(pane => {
          if (pane.id !== targetPaneId) return pane;
          // Check if preview tab already exists for this file
          const existingPreview = pane.tabs.find(t => t.id === previewTab.id);
          if (existingPreview) {
            // Update content and focus
            return {
              ...pane,
              tabs: pane.tabs.map(t => t.id === previewTab.id ? previewTab : t),
              activeTabId: previewTab.id,
            };
          }
          return {
            ...pane,
            tabs: [...pane.tabs, previewTab],
            activeTabId: previewTab.id,
          };
        }));
        setActivePaneId(targetPaneId);
        console.log('[Preview] Done - preview added to existing pane');
      }
    } catch (error) {
      console.error('[Preview] ERROR:', error);
      const message = error instanceof Error ? error.message : 'Preview failed';
      toast.error(message);
    }
  }, [activePaneFile, activePaneIdValue, panes]);

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
    const unsubSplitVertical = window.ipcRenderer.onMenuSplitVertical(handleSplitVertical);
    const unsubSplitHorizontal = window.ipcRenderer.onMenuSplitHorizontal(handleSplitHorizontal);
    const unsubPreview = window.ipcRenderer.onMenuPreview(handlePreview);

    return () => {
      unsubNewFile();
      unsubOpenFile();
      unsubOpenFolder();
      unsubSave();
      unsubFormat();
      unsubExport();
      unsubFind();
      unsubReplace();
      unsubSplitVertical();
      unsubSplitHorizontal();
      unsubPreview();
    };
  }, [handleNewFile, handleOpenFile, handleOpenFolder, handleSave, handleFormat, handleExport, handleFind, handleReplace, handleSplitVertical, handleSplitHorizontal, handlePreview]);

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
      onSplitVertical={handleSplitVertical}
      onSplitHorizontal={handleSplitHorizontal}
      onPreview={handlePreview}
      currentFile={activePaneFile}
      panel={
        <Outline
          currentFile={activePaneFile}
          editor={activeEditor}
          cursorLine={activeCursorLine}
        />
      }
    >
      <PaneWorkspace
        panes={panes}
        paneRows={paneRows}
        activePaneId={activePaneIdValue}
        exportStatus={exportStatus}
        registerPaneHandle={registerPaneHandle}
        onFocusPane={focusPane}
        onClosePane={handleClosePane}
        onTabSelect={handleTabSelect}
        onTabClose={handleTabClose}
        onTabDrop={handleTabDrop}
        onFileLoaded={handlePaneFileLoaded}
        onCursorChange={handlePaneCursorChange}
        onPaneRowsChange={setPaneRows}
      />
    </Layout>
  )
}

export default App
