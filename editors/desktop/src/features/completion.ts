import * as monaco from 'monaco-editor';
import { lspClient } from '../lsp/client';
import type { LspCompletionItem, LspCompletionResponse } from '@/lsp/types';

function normalizeCompletionResponse(response: LspCompletionResponse | null | undefined): LspCompletionItem[] {
  if (!response) {
    return [];
  }
  return Array.isArray(response) ? response : response.items ?? [];
}

function toMonacoRange(range?: { start: { line: number; character: number }; end: { line: number; character: number } }): monaco.IRange | undefined {
  if (!range) return undefined;
  return {
    startLineNumber: range.start.line + 1,
    startColumn: range.start.character + 1,
    endLineNumber: range.end.line + 1,
    endColumn: range.end.character + 1,
  };
}

export function registerCompletion() {
  monaco.languages.registerCompletionItemProvider('lex', {
    triggerCharacters: ['@', '/'],
    provideCompletionItems: async (model, position, context) => {
      const response = await lspClient.sendRequest<LspCompletionResponse>('textDocument/completion', {
        textDocument: { uri: model.uri.toString() },
        position: { line: position.lineNumber - 1, character: position.column - 1 },
        context: {
          triggerKind: context.triggerKind === monaco.languages.CompletionTriggerKind.TriggerCharacter ? 2 : 1,
          triggerCharacter: context.triggerCharacter
        }
      });

      const items = normalizeCompletionResponse(response);
      return {
        suggestions: items.map(item => ({
          label: item.label,
          kind: item.kind,
          insertText: item.insertText ?? item.label,
          detail: item.detail,
          documentation: item.documentation,
          range: toMonacoRange(item.textEdit?.range),
        })),
      };
    }
  });
}
