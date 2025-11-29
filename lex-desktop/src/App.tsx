import { useState, useEffect, useRef } from 'react'
import { Editor, EditorHandle } from './components/Editor'
import { Layout } from './components/Layout'
import { Outline } from './components/Outline'
import { initDebugMonaco } from './debug-monaco'

initDebugMonaco();

function App() {
  const [rootPath, setRootPath] = useState<string | undefined>(undefined);
  const [fileToOpen, setFileToOpen] = useState<string | null>(null);
  const [currentFile, setCurrentFile] = useState<string | null>(null);
  const editorRef = useRef<EditorHandle>(null);

  const handleOpenFolder = async () => {
    const result = await window.ipcRenderer.invoke('folder-open');
    if (result) {
      setRootPath(result);
    }
  };

  const handleOpenFile = async () => {
    await editorRef.current?.openFile();
  };

  const handleSave = async () => {
    await editorRef.current?.save();
  };

  useEffect(() => {
    console.log('App mounted, initializing Monaco...');
    initDebugMonaco();

    const loadBenchmark = async () => {
      try {
        const path = await window.ipcRenderer.getBenchmarkFile();
        if (path) {
          setFileToOpen(path);
        }
      } catch (e) {
        console.error('App: Error loading benchmark file:', e);
      }
    };
    loadBenchmark();
  }, []);

  return (
    <Layout
      rootPath={rootPath}
      onFileSelect={(path) => setFileToOpen(path)}
      onOpenFolder={handleOpenFolder}
      onOpenFile={handleOpenFile}
      onSave={handleSave}
      currentFile={currentFile}
      panel={<Outline currentFile={currentFile} />}
    >
      <Editor
        ref={editorRef}
        fileToOpen={fileToOpen}
        onFileLoaded={(path) => setCurrentFile(path)}
      />
    </Layout>
  )
}

export default App
