import * as monaco from 'monaco-editor';
import { lspClient } from '../lsp/client';

export function registerFormatting() {
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
}
