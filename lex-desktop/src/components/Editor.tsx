import { useRef, useEffect, useState, forwardRef, useImperativeHandle } from 'react';
import * as monaco from 'monaco-editor';
import 'monaco-editor/esm/vs/editor/editor.main'; // Ensure full editor is loaded
import { lspClient } from '../lsp/client';

// Lex Monochrome Theme Colors (from editors/vscode/src/theme.ts)
const LIGHT_COLORS = {
    normal: '#000000',
    muted: '#808080',
    faint: '#b3b3b3',
    faintest: '#cacaca',
    background: '#ffffff',
};

const DARK_COLORS = {
    normal: '#e0e0e0',
    muted: '#888888',
    faint: '#666666',
    faintest: '#555555',
    background: '#1e1e1e',
};

type ThemeMode = 'dark' | 'light';

function getThemeColors(mode: ThemeMode) {
    return mode === 'dark' ? DARK_COLORS : LIGHT_COLORS;
}

function defineMonacoTheme(themeName: string, mode: ThemeMode) {
    const colors = getThemeColors(mode);
    const baseTheme = mode === 'dark' ? 'vs-dark' : 'vs';

    monaco.editor.defineTheme(themeName, {
        base: baseTheme,
        inherit: true,
        rules: [
            // Normal (full contrast) - primary content
            { token: 'SessionTitleText', foreground: colors.normal.replace('#', ''), fontStyle: 'bold' },
            { token: 'DefinitionSubject', foreground: colors.normal.replace('#', ''), fontStyle: 'italic' },
            { token: 'DefinitionContent', foreground: colors.normal.replace('#', '') },
            { token: 'InlineStrong', foreground: colors.normal.replace('#', ''), fontStyle: 'bold' },
            { token: 'InlineEmphasis', foreground: colors.normal.replace('#', ''), fontStyle: 'italic' },
            { token: 'InlineCode', foreground: colors.normal.replace('#', '') },
            { token: 'InlineMath', foreground: colors.normal.replace('#', ''), fontStyle: 'italic' },
            { token: 'VerbatimContent', foreground: colors.normal.replace('#', '') },
            { token: 'ListItemText', foreground: colors.normal.replace('#', '') },

            // Muted (medium gray) - structural elements
            { token: 'DocumentTitle', foreground: colors.muted.replace('#', ''), fontStyle: 'bold' },
            { token: 'SessionMarker', foreground: colors.muted.replace('#', ''), fontStyle: 'italic' },
            { token: 'ListMarker', foreground: colors.muted.replace('#', ''), fontStyle: 'italic' },
            { token: 'Reference', foreground: colors.muted.replace('#', ''), fontStyle: 'underline' },
            { token: 'ReferenceCitation', foreground: colors.muted.replace('#', ''), fontStyle: 'underline' },
            { token: 'ReferenceFootnote', foreground: colors.muted.replace('#', ''), fontStyle: 'underline' },

            // Faint (light gray) - meta-information
            { token: 'AnnotationLabel', foreground: colors.faint.replace('#', '') },
            { token: 'AnnotationParameter', foreground: colors.faint.replace('#', '') },
            { token: 'AnnotationContent', foreground: colors.faint.replace('#', '') },
            { token: 'VerbatimSubject', foreground: colors.faint.replace('#', '') },
            { token: 'VerbatimLanguage', foreground: colors.faint.replace('#', '') },
            { token: 'VerbatimAttribute', foreground: colors.faint.replace('#', '') },

            // Faintest (barely visible) - inline markers
            { token: 'InlineMarker_strong_start', foreground: colors.faintest.replace('#', ''), fontStyle: 'italic' },
            { token: 'InlineMarker_strong_end', foreground: colors.faintest.replace('#', ''), fontStyle: 'italic' },
            { token: 'InlineMarker_emphasis_start', foreground: colors.faintest.replace('#', ''), fontStyle: 'italic' },
            { token: 'InlineMarker_emphasis_end', foreground: colors.faintest.replace('#', ''), fontStyle: 'italic' },
            { token: 'InlineMarker_code_start', foreground: colors.faintest.replace('#', ''), fontStyle: 'italic' },
            { token: 'InlineMarker_code_end', foreground: colors.faintest.replace('#', ''), fontStyle: 'italic' },
            { token: 'InlineMarker_math_start', foreground: colors.faintest.replace('#', ''), fontStyle: 'italic' },
            { token: 'InlineMarker_math_end', foreground: colors.faintest.replace('#', ''), fontStyle: 'italic' },
            { token: 'InlineMarker_ref_start', foreground: colors.faintest.replace('#', ''), fontStyle: 'italic' },
            { token: 'InlineMarker_ref_end', foreground: colors.faintest.replace('#', ''), fontStyle: 'italic' },
        ],
        colors: {
            'editor.foreground': colors.normal,
            'editor.background': colors.background,
            'editorLineNumber.foreground': colors.faint,
            'editorLineNumber.activeForeground': colors.normal,
        },
    });
}

interface EditorProps {
    fileToOpen?: string | null;
    onFileLoaded?: (path: string) => void;
}

export interface EditorHandle {
    openFile: () => Promise<void>;
    save: () => Promise<void>;
    getCurrentFile: () => string | null;
    getEditor: () => monaco.editor.IStandaloneCodeEditor | null;
}

export const Editor = forwardRef<EditorHandle, EditorProps>(function Editor({ fileToOpen, onFileLoaded }, ref) {
    const containerRef = useRef<HTMLDivElement>(null);
    const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
    const [currentFile, setCurrentFile] = useState<string | null>(null);

    // Expose methods to parent
    useImperativeHandle(ref, () => ({
        openFile: handleOpen,
        save: handleSave,
        getCurrentFile: () => currentFile,
        getEditor: () => editorRef.current,
    }));

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

        // 3. Define theme with initial mode (will be updated when we get OS theme)
        // Start with dark mode as default, will be updated after we get the actual OS theme
        defineMonacoTheme(DEBUG_THEME, 'dark');

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

        // 4. Initialize theme based on OS preference and listen for changes
        const applyTheme = (mode: ThemeMode) => {
            console.log('[Editor] Applying theme:', mode);
            defineMonacoTheme(DEBUG_THEME, mode);
            monaco.editor.setTheme(DEBUG_THEME);
        };

        // Get initial theme from OS
        window.ipcRenderer.getNativeTheme().then((mode: ThemeMode) => {
            console.log('[Editor] Initial OS theme:', mode);
            applyTheme(mode);
        });

        // Listen for OS theme changes
        const unsubscribeTheme = window.ipcRenderer.onNativeThemeChanged((mode: ThemeMode) => {
            console.log('[Editor] OS theme changed to:', mode);
            applyTheme(mode);
        });

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
            unsubscribeTheme();
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

    return (
        <div ref={containerRef} style={{ width: '100%', height: '100%', overflow: 'hidden' }} />
    );
});
