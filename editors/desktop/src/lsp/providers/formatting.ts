import * as monaco from 'monaco-editor';
import { ProtocolConnection } from 'vscode-languageserver-protocol/browser';
import { LspTextEdit } from '../types';

export function registerFormattingProvider(languageId: string, connection: ProtocolConnection) {
    // Formatting
    monaco.languages.registerDocumentFormattingEditProvider(languageId, {
        provideDocumentFormattingEdits: async (model, options) => {
            if (!connection) return [];
            const params = {
                textDocument: { uri: model.uri.toString() },
                options: {
                    tabSize: options.tabSize,
                    insertSpaces: options.insertSpaces
                }
            };
            try {
                const result = await connection.sendRequest('textDocument/formatting', params) as LspTextEdit[] | null;
                if (!result) return [];
                return result.map(edit => ({
                    range: {
                        startLineNumber: edit.range.start.line + 1,
                        startColumn: edit.range.start.character + 1,
                        endLineNumber: edit.range.end.line + 1,
                        endColumn: edit.range.end.character + 1
                    },
                    text: edit.newText
                }));
            } catch (e) {
                console.error('[LSP] Formatting failed:', e);
                return [];
            }
        }
    });

    // Range Formatting
    monaco.languages.registerDocumentRangeFormattingEditProvider(languageId, {
        provideDocumentRangeFormattingEdits: async (model, range, options) => {
            if (!connection) return [];
            const params = {
                textDocument: { uri: model.uri.toString() },
                range: {
                    start: { line: range.startLineNumber - 1, character: range.startColumn - 1 },
                    end: { line: range.endLineNumber - 1, character: range.endColumn - 1 }
                },
                options: {
                    tabSize: options.tabSize,
                    insertSpaces: options.insertSpaces
                }
            };
            try {
                const result = await connection.sendRequest('textDocument/rangeFormatting', params) as LspTextEdit[] | null;
                if (!result) return [];
                return result.map(edit => ({
                    range: {
                        startLineNumber: edit.range.start.line + 1,
                        startColumn: edit.range.start.character + 1,
                        endLineNumber: edit.range.end.line + 1,
                        endColumn: edit.range.end.character + 1
                    },
                    text: edit.newText
                }));
            } catch (e) {
                console.error('[LSP] Range Formatting failed:', e);
                return [];
            }
        }
    });
}
