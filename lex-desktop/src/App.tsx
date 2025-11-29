import { useState, useEffect, useRef, useCallback, useMemo } from 'react'
import { toast } from 'sonner'
import { EditorPane, EditorPaneHandle } from './components/EditorPane'
import { Layout } from './components/Layout'
import { Outline } from './components/Outline'
import { ExportStatus } from './components/StatusBar'
import { initDebugMonaco } from './debug-monaco'
import type { Tab, TabDropData } from './components/TabBar'

initDebugMonaco();

interface PaneState {
  id: string;
  tabs: Tab[];
  activeTabId: string | null;
  currentFile: string | null;
  cursorLine: number;
}

interface PaneRowState {
  id: string;
  paneIds: string[];
  size?: number;
  paneSizes?: Record<string, number>;
}

const DEFAULT_ROW_SIZE = 1;
const DEFAULT_PANE_SIZE = 1;
const MIN_ROW_SIZE = 0.1;
const MIN_PANE_SIZE = 0.1;

const createPaneId = () => {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) {
    return crypto.randomUUID();
  }
  return `pane-${Math.random().toString(36).slice(2, 9)}`;
};

const createRowId = () => {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) {
    return crypto.randomUUID();
  }
  return `row-${Math.random().toString(36).slice(2, 9)}`;
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

const getRowSize = (row: PaneRowState): number => (row.size && row.size > 0 ? row.size : DEFAULT_ROW_SIZE);

const normalizePaneSizes = (row: PaneRowState, overridePaneIds?: string[]): Record<string, number> => {
  const normalized: Record<string, number> = {};
  const paneIds = overridePaneIds ?? row.paneIds;
  paneIds.forEach(id => {
    const value = row.paneSizes?.[id];
    normalized[id] = value && value > 0 ? value : DEFAULT_PANE_SIZE;
  });
  return normalized;
};

const getPaneWeight = (row: PaneRowState, paneId: string): number => {
  const value = row.paneSizes?.[paneId];
  return value && value > 0 ? value : DEFAULT_PANE_SIZE;
};

const withRowDefaults = (row: PaneRowState): PaneRowState => ({
  id: row.id,
  paneIds: [...row.paneIds],
  size: row.size && row.size > 0 ? row.size : DEFAULT_ROW_SIZE,
  paneSizes: normalizePaneSizes(row),
});

interface RowResizeState {
  rowId: string;
  nextRowId: string;
  startY: number;
  initialFirstSize: number;
  initialSecondSize: number;
  totalRowSize: number;
  containerHeight: number;
}

interface ColumnResizeState {
  rowId: string;
  leftPaneId: string;
  rightPaneId: string;
  startX: number;
  rowWidth: number;
  initialLeftSize: number;
  initialRightSize: number;
}

function App() {
  const defaultLayoutRef = useRef<{ panes: PaneState[]; rows: PaneRowState[]; activePaneId: string } | null>(null);
  if (!defaultLayoutRef.current) {
    const first = createEmptyPane();
    const second = createEmptyPane();
    const initialRowId = createRowId();
    defaultLayoutRef.current = {
      panes: [first, second],
      rows: [{
        id: initialRowId,
        paneIds: [first.id, second.id],
        size: DEFAULT_ROW_SIZE,
        paneSizes: {
          [first.id]: DEFAULT_PANE_SIZE,
          [second.id]: DEFAULT_PANE_SIZE,
        },
      }],
      activePaneId: first.id,
    };
  }

  const [panes, setPanes] = useState<PaneState[]>(() => defaultLayoutRef.current!.panes);
  const [paneRows, setPaneRows] = useState<PaneRowState[]>(() => defaultLayoutRef.current!.rows.map(withRowDefaults));
  const [activePaneId, setActivePaneId] = useState<string>(() => defaultLayoutRef.current!.activePaneId);
  const [rootPath, setRootPath] = useState<string | undefined>(undefined);
  const [exportStatus, setExportStatus] = useState<ExportStatus>({ isExporting: false, format: null });
  const [layoutInitialized, setLayoutInitialized] = useState(false);
  const paneHandles = useRef(new Map<string, EditorPaneHandle | null>());
  const workspaceRef = useRef<HTMLDivElement>(null);
  const rowRefs = useRef(new Map<string, HTMLDivElement | null>());
  const [rowResize, setRowResize] = useState<RowResizeState | null>(null);
  const [columnResize, setColumnResize] = useState<ColumnResizeState | null>(null);

  const resolvedActivePane = useMemo(() => {
    return panes.find(pane => pane.id === activePaneId) ?? panes[0] ?? null;
  }, [panes, activePaneId]);

  const paneMap = useMemo(() => {
    const map = new Map<string, PaneState>();
    panes.forEach(pane => map.set(pane.id, pane));
    return map;
  }, [panes]);

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
          const hydrated = layout.panes.map<PaneState>((pane) => ({
            id: pane.id || createPaneId(),
            tabs: pane.tabs.map(createTabFromPath),
            activeTabId: pane.activeTab && pane.tabs.includes(pane.activeTab)
              ? pane.activeTab
              : pane.tabs[0] || null,
            currentFile: null,
            cursorLine: 0,
          }));

          if (hydrated.length === 1) {
            hydrated.push(createEmptyPane());
          }

          const paneIdSet = new Set(hydrated.map(p => p.id));
          const rowData = Array.isArray(layout.rows) ? layout.rows : [];
          let rows: PaneRowState[] = rowData
            .map((row: any) => ({
              id: row.id || createRowId(),
              paneIds: Array.isArray(row.paneIds)
                ? row.paneIds.filter((id: string) => paneIdSet.has(id))
                : [],
              size: typeof row.size === 'number' ? row.size : undefined,
              paneSizes: row.paneSizes && typeof row.paneSizes === 'object' ? row.paneSizes : undefined,
            }))
            .filter(row => row.paneIds.length > 0);

          const referencedIds = new Set(rows.flatMap(row => row.paneIds));
          const unreferenced = hydrated
            .map(p => p.id)
            .filter(id => !referencedIds.has(id));

          if (rows.length === 0) {
            rows = [withRowDefaults({ id: createRowId(), paneIds: hydrated.map(p => p.id) })];
          } else if (unreferenced.length > 0) {
            rows[0] = {
              ...rows[0],
              paneIds: [...rows[0].paneIds, ...unreferenced],
            };
          }

          setPanes(hydrated);
          setPaneRows(rows.map(withRowDefaults));

          const savedActiveId = layout.activePaneId && hydrated.some(p => p.id === layout.activePaneId)
            ? layout.activePaneId
            : rows[0]?.paneIds[0] ?? hydrated[0]?.id;
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
        const rowsPayload = paneRows.map(row => ({
          id: row.id,
          paneIds: row.paneIds.filter(id => panes.some(p => p.id === id)),
          size: row.size,
          paneSizes: row.paneSizes,
        }));
        await window.ipcRenderer.setOpenTabs(payload, rowsPayload, activePaneIdValue);
      } catch (error) {
        console.error('Failed to persist pane layout:', error);
      }
    };
    persist();
  }, [panes, paneRows, activePaneIdValue, layoutInitialized]);

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

const updateRowsAfterPaneRemoval = useCallback((paneId: string, remainingPaneIds: string[]) => {
  setPaneRows(prevRows => {
    let removedRowSize = 0;
    let removedIndex = -1;
    const updatedRows: PaneRowState[] = [];

    prevRows.forEach((row, index) => {
      if (!row.paneIds.includes(paneId)) {
        updatedRows.push(row);
        return;
      }

      const paneIds = row.paneIds.filter(id => id !== paneId);
      if (paneIds.length === 0) {
        removedRowSize += getRowSize(row);
        removedIndex = index;
        return;
      }

      const paneSizes = normalizePaneSizes(row, paneIds);
      updatedRows.push({ ...row, paneIds, paneSizes });
    });

    let rows = updatedRows;

    if (rows.length === 0) {
      if (remainingPaneIds.length > 0) {
        rows = [withRowDefaults({ id: createRowId(), paneIds: [remainingPaneIds[0]] })];
      } else {
        rows = [withRowDefaults({ id: createRowId(), paneIds: [] })];
      }
    }

    if (removedRowSize > 0 && rows.length > 0) {
      const targetIndex = Math.min(
        removedIndex >= 0 ? Math.min(removedIndex, rows.length - 1) : rows.length - 1,
        rows.length - 1
      );
      rows = rows.map((row, idx) => (
        idx === targetIndex ? { ...row, size: getRowSize(row) + removedRowSize } : row
      ));
    }

    return rows.map(withRowDefaults);
  });
}, []);

const startRowResize = useCallback((rowId: string, nextRowId: string, clientY: number) => {
  const row = paneRows.find(r => r.id === rowId);
  const nextRow = paneRows.find(r => r.id === nextRowId);
  if (!row || !nextRow || !workspaceRef.current) return;
  const containerHeight = workspaceRef.current.getBoundingClientRect().height || 1;
  const totalRowSize = paneRows.reduce((sum, current) => sum + getRowSize(current), 0);
  setRowResize({
    rowId,
    nextRowId,
    startY: clientY,
    initialFirstSize: getRowSize(row),
    initialSecondSize: getRowSize(nextRow),
    totalRowSize,
    containerHeight,
  });
}, [paneRows]);

const startColumnResize = useCallback((rowId: string, leftPaneId: string, rightPaneId: string, clientX: number) => {
  const row = paneRows.find(r => r.id === rowId);
  const rowElement = rowRefs.current.get(rowId);
  if (!row || !rowElement) return;
  const rowWidth = rowElement.getBoundingClientRect().width || 1;
  const paneSizes = normalizePaneSizes(row);
  setColumnResize({
    rowId,
    leftPaneId,
    rightPaneId,
    startX: clientX,
    rowWidth,
    initialLeftSize: paneSizes[leftPaneId] ?? DEFAULT_PANE_SIZE,
    initialRightSize: paneSizes[rightPaneId] ?? DEFAULT_PANE_SIZE,
  });
}, [paneRows]);

const handleSplitVertical = useCallback(() => {
  if (!activePaneIdValue) return;
  const newPane = createEmptyPane();
  setPanes(prev => [...prev, newPane]);
  setPaneRows(prevRows => {
    if (prevRows.length === 0) {
      return [withRowDefaults({ id: createRowId(), paneIds: [newPane.id] })];
    }
    let handled = false;
    const next = prevRows.map(row => {
      if (!row.paneIds.includes(activePaneIdValue)) {
        return row;
      }
      handled = true;
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
    if (!handled) {
      return [...next, withRowDefaults({ id: createRowId(), paneIds: [newPane.id] })];
    }
    return next;
  });
  setActivePaneId(newPane.id);
}, [activePaneIdValue]);

const handleSplitHorizontal = useCallback(() => {
  if (!activePaneIdValue) return;
  const newPane = createEmptyPane();
  setPanes(prev => [...prev, newPane]);
  setPaneRows(prevRows => {
    if (prevRows.length === 0) {
      return [
        withRowDefaults({ id: createRowId(), paneIds: [activePaneIdValue] }),
        withRowDefaults({ id: createRowId(), paneIds: [newPane.id] }),
      ];
    }
    let handled = false;
    const next: PaneRowState[] = [];
    prevRows.forEach(row => {
      if (!row.paneIds.includes(activePaneIdValue) || handled) {
        next.push(row);
        return;
      }
      handled = true;
      const rowSize = Math.max(getRowSize(row) / 2, MIN_ROW_SIZE);
      const paneSizes = normalizePaneSizes(row);
      next.push({ ...row, size: rowSize, paneSizes });
      next.push(
        withRowDefaults({
          id: createRowId(),
          paneIds: [newPane.id],
          size: rowSize,
          paneSizes: { [newPane.id]: DEFAULT_PANE_SIZE },
        })
      );
    });
    if (!handled) {
      next.push(withRowDefaults({ id: createRowId(), paneIds: [newPane.id] }));
    }
    return next;
  });
  setActivePaneId(newPane.id);
}, [activePaneIdValue]);

useEffect(() => {
  if (!rowResize) return;
  const handleMouseMove = (event: MouseEvent) => {
    const delta = event.clientY - rowResize.startY;
    const deltaSize = (delta / rowResize.containerHeight) * rowResize.totalRowSize;
    const pairSum = rowResize.initialFirstSize + rowResize.initialSecondSize;
    let newFirst = rowResize.initialFirstSize + deltaSize;
    newFirst = Math.max(MIN_ROW_SIZE, Math.min(newFirst, pairSum - MIN_ROW_SIZE));
    const newSecond = pairSum - newFirst;
    setPaneRows(prev => prev.map(row => {
      if (row.id === rowResize.rowId) {
        return { ...row, size: newFirst };
      }
      if (row.id === rowResize.nextRowId) {
        return { ...row, size: newSecond };
      }
      return row;
    }));
  };
  const handleMouseUp = () => setRowResize(null);
  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', handleMouseUp);
  document.body.style.userSelect = 'none';
  document.body.style.cursor = 'row-resize';
  return () => {
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
    document.body.style.userSelect = '';
    document.body.style.cursor = '';
  };
}, [rowResize]);

useEffect(() => {
  if (!columnResize) return;
  const handleMouseMove = (event: MouseEvent) => {
    const delta = event.clientX - columnResize.startX;
    const total = columnResize.initialLeftSize + columnResize.initialRightSize;
    const deltaSize = (delta / columnResize.rowWidth) * total;
    let newLeft = columnResize.initialLeftSize + deltaSize;
    newLeft = Math.max(MIN_PANE_SIZE, Math.min(newLeft, total - MIN_PANE_SIZE));
    const newRight = total - newLeft;
    setPaneRows(prev => prev.map(row => {
      if (row.id !== columnResize.rowId) return row;
      const paneSizes = { ...normalizePaneSizes(row) };
      paneSizes[columnResize.leftPaneId] = newLeft;
      paneSizes[columnResize.rightPaneId] = newRight;
      return { ...row, paneSizes };
    }));
  };
  const handleMouseUp = () => setColumnResize(null);
  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', handleMouseUp);
  document.body.style.userSelect = 'none';
  document.body.style.cursor = 'col-resize';
  return () => {
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
    document.body.style.userSelect = '';
    document.body.style.cursor = '';
  };
}, [columnResize]);

  const handleClosePane = useCallback((paneId: string) => {
    setPanes(prev => {
      if (prev.length <= 1) {
        return prev;
      }
      const filtered = prev.filter(pane => pane.id !== paneId);
      if (filtered.length === prev.length) {
        return prev;
      }
      updateRowsAfterPaneRemoval(paneId, filtered.map(p => p.id));
      if (!filtered.some(pane => pane.id === activePaneId)) {
        setActivePaneId(filtered[0]?.id ?? null);
      }
      return filtered;
    });
  }, [activePaneId, updateRowsAfterPaneRemoval]);

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
        const filtered = next.filter(pane => pane.id !== paneId);
        updateRowsAfterPaneRemoval(paneId, filtered.map(p => p.id));
        return filtered;
      }
      return next;
    });
  }, [updateRowsAfterPaneRemoval]);

  const handleTabDrop = useCallback((targetPaneId: string, data: TabDropData) => {
    const { tabPath, sourcePaneId, duplicate } = data;

    setPanes(prev => {
      const targetPane = prev.find(p => p.id === targetPaneId);
      const sourcePane = prev.find(p => p.id === sourcePaneId);
      if (!targetPane || !sourcePane) return prev;

      // Check if target pane already has this tab
      const existingTab = targetPane.tabs.find(t => t.path === tabPath);
      if (existingTab) {
        // Just focus the existing tab
        return prev.map(pane =>
          pane.id === targetPaneId
            ? { ...pane, activeTabId: existingTab.id }
            : pane
        );
      }

      // Find the tab in source pane
      const sourceTab = sourcePane.tabs.find(t => t.path === tabPath);
      if (!sourceTab) return prev;

      // Create new tab for target
      const newTab = createTabFromPath(tabPath);

      let result = prev.map(pane => {
        if (pane.id === targetPaneId) {
          // Add tab to target pane
          return {
            ...pane,
            tabs: [...pane.tabs, newTab],
            activeTabId: newTab.id,
          };
        }
        if (pane.id === sourcePaneId && !duplicate) {
          // Remove tab from source pane (unless duplicating)
          const remainingTabs = pane.tabs.filter(t => t.path !== tabPath);
          let nextActiveId = pane.activeTabId;
          if (pane.activeTabId === sourceTab.id) {
            const tabIndex = pane.tabs.findIndex(t => t.id === sourceTab.id);
            nextActiveId = remainingTabs.length > 0
              ? remainingTabs[Math.min(tabIndex, remainingTabs.length - 1)].id
              : null;
          }
          return {
            ...pane,
            tabs: remainingTabs,
            activeTabId: nextActiveId,
            currentFile: remainingTabs.length === 0 ? null : pane.currentFile,
            cursorLine: remainingTabs.length === 0 ? 0 : pane.cursorLine,
          };
        }
        return pane;
      });

      // If source pane is now empty and we have more than 1 pane, remove it
      if (!duplicate) {
        const updatedSource = result.find(p => p.id === sourcePaneId);
        if (updatedSource && updatedSource.tabs.length === 0 && result.length > 1) {
          result = result.filter(p => p.id !== sourcePaneId);
          updateRowsAfterPaneRemoval(sourcePaneId, result.map(p => p.id));
        }
      }

      return result;
    });

    setActivePaneId(targetPaneId);
  }, [updateRowsAfterPaneRemoval]);

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
    const unsubSplitVertical = window.ipcRenderer.onMenuSplitVertical(handleSplitVertical);
    const unsubSplitHorizontal = window.ipcRenderer.onMenuSplitHorizontal(handleSplitHorizontal);

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
    };
  }, [handleNewFile, handleOpenFile, handleOpenFolder, handleSave, handleFormat, handleExport, handleFind, handleReplace, handleSplitVertical, handleSplitHorizontal]);

  const renderPanes = () => {
    const totalRowSize = paneRows.reduce((sum, row) => sum + getRowSize(row), 0) || 1;
    return (
      <div className="flex flex-1 flex-col min-h-0" ref={workspaceRef}>
        {paneRows.map((row, rowIndex) => {
          const rowBasis = (getRowSize(row) / totalRowSize) * 100;
          const paneWeights = row.paneIds.map(paneId => getPaneWeight(row, paneId));
          const paneWeightSum = paneWeights.reduce((sum, weight) => sum + weight, 0) || 1;
          return (
            <div key={row.id} className="flex flex-col min-h-0" style={{ flexBasis: `${rowBasis}%` }}>
              <div
                className="flex flex-1 min-h-0"
                ref={(element) => {
                  if (element) {
                    rowRefs.current.set(row.id, element);
                  } else {
                    rowRefs.current.delete(row.id);
                  }
                }}
                data-testid="pane-row"
                data-row-id={row.id}
                data-row-index={rowIndex}
              >
                {row.paneIds.map((paneId, paneIndex) => {
                  const pane = paneMap.get(paneId);
                  if (!pane) return null;
                  const widthPercent = (getPaneWeight(row, paneId) / paneWeightSum) * 100;
                  return (
                    <div key={pane.id} className="flex h-full" style={{ flexBasis: `${widthPercent}%` }}>
                      <div
                        data-testid="editor-pane"
                        data-pane-index={paneIndex}
                        data-pane-id={pane.id}
                        data-active={pane.id === activePaneIdValue}
                        className="relative flex flex-1 flex-col min-w-0"
                        onMouseDown={() => focusPane(pane.id)}
                      >
                        <button
                          className="absolute top-1 right-1 z-10 px-1 text-xs text-muted-foreground hover:text-foreground"
                          title="Close pane"
                          disabled={panes.length <= 1}
                          onClick={(event) => {
                            event.stopPropagation();
                            if (panes.length > 1) {
                              handleClosePane(pane.id);
                            }
                          }}
                        >
                          ×
                        </button>
                        <EditorPane
                          ref={registerPaneHandle(pane.id)}
                          tabs={pane.tabs}
                          activeTabId={pane.activeTabId}
                          paneId={pane.id}
                          onTabSelect={(tabId) => handleTabSelect(pane.id, tabId)}
                          onTabClose={(tabId) => handleTabClose(pane.id, tabId)}
                          onTabDrop={(data) => handleTabDrop(pane.id, data)}
                          onFileLoaded={(path) => handlePaneFileLoaded(pane.id, path)}
                          onCursorChange={(line) => handlePaneCursorChange(pane.id, line)}
                          onActivate={() => focusPane(pane.id)}
                          exportStatus={exportStatus}
                        />
                      </div>
                      {paneIndex < row.paneIds.length - 1 && (
                        <div
                          className="w-1 cursor-col-resize bg-border hover:bg-accent"
                          onMouseDown={(event) => {
                            event.preventDefault();
                            event.stopPropagation();
                            startColumnResize(row.id, pane.id, row.paneIds[paneIndex + 1], event.clientX);
                          }}
                        />
                      )}
                    </div>
                  );
                })}
              </div>
              {rowIndex < paneRows.length - 1 && (
                <div
                  className="h-1 cursor-row-resize bg-border hover:bg-accent"
                  onMouseDown={(event) => {
                    event.preventDefault();
                    event.stopPropagation();
                    startRowResize(row.id, paneRows[rowIndex + 1].id, event.clientY);
                  }}
                />
              )}
            </div>
          );
        })}
      </div>
    );
  };

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
