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

        // --- Language & Provider Setup ---
        const DEBUG_LANG = 'lex';
        const DEBUG_THEME = 'lex-theme';

        // 1. Register Language (Idempotent check)
        const languages = monaco.languages.getLanguages();
        const langExists = languages.some(l => l.id === DEBUG_LANG);
        if (!langExists) {
            console.log('[Editor] Registering language:', DEBUG_LANG);
            monaco.languages.register({ id: DEBUG_LANG, extensions: ['.lex'] });
            monaco.languages.setLanguageConfiguration(DEBUG_LANG, {
                comments: { lineComment: '#' }
            });
        }

        // 2. Semantic Tokens Legend (static, matching LSP server)
        const semanticTokensLegend = {
            tokenTypes: [
                "DocumentTitle", "SessionMarker", "SessionTitleText", "DefinitionSubject", "DefinitionContent",
                "ListMarker", "ListItemText", "AnnotationLabel", "AnnotationParameter", "AnnotationContent",
                "InlineStrong", "InlineEmphasis", "InlineCode", "InlineMath", "Reference", "ReferenceCitation",
                "ReferenceFootnote", "VerbatimSubject", "VerbatimLanguage", "VerbatimAttribute", "VerbatimContent",
                "InlineMarker_strong_start", "InlineMarker_strong_end", "InlineMarker_emphasis_start",
                "InlineMarker_emphasis_end", "InlineMarker_code_start", "InlineMarker_code_end",
                "InlineMarker_math_start", "InlineMarker_math_end", "InlineMarker_ref_start", "InlineMarker_ref_end"
            ],
            tokenModifiers: []
        };

        // Track LSP ready state for the provider
        let lspReady = false;

        // Register semantic tokens provider IMMEDIATELY (before editor creation)
        // This ensures Monaco knows about the provider when the model is attached
        console.log('[Editor] Registering semantic tokens provider with legend:', semanticTokensLegend);
        const providerDisposable = monaco.languages.registerDocumentSemanticTokensProvider(DEBUG_LANG, {
            getLegend: function () {
                return semanticTokensLegend;
            },
            provideDocumentSemanticTokens: async function (model, _lastResultId, _token) {
                const uri = model.uri.toString();
                console.log('[Editor] Provider triggered for:', uri, 'LSP Ready:', lspReady);

                if (!lspReady) {
                    console.log('[Editor] LSP not ready yet, returning empty tokens');
                    return { data: new Uint32Array([]) };
                }

                try {
                    const result = await lspClient.sendRequest('textDocument/semanticTokens/full', {
                        textDocument: { uri }
                    });
                    console.log('[Editor] Received tokens:', result?.data?.length, 'tokens');
                    if (result && result.data) {
                        return { data: new Uint32Array(result.data) };
                    }
                } catch (e) {
                    console.error('[Editor] Failed to get tokens:', e);
                }
                return { data: new Uint32Array([]) };
            },
            releaseDocumentSemanticTokens: function () { }
        });

        // 4. Define Theme (Lex Monochrome - Dark Mode)
        // Source: editors/vscode/src/theme.ts
        const COLORS = {
            normal: '#e0e0e0',   // Full contrast
            muted: '#888888',    // Medium gray
            faint: '#666666',    // Light gray
            faintest: '#555555'  // Barely visible
        };

        monaco.editor.defineTheme(DEBUG_THEME, {
            base: 'vs-dark',
            inherit: true,
            rules: [
                // Semantic Token Rules (Matching VSCode Theme)
                { token: 'SessionTitleText', foreground: COLORS.normal, fontStyle: 'bold' },
                { token: 'DefinitionSubject', foreground: COLORS.normal, fontStyle: 'italic' },
                { token: 'DefinitionContent', foreground: COLORS.normal },
                { token: 'InlineStrong', foreground: COLORS.normal, fontStyle: 'bold' },
                { token: 'InlineEmphasis', foreground: COLORS.normal, fontStyle: 'italic' },
                { token: 'InlineCode', foreground: COLORS.normal },
                { token: 'InlineMath', foreground: COLORS.normal, fontStyle: 'italic' },
                { token: 'VerbatimContent', foreground: COLORS.normal },
                { token: 'ListItemText', foreground: COLORS.normal },

                { token: 'DocumentTitle', foreground: COLORS.muted, fontStyle: 'bold' },
                { token: 'SessionMarker', foreground: COLORS.muted, fontStyle: 'italic' },
                { token: 'ListMarker', foreground: COLORS.muted, fontStyle: 'italic' },
                { token: 'Reference', foreground: COLORS.muted, fontStyle: 'underline' },
                { token: 'ReferenceCitation', foreground: COLORS.muted, fontStyle: 'underline' },
                { token: 'ReferenceFootnote', foreground: COLORS.muted, fontStyle: 'underline' },

                { token: 'AnnotationLabel', foreground: COLORS.faint },
                { token: 'AnnotationParameter', foreground: COLORS.faint },
                { token: 'AnnotationContent', foreground: COLORS.faint },
                { token: 'VerbatimSubject', foreground: COLORS.faint },
                { token: 'VerbatimLanguage', foreground: COLORS.faint },
                { token: 'VerbatimAttribute', foreground: COLORS.faint },

                // Faintest (Markers)
                { token: 'InlineMarker_strong_start', foreground: COLORS.faintest, fontStyle: 'italic' },
                { token: 'InlineMarker_strong_end', foreground: COLORS.faintest, fontStyle: 'italic' },
                { token: 'InlineMarker_emphasis_start', foreground: COLORS.faintest, fontStyle: 'italic' },
                { token: 'InlineMarker_emphasis_end', foreground: COLORS.faintest, fontStyle: 'italic' },
                { token: 'InlineMarker_code_start', foreground: COLORS.faintest, fontStyle: 'italic' },
                { token: 'InlineMarker_code_end', foreground: COLORS.faintest, fontStyle: 'italic' },
                { token: 'InlineMarker_math_start', foreground: COLORS.faintest, fontStyle: 'italic' },
                { token: 'InlineMarker_math_end', foreground: COLORS.faintest, fontStyle: 'italic' },
                { token: 'InlineMarker_ref_start', foreground: COLORS.faintest, fontStyle: 'italic' },
                { token: 'InlineMarker_ref_end', foreground: COLORS.faintest, fontStyle: 'italic' },
            ],
            colors: {
                'editor.foreground': COLORS.normal,
                'editor.background': '#1e1e1e', // Standard VS Dark background
            },
            // @ts-ignore
            semanticHighlighting: true
        });

        // 5. Create Model & Editor
        const uri = monaco.Uri.parse('file:///test.lex');
        let lspModel = monaco.editor.getModel(uri);
        if (lspModel) {
            lspModel.dispose();
        }
        lspModel = monaco.editor.createModel(
            '# Hello Lex\n\nThis is a test document.\nSession Title',
            DEBUG_LANG,
            uri
        );

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
            'semanticHighlighting.enabled': true,
        } as any);
        editorRef.current = editor;

        // @ts-ignore
        window.monaco = monaco;
        // @ts-ignore
        window.editor = editor;

        console.log('Editor created. Model language:', editor.getModel()?.getLanguageId());

        // Debug: Log token info on click
        editor.onMouseDown((e) => {
            const position = e.target.position;
            if (position) {
                const model = editor.getModel();
                if (model) {
                    const word = model.getWordAtPosition(position);
                    const offset = model.getOffsetAt(position);

                    console.log('--- Click Debug (Simulated) ---');
                    console.log('Position:', position);
                    console.log('Word:', word);
                    console.log('Offset:', offset);
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

                // Mark LSP as ready so the semantic tokens provider will work
                lspReady = true;
                console.log('[Editor] LSP ready, triggering semantic tokens refresh');

                // Force Monaco to re-query semantic tokens by simulating a model change
                // We do this by adding a space and removing it, which triggers the provider
                const currentValue = lspModel.getValue();
                lspModel.setValue(currentValue + ' ');
                lspModel.setValue(currentValue);

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
            providerDisposable.dispose();
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
