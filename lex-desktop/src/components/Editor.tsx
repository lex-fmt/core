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

export function Editor() {
    const containerRef = useRef<HTMLDivElement>(null);
    const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
    const [currentFile, setCurrentFile] = useState<string | null>(null);

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

        // Define Theme
        monaco.editor.defineTheme('lex-dark', {
            base: 'vs-dark',
            inherit: true,
            rules: [
                { token: 'DocumentTitle', foreground: 'FF0000', fontStyle: 'bold' }, // Example colors
                { token: 'SessionTitleText', foreground: '00FF00', fontStyle: 'bold' },
                { token: 'AnnotationLabel', foreground: '888888' },
                // Add more mappings based on VSCode theme
            ],
            colors: {}
        });

        // Initialize Editor
        const editor = monaco.editor.create(containerRef.current, {
            value: '# Hello Lex\n\nThis is a test document.',
            language: 'lex',
            theme: 'lex-dark',
            automaticLayout: true,
            minimap: { enabled: false }
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

    return (
        <div style={{ display: 'flex', flexDirection: 'column', height: '100vh' }}>
            <div style={{ padding: '5px', background: '#333', display: 'flex', gap: '10px' }}>
                <button onClick={handleOpen}>Open File</button>
                <button onClick={handleSave} disabled={!currentFile}>Save</button>
                <span style={{ color: '#fff', alignSelf: 'center' }}>{currentFile || 'Untitled'}</span>
            </div>
            <div ref={containerRef} style={{ flex: 1 }} />
        </div>
    );
}
