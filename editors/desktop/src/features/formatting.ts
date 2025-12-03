import * as monaco from 'monaco-editor';
import { lspClient } from '../lsp/client';
import type { LspFormattingEdit } from '@/lsp/types';

function toMonacoEdit(edit: LspFormattingEdit): monaco.languages.TextEdit {
  return {
    range: {
      startLineNumber: edit.range.start.line + 1,
      startColumn: edit.range.start.character + 1,
      endLineNumber: edit.range.end.line + 1,
      endColumn: edit.range.end.character + 1,
    },
    text: edit.newText,
  };
}

export function registerFormatting() {
  monaco.languages.registerDocumentFormattingEditProvider('lex', {
    provideDocumentFormattingEdits: async function (model, options, token) {
            void options;
            void token;
            const response = await lspClient.sendRequest<LspFormattingEdit[]>('textDocument/formatting', {
                textDocument: { uri: model.uri.toString() },
                options: { tabSize: 2, insertSpaces: true } // Default options
            });

            if (response) {
                return response.map(toMonacoEdit);
            }
            return [];
        }
    });

    monaco.languages.registerDocumentRangeFormattingEditProvider('lex', {
        provideDocumentRangeFormattingEdits: async function (model, range, options, token) {
            void options;
            void token;
            const response = await lspClient.sendRequest<LspFormattingEdit[]>('textDocument/rangeFormatting', {
                textDocument: { uri: model.uri.toString() },
                range: {
                    start: { line: range.startLineNumber - 1, character: range.startColumn - 1 },
                    end: { line: range.endLineNumber - 1, character: range.endColumn - 1 }
                },
                options: { tabSize: 2, insertSpaces: true }
            });

            if (response) {
                return response.map(toMonacoEdit);
            }
            return [];
        }
    });
}
