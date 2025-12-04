import { useRef, useEffect, useState, forwardRef, useImperativeHandle, useCallback } from 'react';
import * as monaco from 'monaco-editor';
import 'monaco-editor';
import { initializeMonaco, applyTheme, type ThemeMode } from '@/monaco';
import { getOrCreateModel, disposeModel } from '@/monaco/models';
import { ensureLspInitialized } from '@/lsp/init';
import { initVimMode } from 'monaco-vim';
import { useSettings } from '@/contexts/SettingsContext';
import { lspClient } from '@/lsp/client';
import { buildFormattingOptions, notifyLexTest } from '@/lsp/providers/formatting';
import type { LspTextEdit } from '@/lsp/types';
import { dispatchFileTreeRefresh } from '@/lib/events';

initializeMonaco();

interface EditorProps {
  fileToOpen?: string | null;
  onFileLoaded?: (path: string) => void;
  vimStatusNode?: HTMLDivElement | null;
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

export const Editor = forwardRef<EditorHandle, EditorProps>(function Editor({ fileToOpen, onFileLoaded, vimStatusNode }, ref) {
  const containerRef = useRef<HTMLDivElement>(null);
  const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  const vimModeRef = useRef<any>(null);
  const attachedStatusNodeRef = useRef<HTMLDivElement | null>(null);
  const [currentFile, setCurrentFile] = useState<string | null>(null);
  const { settings } = useSettings();
  console.log('[Editor] Rendered');

  const formatWithLsp = useCallback(async () => {
    const editor = editorRef.current;
    if (!editor) return;
    const model = editor.getModel();
    if (!model || model.getLanguageId() !== 'lex') {
      return;
    }

    const modelOptions = model.getOptions();
    const tabSize = modelOptions.tabSize ?? 4;
    const insertSpaces = modelOptions.insertSpaces ?? true;
    const params = {
      textDocument: { uri: model.uri.toString() },
      options: buildFormattingOptions(tabSize, insertSpaces),
    };
    notifyLexTest({ type: 'document', params });

    let edits: LspTextEdit[] | null = null;
    try {
      edits = await lspClient.sendRequest('textDocument/formatting', params);
    } catch (error) {
      console.error('[LSP] Formatting failed:', error);
      return;
    }

    if (!edits || edits.length === 0) {
      return;
    }

    const monacoEdits = edits.map(edit => ({
      range: new monaco.Range(
        edit.range.start.line + 1,
        edit.range.start.character + 1,
        edit.range.end.line + 1,
        edit.range.end.character + 1,
      ),
      text: edit.newText,
    }));

    editor.pushUndoStop();
    editor.executeEdits('lex-format', monacoEdits);
    editor.pushUndoStop();
  }, []);

  const switchToFile = useCallback(async (path: string) => {
    if (!editorRef.current) return;
    let model = monaco.editor.getModel(monaco.Uri.file(path));

    if (!model || model.isDisposed()) {
      const content = await window.ipcRenderer.invoke('file-read', path) as string | null;
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
    format: formatWithLsp,
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
    const editor = editorRef.current;
    if (!editor) {
      return;
    }

    if (!settings.editor.vimMode || !vimStatusNode) {
      if (vimModeRef.current) {
        vimModeRef.current.dispose();
        vimModeRef.current = null;
      }
      if (attachedStatusNodeRef.current) {
        attachedStatusNodeRef.current.textContent = '';
        attachedStatusNodeRef.current = null;
      }
      return;
    }

    if (vimModeRef.current && attachedStatusNodeRef.current === vimStatusNode) {
      return;
    }

    if (vimModeRef.current) {
      vimModeRef.current.dispose();
      vimModeRef.current = null;
    }

    vimStatusNode.textContent = '';
    vimModeRef.current = initVimMode(editor, vimStatusNode);
    attachedStatusNodeRef.current = vimStatusNode;
  }, [settings.editor.vimMode, vimStatusNode]);

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
      rulers: settings.editor.showRuler ? [settings.editor.rulerWidth] : [],
    } satisfies monaco.editor.IStandaloneEditorConstructionOptions);
    editorRef.current = editor;
    // Expose monaco to window for E2E testing
    (window as any).monaco = monaco;

    // ... existing code

    const applyThemeFromNative = (mode: ThemeMode) => {
      applyTheme(mode);
    };

    const initialTheme = document.documentElement.getAttribute('data-theme');
    if (initialTheme === 'dark' || initialTheme === 'light') {
      applyThemeFromNative(initialTheme);
    } else {
      void window.ipcRenderer.getNativeTheme().then(applyThemeFromNative);
    }
    const unsubscribeTheme = window.ipcRenderer.onNativeThemeChanged(applyThemeFromNative);

    // Listen for insert commands
    const unsubscribeInsertAsset = window.ipcRenderer.on('menu-insert-asset', () => {
      if (editorRef.current) {
        import('../commands').then(({ insertAssetReference }) => {
          insertAssetReference(editorRef.current!);
        });
      }
    });

    const unsubscribeInsertVerbatim = window.ipcRenderer.on('menu-insert-verbatim', () => {
      if (editorRef.current) {
        import('../commands').then(({ insertVerbatimBlock }) => {
          insertVerbatimBlock(editorRef.current!);
        });
      }
    });

    return () => {
      editor.dispose();
      if (vimModeRef.current) {
        vimModeRef.current.dispose();
        vimModeRef.current = null;
      }
      if (attachedStatusNodeRef.current) {
        attachedStatusNodeRef.current.textContent = '';
        attachedStatusNodeRef.current = null;
      }
      unsubscribeTheme();
      unsubscribeInsertAsset();
      unsubscribeInsertVerbatim();
    };
  }, []);

  // Spell checker effect
  useEffect(() => {
    const editor = editorRef.current;
    if (!editor || !settings.editor.spellCheck) {
      return;
    }

    let isCancelled = false;
    let spellChecker: monaco.IDisposable | null = null;

    const loadSpellChecker = async () => {
      console.log('[SpellCheck] loadSpellChecker called');
      try {
        if (isCancelled) return;

        const lang = settings.editor.spellCheckLanguage;
        const [affResponse, dicResponse] = await Promise.all([
          fetch(`dictionaries/${lang}.aff`),
          fetch(`dictionaries/${lang}.dic`)
        ]);

        if (!affResponse.ok || !dicResponse.ok) {
          console.error(`[SpellCheck] Failed to load dictionary for ${lang}`, affResponse.status, dicResponse.status);
          return;
        }

        const aff = await affResponse.text();
        const dic = await dicResponse.text();

        if (isCancelled) return;

        // Initialize Worker
        console.log('[SpellCheck] Creating worker...');
        const worker = new Worker(new URL('../workers/spellcheck.worker.ts', import.meta.url));
        console.log('[SpellCheck] Worker created', worker);

        worker.postMessage({
          type: 'init',
          payload: { lang, aff, dic }
        });

        worker.onerror = (err) => {
          console.error('[SpellCheck] Worker error:', err);
        };

        let requestId = 0;
        const pendingSuggestions = new Map<number, (suggestions: string[]) => void>();

        worker.onmessage = (e) => {
          const { type, payload } = e.data;

          if (type === 'init_complete') {
            if (payload.success) {
              checkSpelling();
            } else {
              console.error('[SpellCheck] Worker failed to initialize:', payload.error);
            }
          } else if (type === 'check_result') {
            const { misspelled } = payload;
            if (!editor.getModel()) return;
            const model = editor.getModel()!;

            const markers: monaco.editor.IMarkerData[] = [];

            misspelled.forEach((item: { word: string, index: number }) => {
              const start = item.index;
              const end = start + item.word.length;
              const position = model.getPositionAt(start);
              const endPosition = model.getPositionAt(end);

              markers.push({
                startLineNumber: position.lineNumber,
                startColumn: position.column,
                endLineNumber: endPosition.lineNumber,
                endColumn: endPosition.column,
                message: `Misspelled word: ${item.word}`,
                severity: monaco.MarkerSeverity.Warning,
                source: 'spellcheck',
              });
            });

            monaco.editor.setModelMarkers(model, 'spellcheck', markers);
          } else if (type === 'suggest_result') {
            const { id, suggestions } = payload;
            const resolve = pendingSuggestions.get(id);
            if (resolve) {
              resolve(suggestions);
              pendingSuggestions.delete(id);
            }
          } else if (type === 'debug') {
            console.log(`[SpellCheckWorker] ${payload.msg}`, payload.data || '');
          }
        };

        const checkSpelling = () => {
          if (!editor.getModel()) return;
          const model = editor.getModel()!;
          const value = model.getValue();

          const wordRegex = /[a-zA-Z\u00C0-\u00FF]+/g;
          let match;
          const words: { word: string, index: number }[] = [];

          while ((match = wordRegex.exec(value)) !== null) {
            words.push({ word: match[0], index: match.index });
          }

          worker.postMessage({
            type: 'check',
            payload: { words, id: ++requestId }
          });
        };

        // Listen for changes
        const disposable = editor.onDidChangeModelContent(() => {
          // TODO: Add debounce
          checkSpelling();
        });

        // Register Code Action Provider for suggestions
        const codeActionProvider = monaco.languages.registerCodeActionProvider('lex', {
          provideCodeActions: async (model, range) => {
            const actions: monaco.languages.CodeAction[] = [];
            const markers = monaco.editor.getModelMarkers({ resource: model.uri, owner: 'spellcheck' });

            for (const marker of markers) {
              const markerRange = new monaco.Range(marker.startLineNumber, marker.startColumn, marker.endLineNumber, marker.endColumn);

              if (monaco.Range.areIntersectingOrTouching(range, markerRange)) {
                const word = model.getValueInRange(markerRange);

                // Request suggestions from worker
                const suggestions = await new Promise<string[]>((resolve) => {
                  const id = ++requestId;
                  pendingSuggestions.set(id, resolve);
                  worker.postMessage({ type: 'suggest', payload: { word, id } });

                  // Timeout safety
                  setTimeout(() => {
                    if (pendingSuggestions.has(id)) {
                      pendingSuggestions.delete(id);
                      resolve([]);
                    }
                  }, 1000);
                });

                for (const suggestion of suggestions) {
                  actions.push({
                    title: suggestion,
                    kind: 'quickfix',
                    diagnostics: [marker],
                    edit: {
                      edits: [{
                        resource: model.uri,
                        versionId: undefined,
                        textEdit: {
                          range: new monaco.Range(marker.startLineNumber, marker.startColumn, marker.endLineNumber, marker.endColumn),
                          text: suggestion
                        }
                      }]
                    }
                  });
                }
              }
            }

            return {
              actions: actions,
              dispose: () => { }
            };
          }
        });

        // Save disposable to clean up
        spellChecker = {
          dispose: () => {
            disposable.dispose();
            codeActionProvider.dispose();
            worker.terminate();
          }
        };

      } catch (e) {
        console.error('Failed to initialize spell checker', e);
      }
    };

    void loadSpellChecker();

    return () => {
      isCancelled = true;
      if (spellChecker) {
        spellChecker.dispose();
      }
    };
  }, [settings.editor.spellCheck, settings.editor.spellCheckLanguage]);

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
      if (settings.formatter.formatOnSave) {
        await handleFormat();
      }
      await window.ipcRenderer.fileSave(currentFile, editorRef.current.getValue());
      dispatchFileTreeRefresh();
    }
  };

  const handleFormat = formatWithLsp;

  // ... existing code

  return (
    <div className="relative w-full h-full overflow-hidden">
      <div id="debug-log" style={{ display: 'none' }} />
      <div ref={containerRef} className="w-full h-full" />
    </div>
  );
});
