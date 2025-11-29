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

// Model cache - stores models by file path
const modelCache = new Map<string, monaco.editor.ITextModel>();

function getOrCreateModel(path: string, content: string): monaco.editor.ITextModel {
    const cached = modelCache.get(path);
    if (cached && !cached.isDisposed()) {
        return cached;
    }

    const uri = monaco.Uri.file(path);
    // Check if Monaco already has this model
    let model = monaco.editor.getModel(uri);
    if (model) {
        modelCache.set(path, model);
        return model;
    }

    model = monaco.editor.createModel(content, 'lex', uri);
    modelCache.set(path, model);

    // Notify LSP
    lspClient.sendNotification('textDocument/didOpen', {
        textDocument: {
            uri: uri.toString(),
            languageId: 'lex',
            version: 1,
            text: content
        }
    });

    // Handle changes for LSP
    model.onDidChangeContent(() => {
        lspClient.sendNotification('textDocument/didChange', {
            textDocument: {
                uri: uri.toString(),
                version: 2, // Should increment properly
            },
            contentChanges: [{ text: model!.getValue() }]
        });
    });

    return model;
}

function disposeModel(path: string) {
    const model = modelCache.get(path);
    if (model && !model.isDisposed()) {
        lspClient.sendNotification('textDocument/didClose', {
            textDocument: { uri: model.uri.toString() }
        });
        model.dispose();
    }
    modelCache.delete(path);
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
    switchToFile: (path: string) => Promise<void>;
    closeFile: (path: string) => void;
}

export const Editor = forwardRef<EditorHandle, EditorProps>(function Editor({ fileToOpen, onFileLoaded }, ref) {
    const containerRef = useRef<HTMLDivElement>(null);
    const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
    const [currentFile, setCurrentFile] = useState<string | null>(null);

    // Switch to a file - loads from disk if new, otherwise uses cached model
    const switchToFile = async (path: string) => {
        if (!editorRef.current) return;

        let model = modelCache.get(path);

        if (!model || model.isDisposed()) {
            // Load from disk
            const content = await window.ipcRenderer.invoke('file-read', path);
            if (content === null) return;
            model = getOrCreateModel(path, content);
        }

        editorRef.current.setModel(model);
        setCurrentFile(path);
        onFileLoaded?.(path);
    };

    const closeFile = (path: string) => {
        disposeModel(path);
    };

    // Expose methods to parent
    useImperativeHandle(ref, () => ({
        openFile: handleOpen,
        save: handleSave,
        getCurrentFile: () => currentFile,
        getEditor: () => editorRef.current,
        switchToFile,
        closeFile,
    }));

    // Handle fileToOpen prop change
    useEffect(() => {
        if (fileToOpen) {
            switchToFile(fileToOpen);
        }
    }, [fileToOpen]);

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

        // 3. Define theme with initial mode
        defineMonacoTheme(DEBUG_THEME, 'dark');

        // Create editor without a model initially
        const editor = monaco.editor.create(containerRef.current, {
            model: null,
            theme: DEBUG_THEME,
            automaticLayout: true,
            minimap: { enabled: false },
            fontSize: 13,
            lineNumbers: 'on',
            scrollBeyondLastLine: false,
            wordWrap: 'on',
            padding: { top: 10, bottom: 10 },
            fontFamily: 'Geist, -apple-system, BlinkMacSystemFont, sans-serif',
            'semanticHighlighting.enabled': true,
        } as any);
        editorRef.current = editor;

        // @ts-ignore
        window.monaco = monaco;
        // @ts-ignore
        window.editor = editor;

        console.log('Editor created.');

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

        // Initialize LSP
        lspClient.initialize().then(() => {
            lspReady = true;
            console.log('[Editor] LSP ready');
        }).catch(err => {
            console.error('LSP initialization failed in Editor:', err);
        });

        return () => {
            editor.dispose();
            providerDisposable.dispose();
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

    return (
        <div ref={containerRef} style={{ width: '100%', height: '100%', overflow: 'hidden' }} />
    );
});
