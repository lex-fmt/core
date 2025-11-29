import { forwardRef, useImperativeHandle, useRef, useState, useCallback, useEffect } from 'react';
import { Editor, EditorHandle } from './Editor';
import { TabBar, Tab } from './TabBar';
import { StatusBar } from './StatusBar';
import type * as Monaco from 'monaco-editor';

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

export const EditorPane = forwardRef<EditorPaneHandle, EditorPaneProps>(function EditorPane(
    { onFileLoaded, onCursorChange },
    ref
) {
    const [tabs, setTabs] = useState<Tab[]>([]);
    const [activeTabId, setActiveTabId] = useState<string | null>(null);
    const [fileToOpen, setFileToOpen] = useState<string | null>(null);
    const [editor, setEditor] = useState<Monaco.editor.IStandaloneCodeEditor | null>(null);
    const editorRef = useRef<EditorHandle>(null);

    const getTabIdFromPath = (path: string) => path;

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

    const handleSave = useCallback(async () => {
        await editorRef.current?.save();
    }, []);

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
