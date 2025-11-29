import { useState, useEffect, useRef, useCallback } from 'react'
import { toast } from 'sonner'
import { EditorPane, EditorPaneHandle } from './components/EditorPane'
import { Layout } from './components/Layout'
import { Outline } from './components/Outline'
import { ExportStatus } from './components/StatusBar'
import { initDebugMonaco } from './debug-monaco'

initDebugMonaco();

function App() {
  const [rootPath, setRootPath] = useState<string | undefined>(undefined);
  const [currentFile, setCurrentFile] = useState<string | null>(null);
  const [cursorLine, setCursorLine] = useState<number>(0);
  const [exportStatus, setExportStatus] = useState<ExportStatus>({ isExporting: false, format: null });
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

  const handleFormat = useCallback(async () => {
    await editorPaneRef.current?.format();
  }, []);

  const handleFind = useCallback(() => {
    editorPaneRef.current?.find();
  }, []);

  const handleReplace = useCallback(() => {
    editorPaneRef.current?.replace();
  }, []);

  const handleShareWhatsApp = useCallback(async () => {
    const editor = editorPaneRef.current?.getEditor();
    if (!editor) {
      toast.error('No document to share');
      return;
    }
    const content = editor.getValue();
    if (!content.trim()) {
      toast.error('Document is empty');
      return;
    }
    await window.ipcRenderer.shareWhatsApp(content);
  }, []);

  /**
   * Converts the current non-lex file to lex format.
   *
   * Uses the lex CLI to convert markdown/html/txt to lex format,
   * then opens the new .lex file.
   */
  const handleConvertToLex = useCallback(async () => {
    const filePath = editorPaneRef.current?.getCurrentFile();
    if (!filePath) {
      toast.error('No file open to convert');
      return;
    }

    // Save before conversion - CLI uses the file on disk
    await editorPaneRef.current?.save();

    setExportStatus({ isExporting: true, format: 'lex' });

    try {
      const outputPath = await window.ipcRenderer.fileExport(filePath, 'lex');
      const fileName = outputPath.split('/').pop() || outputPath;
      toast.success(`Converted to ${fileName}`);
      // Open the newly created lex file
      await editorPaneRef.current?.openFile(outputPath);
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Conversion failed';
      toast.error(message);
    } finally {
      setExportStatus({ isExporting: false, format: null });
    }
  }, []);

  /**
   * Exports the current file to the specified format.
   *
   * Export flow:
   * 1. Save the current editor content to disk (export uses the file on disk)
   * 2. Show spinner in status bar
   * 3. Call the lex CLI to convert the file
   * 4. Show success/error toast
   */
  const handleExport = useCallback(async (format: string) => {
    const filePath = editorPaneRef.current?.getCurrentFile();
    if (!filePath) {
      toast.error('No file open to export');
      return;
    }

    // Save before export - export uses the file on disk
    await editorPaneRef.current?.save();

    setExportStatus({ isExporting: true, format });

    try {
      const outputPath = await window.ipcRenderer.fileExport(filePath, format);
      const fileName = outputPath.split('/').pop() || outputPath;
      toast.success(`Exported to ${fileName}`);
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Export failed';
      toast.error(message);
    } finally {
      setExportStatus({ isExporting: false, format: null });
    }
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
        exportStatus={exportStatus}
      />
    </Layout>
  )
}

export default App
