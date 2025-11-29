import * as monaco from 'monaco-editor';
import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker&inline';
import jsonWorker from 'monaco-editor/esm/vs/language/json/json.worker?worker&inline';
import cssWorker from 'monaco-editor/esm/vs/language/css/css.worker?worker&inline';
import htmlWorker from 'monaco-editor/esm/vs/language/html/html.worker?worker&inline';
import tsWorker from 'monaco-editor/esm/vs/language/typescript/ts.worker?worker&inline';

const DEBUG_LANG = 'lex';
const DEBUG_THEME = 'lex-theme';

// @ts-ignore
window.MonacoEnvironment = {
    getWorker(_: any, label: string) {
        if (label === 'json') {
            return new jsonWorker();
        }
        if (label === 'css' || label === 'scss' || label === 'less') {
            return new cssWorker();
        }
        if (label === 'html' || label === 'handlebars' || label === 'razor') {
            return new htmlWorker();
        }
        if (label === 'typescript' || label === 'javascript') {
            return new tsWorker();
        }
        return new editorWorker();
    }
};

import { lspClient } from './lsp/client';

export function initDebugMonaco() {
    // Register lex language
    monaco.languages.register({ id: DEBUG_LANG, extensions: ['.lex'] });

    monaco.languages.setLanguageConfiguration(DEBUG_LANG, {
        comments: {
            lineComment: '#',
        }
    });

    // Semantic Token Legend (Must match VSCode package.json and Server)
    const tokenTypes = [
        "DocumentTitle", "SessionMarker", "SessionTitleText", "DefinitionSubject", "DefinitionContent",
        "ListMarker", "ListItemText", "AnnotationLabel", "AnnotationParameter", "AnnotationContent",
        "InlineStrong", "InlineEmphasis", "InlineCode", "InlineMath", "Reference", "ReferenceCitation",
        "ReferenceFootnote", "VerbatimSubject", "VerbatimLanguage", "VerbatimAttribute", "VerbatimContent",
        "InlineMarker_strong_start", "InlineMarker_strong_end", "InlineMarker_emphasis_start",
        "InlineMarker_emphasis_end", "InlineMarker_code_start", "InlineMarker_code_end",
        "InlineMarker_math_start", "InlineMarker_math_end", "InlineMarker_ref_start", "InlineMarker_ref_end"
    ];
    const tokenModifiers: string[] = [];

    const legend = {
        tokenTypes,
        tokenModifiers
    };

    monaco.languages.registerDocumentSemanticTokensProvider(DEBUG_LANG, {
        getLegend: function () {
            return legend;
        },
        provideDocumentSemanticTokens: async function (model, _lastResultId, _token) {
            const uri = model.uri.toString();
            console.log('[Monaco] Requesting semantic tokens for:', uri);
            try {
                // Wait for LSP to be ready? It should be if the file is open.
                const result = await lspClient.sendRequest('textDocument/semanticTokens/full', {
                    textDocument: { uri }
                });

                console.log('[Monaco] Received semantic tokens:', result);

                if (result && result.data) {
                    return {
                        data: new Uint32Array(result.data)
                    };
                }
            } catch (e) {
                console.error('[Monaco] Failed to get semantic tokens:', e);
            }
            return { data: new Uint32Array([]) };
        },
        releaseDocumentSemanticTokens: function () { }
    });

    monaco.languages.registerDocumentRangeSemanticTokensProvider(DEBUG_LANG, {
        getLegend: function () {
            return legend;
        },
        provideDocumentRangeSemanticTokens: async function (model, _range, _token) {
            const uri = model.uri.toString();
            console.log('[Monaco] Requesting range semantic tokens for:', uri);
            try {
                // For now, request full tokens even for range (optimization later)
                const result = await lspClient.sendRequest('textDocument/semanticTokens/full', {
                    textDocument: { uri }
                });

                console.log('[Monaco] Received range semantic tokens:', result);

                if (result && result.data) {
                    return {
                        data: new Uint32Array(result.data)
                    };
                }
            } catch (e) {
                console.error('[Monaco] Failed to get range semantic tokens:', e);
            }
            return { data: new Uint32Array([]) };
        }
    });

    // Monarch provider as fallback (or for simple things like comments if LSP fails)
    monaco.languages.setMonarchTokensProvider(DEBUG_LANG, {
        tokenizer: {
            root: [
                [/^Session Title.*/, 'sessionTitle'],
                [/^#.*/, 'comment'],
                [/[a-z]+/, 'string'],
            ]
        }
    });

    // Define Theme
    monaco.editor.defineTheme(DEBUG_THEME, {
        base: 'vs-dark',
        inherit: true,
        rules: [
            { token: 'sessionTitle', foreground: '#FF0000', fontStyle: 'bold' },
            { token: 'comment', foreground: '#888888' },
            // Semantic Token Colors
            { token: 'DocumentTitle', foreground: '#FFD700', fontStyle: 'bold' }, // Gold
            { token: 'SessionMarker', foreground: '#FF0000', fontStyle: 'bold' }, // Red
            { token: 'SessionTitleText', foreground: '#FF4500', fontStyle: 'bold' }, // OrangeRed
            { token: 'DefinitionSubject', foreground: '#00BFFF', fontStyle: 'bold' }, // DeepSkyBlue
            { token: 'ListMarker', foreground: '#32CD32' }, // LimeGreen
            { token: 'AnnotationLabel', foreground: '#DA70D6' }, // Orchid
            { token: 'VerbatimLanguage', foreground: '#ADFF2F' }, // GreenYellow
            { token: 'VerbatimContent', foreground: '#D3D3D3' }, // LightGray
        ],
        colors: {},
        // @ts-ignore
        semanticHighlighting: true
    });
}
