import * as monaco from 'monaco-editor';
import { lspClient } from '../lsp/client';

export function registerCompletion() {
    monaco.languages.registerCompletionItemProvider('lex', {
        triggerCharacters: ['@', '/'],
        provideCompletionItems: async function (model, position, context, _token) {
            const response = await lspClient.sendRequest('textDocument/completion', {
                textDocument: { uri: model.uri.toString() },
                position: { line: position.lineNumber - 1, character: position.column - 1 },
                context: {
                    triggerKind: context.triggerKind === monaco.languages.CompletionTriggerKind.TriggerCharacter ? 2 : 1,
                    triggerCharacter: context.triggerCharacter
                }
            });

            if (response) {
                const items = Array.isArray(response) ? response : response.items;
                return {
                    suggestions: items.map((item: any) => ({
                        label: item.label,
                        kind: item.kind, // Map LSP kind to Monaco kind if necessary, but usually compatible
                        insertText: item.insertText || item.label,
                        detail: item.detail,
                        documentation: item.documentation,
                        range: item.textEdit ? {
                            startLineNumber: item.textEdit.range.start.line + 1,
                            startColumn: item.textEdit.range.start.character + 1,
                            endLineNumber: item.textEdit.range.end.line + 1,
                            endColumn: item.textEdit.range.end.character + 1
                        } : undefined
                    }))
                };
            }
            return { suggestions: [] };
        }
    });
}
