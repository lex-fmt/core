import { useState, useEffect, useRef } from 'react'
import { EditorPane, EditorPaneHandle } from './components/EditorPane'
import { Layout } from './components/Layout'
import { Outline } from './components/Outline'
import { initDebugMonaco } from './debug-monaco'

initDebugMonaco();

function App() {
  const [rootPath, setRootPath] = useState<string | undefined>(undefined);
  const [currentFile, setCurrentFile] = useState<string | null>(null);
  const editorPaneRef = useRef<EditorPaneHandle>(null);

  const handleOpenFolder = async () => {
    const result = await window.ipcRenderer.invoke('folder-open');
    if (result) {
      setRootPath(result);
      // Persist the selected folder
      await window.ipcRenderer.setLastFolder(result);
    }
  };

  const handleOpenFile = async () => {
    const result = await window.ipcRenderer.fileOpen();
    if (result) {
      await editorPaneRef.current?.openFile(result.filePath);
    }
  };

  const handleSave = async () => {
    await editorPaneRef.current?.save();
  };

  const handleFileSelect = async (path: string) => {
    await editorPaneRef.current?.openFile(path);
  };

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

  return (
    <Layout
      rootPath={rootPath}
      onFileSelect={handleFileSelect}
      onOpenFolder={handleOpenFolder}
      onOpenFile={handleOpenFile}
      onSave={handleSave}
      currentFile={currentFile}
      panel={<Outline currentFile={currentFile} />}
    >
      <EditorPane
        ref={editorPaneRef}
        onFileLoaded={(path) => setCurrentFile(path)}
      />
    </Layout>
  )
}

export default App
