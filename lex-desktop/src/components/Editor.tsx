import { useRef, useEffect, useState } from 'react';
import * as monaco from 'monaco-editor';
import 'monaco-editor/esm/vs/editor/editor.main'; // Ensure full editor is loaded
import { lspClient } from '../lsp/client';


interface EditorProps {
    fileToOpen?: string | null;
    onFileLoaded?: (path: string) => void;
}

export function Editor({ fileToOpen, onFileLoaded }: EditorProps) {
    const containerRef = useRef<HTMLDivElement>(null);
    const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
    const [currentFile, setCurrentFile] = useState<string | null>(null);

    // Handle fileToOpen prop change
    useEffect(() => {
        if (fileToOpen && fileToOpen !== currentFile) {
            handleOpenFile(fileToOpen);
        }
    }, [fileToOpen, currentFile]); // Added currentFile to dependency array

    const handleOpenFile = async (path: string) => {
        const content = await window.ipcRenderer.invoke('file-read', path);
        if (content !== null && editorRef.current) {
            setCurrentFile(path);
            if (onFileLoaded) onFileLoaded(path);

            // Dispose old model if it exists and is not the initial one (optional optimization)
            // const oldModel = editorRef.current.getModel();
            // if (oldModel) {
            //     oldModel.dispose();
            // }

            const uri = monaco.Uri.file(path);
            const model = monaco.editor.createModel(content, 'lex', uri);
            editorRef.current.setModel(model);

            lspClient.sendNotification('textDocument/didOpen', {
                textDocument: {
                    uri: uri.toString(),
                    languageId: 'lex',
                    version: 1,
                    text: content
                }
            });

            // Handle changes for the new model
            model.onDidChangeContent((_e) => {
                lspClient.sendNotification('textDocument/didChange', {
                    textDocument: {
                        uri: uri.toString(),
                        version: 2, // Should increment
                    },
                    contentChanges: [{ text: model.getValue() }]
                });
            });

        }
    };

    useEffect(() => {
        if (!containerRef.current) return;

        // ISOLATED SETUP MOVED TO debug-monaco.ts
        const DEBUG_LANG = 'lex';
        const DEBUG_THEME = 'lex-theme';

        // Create Model with DEBUG_LANG
        const uri = monaco.Uri.parse('file:///test.lex');
        let lspModel = monaco.editor.getModel(uri);
        if (lspModel) {
            lspModel.dispose(); // Dispose existing model to force recreation
        }
        lspModel = monaco.editor.createModel(
            '# Hello Lex\n\nThis is a test document.\nSession Title',
            DEBUG_LANG,
            uri
        );

        // Force language set
        monaco.editor.setModelLanguage(lspModel, DEBUG_LANG);

        const editor = monaco.editor.create(containerRef.current, {
            model: lspModel,
            theme: DEBUG_THEME,
            automaticLayout: true,
            minimap: { enabled: false },
            fontSize: 14,
            lineNumbers: 'on',
            scrollBeyondLastLine: false,
            wordWrap: 'on',
            padding: { top: 10, bottom: 10 },
            fontFamily: 'JetBrains Mono, monospace',
            // @ts-ignore
            semanticHighlighting: { enabled: true },
        });
        editorRef.current = editor;

        // @ts-ignore
        window.monaco = monaco;
        // @ts-ignore
        window.editor = editor;

        console.log('Editor created. Model language:', editor.getModel()?.getLanguageId());
        // @ts-ignore
        editor.updateOptions({ semanticHighlighting: { enabled: true } });

        // Debug: Log token info on click
        editor.onMouseDown((e) => {
            const position = e.target.position;
            if (position) {
                const model = editor.getModel();
                if (model) {
                    const word = model.getWordAtPosition(position);
                    const offset = model.getOffsetAt(position);
                    console.log('--- Click Debug ---');
                    console.log('Position:', position);
                    console.log('Word:', word);
                    console.log('Offset:', offset);

                    // Log DOM element if available
                    console.log('DOM Target:', e.target.element);

                    // Attempt to get token info (Monarch)
                    // @ts-ignore
                    const tokens = monaco.editor.tokenize(model.getValue(), model.getLanguageId());
                    if (tokens && tokens[position.lineNumber - 1]) {
                        const lineTokens = tokens[position.lineNumber - 1];
                        const token = lineTokens.find(t => t.offset <= position.column - 1); // approximate
                        console.log('Monarch Token (Line):', token);
                    }

                    // Log full line tokens for context
                    console.log('All Line Tokens:', tokens[position.lineNumber - 1]);
                }
            }
        });

        if (lspModel) {
            // Initialize LSP
            const uriStr = lspModel.uri.toString();
            lspClient.initialize().then(() => {
                // Open Document
                lspClient.sendNotification('textDocument/didOpen', {
                    textDocument: {
                        uri: uriStr,
                        languageId: 'lex',
                        version: 1,
                        text: lspModel.getValue()
                    }
                });
            }).catch(err => {
                console.error('LSP initialization failed in Editor:', err);
            });

            // Handle Changes
            lspModel.onDidChangeContent((_e) => {
                lspClient.sendNotification('textDocument/didChange', {
                    textDocument: {
                        uri: lspModel.uri.toString(),
                        version: 2, // Should increment
                    },
                    contentChanges: [{ text: lspModel.getValue() }]
                });
            });
        }

        return () => {
            editor.dispose();
            if (lspModel) {
                lspModel.dispose();
            }
        };
    }, []);

    const handleOpen = async () => {
        const result = await window.ipcRenderer.fileOpen();
        if (result && editorRef.current) {
            const { filePath, content } = result;
            setCurrentFile(filePath);
            if (onFileLoaded) onFileLoaded(filePath);
            const model = editorRef.current.getModel();
            if (model) {
                model.setValue(content);
                // Notify LSP of new file
                // Note: In a real app we should handle closing the old file
                const uri = 'file://' + filePath;
                // Update model URI (Monaco doesn't easily allow changing model URI, usually we create a new model)
                // For simplicity, we just send didOpen with the new URI
                lspClient.sendNotification('textDocument/didOpen', {
                    textDocument: {
                        uri: uri,
                        languageId: 'lex',
                        version: 1,
                        text: content
                    }
                });
            }
        }
    };

    const handleSave = async () => {
        if (currentFile && editorRef.current) {
            await window.ipcRenderer.fileSave(currentFile, editorRef.current.getValue());
        }
    };

    const handleMockDiagnostics = () => {
        if (editorRef.current) {
            const model = editorRef.current.getModel();
            if (model) {
                const markers = [{
                    severity: monaco.MarkerSeverity.Error,
                    startLineNumber: 1,
                    startColumn: 1,
                    endLineNumber: 1,
                    endColumn: 5,
                    message: 'Mock Error: Invalid syntax',
                    source: 'Mock LSP'
                }];
                monaco.editor.setModelLanguage(model, 'lex'); // Ensure language is set for markers
                monaco.editor.setModelMarkers(model, 'lex', markers);
            }
        }
    };

    const [lspStatus, setLspStatus] = useState<any>('Initializing');

    useEffect(() => {
        lspClient.onStatusChange((status) => {
            setLspStatus(status);
        });
    }, []);

    const getStatusColor = () => {
        switch (lspStatus) {
            case 'Ready': return '#4caf50'; // Green
            case 'Error': return '#f44336'; // Red
            default: return '#ff9800'; // Orange
        }
    };

    return (
        <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
            <div style={{ padding: '5px', background: '#333', display: 'flex', gap: '10px', alignItems: 'center' }}>
                <button onClick={handleOpen}>Open File</button>
                <button onClick={handleSave} disabled={!currentFile}>Save</button>
                <button onClick={handleMockDiagnostics} style={{ marginLeft: 'auto', background: '#555' }}>Mock Diagnostics</button>

                {/* LSP Status Indicator */}
                <div style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: '5px',
                    fontSize: '12px',
                    color: '#ccc',
                    borderLeft: '1px solid #555',
                    paddingLeft: '10px',
                    marginLeft: '10px'
                }}>
                    <div style={{
                        width: '8px',
                        height: '8px',
                        borderRadius: '50%',
                        backgroundColor: getStatusColor()
                    }} />
                    <span>LSP: {lspStatus}</span>
                </div>

                <span style={{ color: '#fff', marginLeft: '10px' }}>{currentFile || 'Untitled'}</span>
            </div>
            <div ref={containerRef} style={{ flex: 1, overflow: 'hidden', position: 'relative' }} />
        </div>
    );
}
