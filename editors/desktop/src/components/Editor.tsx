import { useRef, useEffect, useState, forwardRef, useImperativeHandle, useCallback } from 'react';
import * as monaco from 'monaco-editor';
import 'monaco-editor/esm/vs/editor/editor.main';
import { initializeMonaco, applyTheme, type ThemeMode } from '@/monaco';
import { getOrCreateModel, disposeModel } from '@/monaco/models';
import { ensureLspInitialized } from '@/lsp/init';

initializeMonaco();

interface EditorProps {
  fileToOpen?: string | null;
  onFileLoaded?: (path: string) => void;
}

export interface EditorHandle {
  openFile: () => Promise<void>;
  save: () => Promise<void>;
  format: () => Promise<void>;
  getCurrentFile: () => string | null;
  getEditor: () => monaco.editor.IStandaloneCodeEditor | null;
  switchToFile: (path: string) => Promise<void>;
  closeFile: (path: string) => void;
  find: () => void;
  replace: () => void;
}

export const Editor = forwardRef<EditorHandle, EditorProps>(function Editor({ fileToOpen, onFileLoaded }, ref) {
  const containerRef = useRef<HTMLDivElement>(null);
  const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  const [currentFile, setCurrentFile] = useState<string | null>(null);

  const switchToFile = useCallback(async (path: string) => {
    if (!editorRef.current) return;
    let model = monaco.editor.getModel(monaco.Uri.file(path));

    if (!model || model.isDisposed()) {
      const content = await window.ipcRenderer.invoke('file-read', path);
      if (content === null) return;
      model = getOrCreateModel(path, content);
    }

    editorRef.current.setModel(model);
    setCurrentFile(path);
    onFileLoaded?.(path);
  }, [onFileLoaded]);

  const closeFile = (path: string) => {
    disposeModel(path);
  };

  useImperativeHandle(ref, () => ({
    openFile: handleOpen,
    save: handleSave,
    format: handleFormat,
    getCurrentFile: () => currentFile,
    getEditor: () => editorRef.current,
    switchToFile,
    closeFile,
    find: () => {
      editorRef.current?.trigger('menu', 'actions.find', null);
    },
    replace: () => {
      editorRef.current?.trigger('menu', 'editor.action.startFindReplaceAction', null);
    },
  }));

  useEffect(() => {
    if (fileToOpen) {
      void switchToFile(fileToOpen);
    }
  }, [fileToOpen, switchToFile]);

  useEffect(() => {
    if (!containerRef.current) return;

    ensureLspInitialized();

    const editor = monaco.editor.create(containerRef.current, {
      model: null,
      automaticLayout: true,
      minimap: { enabled: false },
      fontSize: 13,
      lineNumbers: 'on',
      scrollBeyondLastLine: false,
      wordWrap: 'on',
      padding: { top: 10, bottom: 10 },
      fontFamily: 'Geist, -apple-system, BlinkMacSystemFont, sans-serif',
      'semanticHighlighting.enabled': true,
    } satisfies monaco.editor.IStandaloneEditorConstructionOptions);
    editorRef.current = editor;

    // Expose editor for end-to-end tests/debug builds
    const globalWindow = window as typeof window & {
      monaco?: typeof monaco;
      editor?: monaco.editor.IStandaloneCodeEditor | null;
    };
    globalWindow.monaco = monaco;
    globalWindow.editor = editor;

    const applyThemeFromNative = (mode: ThemeMode) => {
      applyTheme(mode);
    };

    window.ipcRenderer.getNativeTheme().then(applyThemeFromNative);
    const unsubscribeTheme = window.ipcRenderer.onNativeThemeChanged(applyThemeFromNative);

    return () => {
      editor.dispose();
      unsubscribeTheme();
    };
  }, []);

  const handleOpen = async () => {
    const result = await window.ipcRenderer.fileOpen();
    if (result && editorRef.current) {
      const { filePath, content } = result;
      const model = getOrCreateModel(filePath, content);
      editorRef.current.setModel(model);
      setCurrentFile(filePath);
      onFileLoaded?.(filePath);
    }
  };

  const handleSave = async () => {
    if (currentFile && editorRef.current) {
      await window.ipcRenderer.fileSave(currentFile, editorRef.current.getValue());
    }
  };

  const handleFormat = async () => {
    if (editorRef.current) {
      await editorRef.current.getAction('editor.action.formatDocument')?.run();
    }
  };

  return (
    <div ref={containerRef} style={{ width: '100%', height: '100%', overflow: 'hidden' }} />
  );
});
