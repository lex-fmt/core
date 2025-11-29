import { forwardRef, useImperativeHandle, useRef, useState, useCallback, useEffect } from 'react';
import { Editor, EditorHandle } from './Editor';
import { TabBar, Tab } from './TabBar';
import { StatusBar } from './StatusBar';
import type * as Monaco from 'monaco-editor';

const AUTO_SAVE_INTERVAL_MS = 5 * 60 * 1000; // 5 minutes

export interface EditorPaneHandle {
    openFile: (path: string) => Promise<void>;
    save: () => Promise<void>;
    getCurrentFile: () => string | null;
    getEditor: () => Monaco.editor.IStandaloneCodeEditor | null;
}

interface EditorPaneProps {
    onFileLoaded?: (path: string | null) => void;
    onCursorChange?: (line: number) => void;
}

// Compute checksum of content (same algorithm as backend)
function computeChecksum(content: string): string {
    let hash = 0;
    for (let i = 0; i < content.length; i++) {
        hash = ((hash << 5) - hash + content.charCodeAt(i)) | 0;
    }
    return hash.toString(16);
}

export const EditorPane = forwardRef<EditorPaneHandle, EditorPaneProps>(function EditorPane(
    { onFileLoaded, onCursorChange },
    ref
) {
    const [tabs, setTabs] = useState<Tab[]>([]);
    const [activeTabId, setActiveTabId] = useState<string | null>(null);
    const [fileToOpen, setFileToOpen] = useState<string | null>(null);
    const [editor, setEditor] = useState<Monaco.editor.IStandaloneCodeEditor | null>(null);
    const [isInitialized, setIsInitialized] = useState(false);
    const editorRef = useRef<EditorHandle>(null);

    // Auto-save state
    const autoSaveIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
    const autoSaveChecksumRef = useRef<string | null>(null);

    const getTabIdFromPath = (path: string) => path;

    // Load persisted tabs on mount
    useEffect(() => {
        const loadPersistedTabs = async () => {
            try {
                const { tabs: savedTabs, activeTab } = await window.ipcRenderer.getOpenTabs();
                if (savedTabs.length > 0) {
                    const newTabs: Tab[] = savedTabs.map(path => ({
                        id: path,
                        path,
                        name: path.split('/').pop() || path
                    }));
                    setTabs(newTabs);
                    if (activeTab) {
                        setActiveTabId(activeTab);
                        setFileToOpen(activeTab);
                    }
                }
            } catch (e) {
                console.error('Failed to load persisted tabs:', e);
            } finally {
                setIsInitialized(true);
            }
        };
        loadPersistedTabs();
    }, []);

    // Persist tabs whenever they change (after initial load)
    useEffect(() => {
        if (!isInitialized) return;
        const tabPaths = tabs.map(t => t.path);
        window.ipcRenderer.setOpenTabs(tabPaths, activeTabId);
    }, [tabs, activeTabId, isInitialized]);

    const openFile = useCallback(async (path: string) => {
        const tabId = getTabIdFromPath(path);
        const existingTab = tabs.find(t => t.id === tabId);

        if (existingTab) {
            // Tab already exists, just activate it
            setActiveTabId(tabId);
            setFileToOpen(path);
        } else {
            // Create new tab
            const name = path.split('/').pop() || path;
            const newTab: Tab = { id: tabId, path, name };
            setTabs(prev => [...prev, newTab]);
            setActiveTabId(tabId);
            setFileToOpen(path);
        }
    }, [tabs]);

    const handleTabSelect = useCallback((tabId: string) => {
        const tab = tabs.find(t => t.id === tabId);
        if (tab) {
            setActiveTabId(tabId);
            setFileToOpen(tab.path);
        }
    }, [tabs]);

    const handleTabClose = useCallback((tabId: string) => {
        // Dispose the model in the editor
        const tab = tabs.find(t => t.id === tabId);
        if (tab) {
            editorRef.current?.closeFile(tab.path);
        }

        setTabs(prev => {
            const newTabs = prev.filter(t => t.id !== tabId);

            // If we're closing the active tab, switch to another
            if (activeTabId === tabId && newTabs.length > 0) {
                const closedIndex = prev.findIndex(t => t.id === tabId);
                const newActiveIndex = Math.min(closedIndex, newTabs.length - 1);
                setActiveTabId(newTabs[newActiveIndex].id);
                setFileToOpen(newTabs[newActiveIndex].path);
            } else if (newTabs.length === 0) {
                setActiveTabId(null);
                setFileToOpen(null);
                onFileLoaded?.(null);
            }

            return newTabs;
        });
    }, [activeTabId, tabs, onFileLoaded]);

    const handleFileLoaded = useCallback((path: string) => {
        // Update editor reference
        setEditor(editorRef.current?.getEditor() ?? null);
        onFileLoaded?.(path);
    }, [onFileLoaded]);

    // Start auto-save interval - captures current file checksum from disk
    const startAutoSaveInterval = useCallback(async () => {
        // Clear any existing interval
        if (autoSaveIntervalRef.current) {
            clearInterval(autoSaveIntervalRef.current);
            autoSaveIntervalRef.current = null;
        }

        const currentFile = editorRef.current?.getCurrentFile();
        if (!currentFile) return;

        // Get checksum of file on disk when starting interval
        const diskChecksum = await window.ipcRenderer.fileChecksum(currentFile);
        autoSaveChecksumRef.current = diskChecksum;

        autoSaveIntervalRef.current = setInterval(async () => {
            const filePath = editorRef.current?.getCurrentFile();
            const editorInstance = editorRef.current?.getEditor();
            if (!filePath || !editorInstance) return;

            // Get current checksum from disk
            const currentDiskChecksum = await window.ipcRenderer.fileChecksum(filePath);

            // Only save if disk checksum matches what we captured
            if (currentDiskChecksum === autoSaveChecksumRef.current) {
                const content = editorInstance.getValue();
                await window.ipcRenderer.fileSave(filePath, content);
                // Update checksum to new content
                autoSaveChecksumRef.current = computeChecksum(content);
                console.log('[AutoSave] Saved file:', filePath);
            } else {
                // File was modified externally - take new checksum for next interval
                autoSaveChecksumRef.current = currentDiskChecksum;
                console.log('[AutoSave] File modified externally, skipping save:', filePath);
            }
        }, AUTO_SAVE_INTERVAL_MS);
    }, []);

    // Stop auto-save interval
    const stopAutoSaveInterval = useCallback(() => {
        if (autoSaveIntervalRef.current) {
            clearInterval(autoSaveIntervalRef.current);
            autoSaveIntervalRef.current = null;
        }
        autoSaveChecksumRef.current = null;
    }, []);

    // Manual save - also resets the auto-save interval
    const handleSave = useCallback(async () => {
        await editorRef.current?.save();
        // Reset auto-save interval after manual save
        await startAutoSaveInterval();
    }, [startAutoSaveInterval]);

    // Auto-save: track window focus
    useEffect(() => {
        const handleFocus = () => {
            startAutoSaveInterval();
        };

        const handleBlur = () => {
            stopAutoSaveInterval();
        };

        window.addEventListener('focus', handleFocus);
        window.addEventListener('blur', handleBlur);

        // Start interval if window is already focused
        if (document.hasFocus()) {
            startAutoSaveInterval();
        }

        return () => {
            window.removeEventListener('focus', handleFocus);
            window.removeEventListener('blur', handleBlur);
            stopAutoSaveInterval();
        };
    }, [startAutoSaveInterval, stopAutoSaveInterval]);

    // Listen for cursor position changes
    useEffect(() => {
        if (!editor || !onCursorChange) return;

        const disposable = editor.onDidChangeCursorPosition((e) => {
            // Monaco uses 1-based lines, LSP uses 0-based
            onCursorChange(e.position.lineNumber - 1);
        });

        // Emit initial position
        const pos = editor.getPosition();
        if (pos) {
            onCursorChange(pos.lineNumber - 1);
        }

        return () => disposable.dispose();
    }, [editor, onCursorChange]);

    useImperativeHandle(ref, () => ({
        openFile,
        save: handleSave,
        getCurrentFile: () => editorRef.current?.getCurrentFile() ?? null,
        getEditor: () => editorRef.current?.getEditor() ?? null,
    }), [openFile, handleSave]);

    return (
        <div className="flex flex-col flex-1 min-h-0">
            <TabBar
                tabs={tabs}
                activeTabId={activeTabId}
                onTabSelect={handleTabSelect}
                onTabClose={handleTabClose}
            />
            <div className="flex-1 min-h-0">
                <Editor
                    ref={editorRef}
                    fileToOpen={fileToOpen}
                    onFileLoaded={handleFileLoaded}
                />
            </div>
            <StatusBar editor={editor} />
        </div>
    );
});
