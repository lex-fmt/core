import './App.css'
import { useState } from 'react'
import { Editor } from './components/Editor'
import { Layout } from './components/Layout'
import { Outline } from './components/Outline'

function App() {
  const [rootPath, setRootPath] = useState<string | undefined>(undefined);
  const [fileToOpen, setFileToOpen] = useState<string | null>(null);
  const [currentFile, setCurrentFile] = useState<string | null>(null);

  const handleOpenFolder = async () => {
    const result = await window.ipcRenderer.invoke('folder-open');
    if (result) {
      setRootPath(result);
    }
  };

  return (
    <Layout
      rootPath={rootPath}
      onFileSelect={(path) => setFileToOpen(path)}
      panel={<Outline currentFile={currentFile} />}
    >
      <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
        {!rootPath && (
          <div style={{ padding: '10px', background: '#333', color: '#fff' }}>
            <button onClick={handleOpenFolder}>Open Folder</button>
          </div>
        )}
        <div style={{ flex: 1 }}>
          <Editor
            fileToOpen={fileToOpen}
            onFileLoaded={(path) => setCurrentFile(path)}
          />
        </div>
      </div>
    </Layout>
  )
}

export default App
