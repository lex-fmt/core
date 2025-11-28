import { useRef, useEffect, useState } from 'react';
import * as monaco from 'monaco-editor';
import { lspClient } from '../lsp/client';

const TOKEN_TYPES = [
    "DocumentTitle", "SessionMarker", "SessionTitleText", "DefinitionSubject",
    "DefinitionContent", "ListMarker", "ListItemText", "AnnotationLabel",
    "AnnotationParameter", "AnnotationContent", "InlineStrong", "InlineEmphasis",
    "InlineCode", "InlineMath", "Reference", "ReferenceCitation", "ReferenceFootnote",
    "VerbatimSubject", "VerbatimLanguage", "VerbatimAttribute", "VerbatimContent",
    "InlineMarker_strong_start", "InlineMarker_strong_end", "InlineMarker_emphasis_start",
    "InlineMarker_emphasis_end", "InlineMarker_code_start", "InlineMarker_code_end",
    "InlineMarker_math_start", "InlineMarker_math_end", "InlineMarker_ref_start",
    "InlineMarker_ref_end"
];

const TOKEN_MODIFIERS: string[] = [];

const LEGEND = {
    tokenTypes: TOKEN_TYPES,
    tokenModifiers: TOKEN_MODIFIERS
};

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
        // We need a way to read file content by path.
        // We can reuse fileOpen logic but we need a new IPC or just use fs in main.
        // Wait, we have fileOpen which opens a dialog. We need fileRead(path).
        // Let's add fileRead to IPC.
        // For now, let's assume we can read it.
        // Actually, we can just use the existing fileOpen logic but passing a path?
        // No, fileOpen opens a dialog.

        // Let's assume we implement 'file-read' IPC.
        const content = await window.ipcRenderer.invoke('file-read', path);
        if (content !== null && editorRef.current) {
            setCurrentFile(path);
            if (onFileLoaded) onFileLoaded(path);
            const model = editorRef.current.getModel();
            if (model) {
                model.setValue(content);
                const uri = 'file://' + path;
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

    useEffect(() => {
        if (!containerRef.current) return;

        // Register Language
        monaco.languages.register({ id: 'lex' });

        // Register Semantic Tokens Provider
        monaco.languages.registerDocumentSemanticTokensProvider('lex', {
            getLegend: function () {
                return LEGEND;
            },
            provideDocumentSemanticTokens: async function (model, _lastResultId, _token) {
                const response = await lspClient.sendRequest('textDocument/semanticTokens/full', {
                    textDocument: { uri: model.uri.toString() }
                });

                if (response && response.data) {
                    return {
                        data: new Uint32Array(response.data),
                        resultId: response.resultId
                    };
                }
                return null;
            },
            releaseDocumentSemanticTokens: function (_resultId) { }
        });

        // Register Formatting Provider
        monaco.languages.registerDocumentFormattingEditProvider('lex', {
            provideDocumentFormattingEdits: async function (model, _options, _token) {
                const response = await lspClient.sendRequest('textDocument/formatting', {
                    textDocument: { uri: model.uri.toString() },
                    options: { tabSize: 2, insertSpaces: true } // Default options
                });

                if (response) {
                    return response.map((edit: any) => ({
                        range: {
                            startLineNumber: edit.range.start.line + 1,
                            startColumn: edit.range.start.character + 1,
                            endLineNumber: edit.range.end.line + 1,
                            endColumn: edit.range.end.character + 1
                        },
                        text: edit.newText
                    }));
                }
                return [];
            }
        });

        // Register Hover Provider
        monaco.languages.registerHoverProvider('lex', {
            provideHover: async function (model, position) {
                const response = await lspClient.sendRequest('textDocument/hover', {
                    textDocument: { uri: model.uri.toString() },
                    position: { line: position.lineNumber - 1, character: position.column - 1 }
                });

                if (response && response.contents) {
                    return {
                        range: new monaco.Range(
                            position.lineNumber, position.column,
                            position.lineNumber, position.column
                        ),
                        contents: Array.isArray(response.contents)
                            ? response.contents.map((c: any) => ({ value: c.value || c }))
                            : [{ value: response.contents.value || response.contents }]
                    };
                }
                return null;
            }
        });

        // Register Definition Provider
        monaco.languages.registerDefinitionProvider('lex', {
            provideDefinition: async function (model, position) {
                const response = await lspClient.sendRequest('textDocument/definition', {
                    textDocument: { uri: model.uri.toString() },
                    position: { line: position.lineNumber - 1, character: position.column - 1 }
                });

                if (response) {
                    const locations = Array.isArray(response) ? response : [response];
                    return locations.map((loc: any) => ({
                        uri: monaco.Uri.parse(loc.uri),
                        range: {
                            startLineNumber: loc.range.start.line + 1,
                            startColumn: loc.range.start.character + 1,
                            endLineNumber: loc.range.end.line + 1,
                            endColumn: loc.range.end.character + 1
                        }
                    }));
                }
                return null;
            }
        });

        // Listen for Diagnostics
        lspClient.onNotification('textDocument/publishDiagnostics', (params: any) => {
            // params: { uri: string, diagnostics: Diagnostic[] }
            // We need to match the URI to the current model
            if (editorRef.current) {
                const model = editorRef.current.getModel();
                if (model && model.uri.toString() === params.uri) {
                    const markers = params.diagnostics.map((diag: any) => ({
                        severity: diag.severity === 1 ? monaco.MarkerSeverity.Error : monaco.MarkerSeverity.Warning,
                        startLineNumber: diag.range.start.line + 1,
                        startColumn: diag.range.start.character + 1,
                        endLineNumber: diag.range.end.line + 1,
                        endColumn: diag.range.end.character + 1,
                        message: diag.message,
                        source: 'Lex LSP'
                    }));
                    monaco.editor.setModelMarkers(model, 'lex', markers);
                }
            }
        });

        // Define Theme (Lex Monochrome)
        monaco.editor.defineTheme('lex-dark', {
            base: 'vs-dark',
            inherit: true,
            rules: [
                // Normal (#e0e0e0)
                { token: 'SessionTitleText', foreground: 'e0e0e0', fontStyle: 'bold' },
                { token: 'DefinitionSubject', foreground: 'e0e0e0', fontStyle: 'italic' },
                { token: 'DefinitionContent', foreground: 'e0e0e0' },
                { token: 'InlineStrong', foreground: 'e0e0e0', fontStyle: 'bold' },
                { token: 'InlineEmphasis', foreground: 'e0e0e0', fontStyle: 'italic' },
                { token: 'InlineCode', foreground: 'e0e0e0' },
                { token: 'InlineMath', foreground: 'e0e0e0', fontStyle: 'italic' },
                { token: 'VerbatimContent', foreground: 'e0e0e0' },
                { token: 'ListItemText', foreground: 'e0e0e0' },

                // Muted (#888888)
                { token: 'DocumentTitle', foreground: '888888', fontStyle: 'bold' },
                { token: 'SessionMarker', foreground: '888888', fontStyle: 'italic' },
                { token: 'ListMarker', foreground: '888888', fontStyle: 'italic' },
                { token: 'Reference', foreground: '888888', fontStyle: 'underline' },
                { token: 'ReferenceCitation', foreground: '888888', fontStyle: 'underline' },
                { token: 'ReferenceFootnote', foreground: '888888', fontStyle: 'underline' },

                // Faint (#666666)
                { token: 'AnnotationLabel', foreground: '666666' },
                { token: 'AnnotationParameter', foreground: '666666' },
                { token: 'AnnotationContent', foreground: '666666' },
                { token: 'VerbatimSubject', foreground: '666666' },
                { token: 'VerbatimLanguage', foreground: '666666' },
                { token: 'VerbatimAttribute', foreground: '666666' },

                // Faintest (#555555)
                { token: 'InlineMarker_strong_start', foreground: '555555', fontStyle: 'italic' },
                { token: 'InlineMarker_strong_end', foreground: '555555', fontStyle: 'italic' },
                { token: 'InlineMarker_emphasis_start', foreground: '555555', fontStyle: 'italic' },
                { token: 'InlineMarker_emphasis_end', foreground: '555555', fontStyle: 'italic' },
                { token: 'InlineMarker_code_start', foreground: '555555', fontStyle: 'italic' },
                { token: 'InlineMarker_code_end', foreground: '555555', fontStyle: 'italic' },
                { token: 'InlineMarker_math_start', foreground: '555555', fontStyle: 'italic' },
                { token: 'InlineMarker_math_end', foreground: '555555', fontStyle: 'italic' },
                { token: 'InlineMarker_ref_start', foreground: '555555', fontStyle: 'italic' },
                { token: 'InlineMarker_ref_end', foreground: '555555', fontStyle: 'italic' },
            ],
            colors: {
                'editor.background': '#1e1e1e',
                'editor.foreground': '#cccccc',
                'editorLineNumber.foreground': '#858585',
                'editor.selectionBackground': '#264f78',
                'editor.inactiveSelectionBackground': '#3a3d41',
            }
        });

        // Initialize Editor
        const editor = monaco.editor.create(containerRef.current, {
            value: '# Hello Lex\n\nThis is a test document.',
            language: 'lex',
            theme: 'lex-dark',
            automaticLayout: true,
            minimap: { enabled: false },
            fontFamily: 'Menlo, Monaco, "Courier New", monospace',
            fontSize: 14,
            lineHeight: 22,
        });
        editorRef.current = editor;

        const model = editor.getModel();
        if (model) {
            // Initialize LSP
            const uri = model.uri.toString();
            lspClient.sendRequest('initialize', {
                processId: null,
                rootUri: null,
                capabilities: {}
            }).then(() => {
                lspClient.sendNotification('initialized', {});

                // Open Document
                lspClient.sendNotification('textDocument/didOpen', {
                    textDocument: {
                        uri: uri,
                        languageId: 'lex',
                        version: 1,
                        text: model.getValue()
                    }
                });
            });

            // Handle Changes
            model.onDidChangeContent((_e) => {
                lspClient.sendNotification('textDocument/didChange', {
                    textDocument: {
                        uri: model.uri.toString(),
                        version: 2, // Should increment
                    },
                    contentChanges: [{ text: model.getValue() }]
                });
            });
        }

        return () => {
            editor.dispose();
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
                monaco.editor.setModelMarkers(model, 'lex', markers);
            }
        }
    };

    return (
        <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
            <div style={{ padding: '5px', background: '#333', display: 'flex', gap: '10px' }}>
                <button onClick={handleOpen}>Open File</button>
                <button onClick={handleSave} disabled={!currentFile}>Save</button>
                <button onClick={handleMockDiagnostics} style={{ marginLeft: 'auto', background: '#555' }}>Mock Diagnostics</button>
                <span style={{ color: '#fff', alignSelf: 'center' }}>{currentFile || 'Untitled'}</span>
            </div>
            <div ref={containerRef} style={{ flex: 1 }} />
        </div>
    );
}
