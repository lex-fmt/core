import { forwardRef, useImperativeHandle, useRef, useState, useCallback, useEffect } from 'react';
import { Editor, EditorHandle } from './Editor';
import { TabBar, Tab } from './TabBar';
import { StatusBar } from './StatusBar';
import type * as Monaco from 'monaco-editor';

/**
 * Auto-save interval in milliseconds.
 * Files are automatically saved every 5 minutes while the window has focus.
 */
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

/**
 * Computes a simple hash checksum of the given content.
 * Uses the same algorithm as the backend (main.ts) to ensure consistency.
 *
 * This is a fast, non-cryptographic hash suitable for detecting file changes.
 * It's used by auto-save to detect if a file was modified externally.
 */
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

    /**
     * AUTO-SAVE SYSTEM
     *
     * Design goals:
     * 1. Automatically save user work to prevent data loss
     * 2. Never overwrite external changes (e.g., if another editor modifies the file)
     * 3. Only save when the user is actively using the editor (window focused)
     *
     * How it works:
     * - When window gains focus: start a 5-minute interval timer and capture file checksum
     * - When window loses focus: stop the timer (prevents saving stale content)
     * - On each interval tick: compare disk checksum with captured checksum
     *   - If match: safe to save, update checksum to new content
     *   - If mismatch: file was modified externally, take new checksum but don't save
     * - On manual save (Cmd+S): reset interval and checksum
     *
     * The checksum comparison prevents a common scenario:
     * 1. User opens file in Lex, makes edits
     * 2. User switches to another app (auto-save stops)
     * 3. User edits the same file in another editor and saves
     * 4. User returns to Lex (auto-save resumes with fresh checksum)
     * 5. Auto-save won't overwrite because checksums don't match
     */
    const autoSaveIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
    /** Checksum of file on disk when interval started. Used to detect external modifications. */
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

    /**
     * Starts the auto-save interval timer.
     *
     * Called when:
     * - Window gains focus
     * - After a manual save (to reset the timer)
     *
     * Captures the current file's checksum from disk as the baseline for
     * detecting external modifications.
     */
    const startAutoSaveInterval = useCallback(async () => {
        // Clear any existing interval to avoid duplicates
        if (autoSaveIntervalRef.current) {
            clearInterval(autoSaveIntervalRef.current);
            autoSaveIntervalRef.current = null;
        }

        const currentFile = editorRef.current?.getCurrentFile();
        if (!currentFile) return;

        // Capture checksum of file on disk - this is our baseline for detecting external changes
        const diskChecksum = await window.ipcRenderer.fileChecksum(currentFile);
        autoSaveChecksumRef.current = diskChecksum;

        autoSaveIntervalRef.current = setInterval(async () => {
            const filePath = editorRef.current?.getCurrentFile();
            const editorInstance = editorRef.current?.getEditor();
            if (!filePath || !editorInstance) return;

            // Read current checksum from disk to check for external modifications
            const currentDiskChecksum = await window.ipcRenderer.fileChecksum(filePath);

            if (currentDiskChecksum === autoSaveChecksumRef.current) {
                // Checksum matches - file hasn't been modified externally, safe to save
                const content = editorInstance.getValue();
                await window.ipcRenderer.fileSave(filePath, content);
                // Update our baseline checksum to the new content we just saved
                autoSaveChecksumRef.current = computeChecksum(content);
                console.log('[AutoSave] Saved file:', filePath);
            } else {
                // Checksum mismatch - file was modified by another program.
                // Don't overwrite! Instead, update our baseline to the new checksum.
                // This breaks potential infinite loops where we'd never save because
                // checksums keep mismatching.
                autoSaveChecksumRef.current = currentDiskChecksum;
                console.log('[AutoSave] File modified externally, skipping save:', filePath);
            }
        }, AUTO_SAVE_INTERVAL_MS);
    }, []);

    /**
     * Stops the auto-save interval timer.
     *
     * Called when:
     * - Window loses focus (user switched to another app)
     * - Component unmounts
     *
     * Stopping on blur is critical: if the user switches apps and edits the file
     * elsewhere, we don't want a stale auto-save to overwrite their changes.
     */
    const stopAutoSaveInterval = useCallback(() => {
        if (autoSaveIntervalRef.current) {
            clearInterval(autoSaveIntervalRef.current);
            autoSaveIntervalRef.current = null;
        }
        autoSaveChecksumRef.current = null;
    }, []);

    /**
     * Manual save handler.
     *
     * After saving, resets the auto-save interval. This ensures:
     * 1. The timer restarts from zero (user gets full 5 minutes before next auto-save)
     * 2. The checksum is updated to reflect the just-saved content
     */
    const handleSave = useCallback(async () => {
        await editorRef.current?.save();
        // Reset auto-save interval after manual save
        await startAutoSaveInterval();
    }, [startAutoSaveInterval]);

    /**
     * Window focus tracking for auto-save.
     *
     * Why focus-based?
     * - Only save when user is actively using the editor
     * - Prevents saving stale content when user is away
     * - Allows safe editing of the same file in other applications
     */
    useEffect(() => {
        const handleFocus = () => {
            startAutoSaveInterval();
        };

        const handleBlur = () => {
            stopAutoSaveInterval();
        };

        window.addEventListener('focus', handleFocus);
        window.addEventListener('blur', handleBlur);

        // Start interval if window is already focused on mount
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
