import * as monaco from 'monaco-editor';
import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';
import jsonWorker from 'monaco-editor/esm/vs/language/json/json.worker?worker';
import cssWorker from 'monaco-editor/esm/vs/language/css/css.worker?worker';
import htmlWorker from 'monaco-editor/esm/vs/language/html/html.worker?worker';
import tsWorker from 'monaco-editor/esm/vs/language/typescript/ts.worker?worker';

// @ts-ignore
self.MonacoEnvironment = {
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

import { LEGEND } from '@lex/shared';

let initialized = false;

export function initMonaco() {
    if (initialized) return;
    initialized = true;

    console.log('Initializing Monaco configuration...');

    // Register Language
    monaco.languages.register({ id: 'lex', extensions: ['.lex'] });

    // Register Semantic Tokens Provider
    monaco.languages.registerDocumentSemanticTokensProvider('lex', {
        getLegend: function () {
            console.log('getLegend called');
            return LEGEND;
        },
        provideDocumentSemanticTokens: async function (model, _lastResultId, _token) {
            console.log('Requesting semantic tokens for', model.uri.toString());
            try {
                const response = await lspClient.sendRequest('textDocument/semanticTokens/full', {
                    textDocument: { uri: model.uri.toString() }
                });
                console.log('Semantic tokens response:', response ? 'received' : 'null', response?.data?.length);

                if (response && response.data) {
                    return {
                        data: new Uint32Array(response.data),
                        resultId: response.resultId
                    };
                }
            } catch (e) {
                console.error('Failed to fetch semantic tokens', e);
            }
            return null;
        },
        releaseDocumentSemanticTokens: function (_resultId) { }
    });

    // Register Semantic Tokens Provider (Range)
    monaco.languages.registerDocumentRangeSemanticTokensProvider('lex', {
        getLegend: function () {
            console.log('getLegend (Range) called');
            return LEGEND;
        },
        provideDocumentRangeSemanticTokens: async function (model, _range, _token) {
            console.log('Requesting range semantic tokens for', model.uri.toString());
            return {
                data: new Uint32Array([]),
                resultId: undefined
            };
        }
    });

    // Register Hover Provider
    monaco.languages.registerHoverProvider('lex', {
        provideHover: function (_model, position) {
            console.log('Hover provider called at', position.toString());
            return {
                contents: [
                    { value: '**Hover Test**' }
                ]
            };
        }
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

    // Define Theme (Lex Monochrome)
    monaco.editor.defineTheme('lex-dark', {
        base: 'vs-dark',
        inherit: true,
        // @ts-ignore
        semanticHighlighting: true,
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

    // Listen for Diagnostics
    lspClient.onNotification('textDocument/publishDiagnostics', (params: any) => {
        // params: { uri: string, diagnostics: Diagnostic[] }
        const uri = monaco.Uri.parse(params.uri);
        const model = monaco.editor.getModel(uri);
        
        if (model) {
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
    });
}
