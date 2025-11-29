import { useState, useEffect, useRef, useCallback } from 'react'
import { EditorPane, EditorPaneHandle } from './components/EditorPane'
import { Layout } from './components/Layout'
import { Outline } from './components/Outline'
import { initDebugMonaco } from './debug-monaco'

initDebugMonaco();

function App() {
  const [rootPath, setRootPath] = useState<string | undefined>(undefined);
  const [currentFile, setCurrentFile] = useState<string | null>(null);
  const [cursorLine, setCursorLine] = useState<number>(0);
  const editorPaneRef = useRef<EditorPaneHandle>(null);

  const handleNewFile = useCallback(async () => {
    // Use rootPath as the default directory for the save dialog
    const result = await window.ipcRenderer.fileNew(rootPath);
    if (result) {
      await editorPaneRef.current?.openFile(result.filePath);
    }
  }, [rootPath]);

  const handleOpenFolder = useCallback(async () => {
    const result = await window.ipcRenderer.invoke('folder-open');
    if (result) {
      setRootPath(result);
      // Persist the selected folder
      await window.ipcRenderer.setLastFolder(result);
    }
  }, []);

  const handleOpenFile = useCallback(async () => {
    const result = await window.ipcRenderer.fileOpen();
    if (result) {
      await editorPaneRef.current?.openFile(result.filePath);
    }
  }, []);

  const handleSave = useCallback(async () => {
    await editorPaneRef.current?.save();
  }, []);

  const handleFileSelect = useCallback(async (path: string) => {
    await editorPaneRef.current?.openFile(path);
  }, []);

  useEffect(() => {
    console.log('App mounted, initializing Monaco...');
    initDebugMonaco();

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

  // Listen for menu events
  useEffect(() => {
    const unsubNewFile = window.ipcRenderer.onMenuNewFile(handleNewFile);
    const unsubOpenFile = window.ipcRenderer.onMenuOpenFile(handleOpenFile);
    const unsubOpenFolder = window.ipcRenderer.onMenuOpenFolder(handleOpenFolder);
    const unsubSave = window.ipcRenderer.onMenuSave(handleSave);

    return () => {
      unsubNewFile();
      unsubOpenFile();
      unsubOpenFolder();
      unsubSave();
    };
  }, [handleNewFile, handleOpenFile, handleOpenFolder, handleSave]);

  return (
    <Layout
      rootPath={rootPath}
      onFileSelect={handleFileSelect}
      onNewFile={handleNewFile}
      onOpenFolder={handleOpenFolder}
      onOpenFile={handleOpenFile}
      onSave={handleSave}
      currentFile={currentFile}
      panel={
        <Outline
          currentFile={currentFile}
          editor={editorPaneRef.current?.getEditor()}
          cursorLine={cursorLine}
        />
      }
    >
      <EditorPane
        ref={editorPaneRef}
        onFileLoaded={(path) => setCurrentFile(path)}
        onCursorChange={setCursorLine}
      />
    </Layout>
  )
}

export default App
