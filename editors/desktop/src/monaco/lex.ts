import * as monaco from 'monaco-editor';
import { LEGEND } from '@lex/shared';
import { lspClient } from '@/lsp/client';
import { registerFormatting } from '@/features/formatting';
import { registerCompletion } from '@/features/completion';
import type { LspRange } from '@/lsp/types';

type HoverContent = string | { value: string };

interface HoverResponse {
  contents?: HoverContent | HoverContent[];
}

interface DefinitionLocation {
  uri: string;
  range: LspRange;
}

interface Diagnostic {
  severity?: number;
  range: LspRange;
  message: string;
}

interface PublishDiagnosticsParams {
  uri: string;
  diagnostics: Diagnostic[];
}

let registered = false;

export function registerLexLanguage() {
  if (registered) return;
  registered = true;

  const languages = monaco.languages.getLanguages();
  const lexRegistered = languages.some(language => language.id === 'lex');
  if (!lexRegistered) {
    monaco.languages.register({ id: 'lex', extensions: ['.lex'] });
    monaco.languages.setLanguageConfiguration('lex', {
      comments: { lineComment: '#' },
    });
  }

  registerFormatting();
  registerCompletion();

  monaco.languages.registerDocumentSemanticTokensProvider('lex', {
    getLegend: () => LEGEND,
    async provideDocumentSemanticTokens(model) {
      try {
        const response = await lspClient.sendRequest('textDocument/semanticTokens/full', {
          textDocument: { uri: model.uri.toString() },
        });
        if (response?.data) {
          return {
            data: new Uint32Array(response.data),
            resultId: response.resultId,
          };
        }
      } catch (error) {
        console.error('[LexMonaco] Failed to fetch semantic tokens', error);
      }
      return { data: new Uint32Array([]) };
    },
    releaseDocumentSemanticTokens() {},
  });

  monaco.languages.registerHoverProvider('lex', {
    async provideHover(model, position) {
      const response = await lspClient.sendRequest('textDocument/hover', {
        textDocument: { uri: model.uri.toString() },
        position: { line: position.lineNumber - 1, character: position.column - 1 },
      });

      const contents = normalizeHoverContents((response as HoverResponse | null)?.contents);
      if (!contents.length) {
        return null;
      }
      return {
        contents,
        range: new monaco.Range(
          position.lineNumber,
          position.column,
          position.lineNumber,
          position.column,
        ),
      };
    },
  });

  monaco.languages.registerDefinitionProvider('lex', {
    async provideDefinition(model, position) {
      const response = await lspClient.sendRequest<DefinitionLocation | DefinitionLocation[] | null>('textDocument/definition', {
        textDocument: { uri: model.uri.toString() },
        position: { line: position.lineNumber - 1, character: position.column - 1 },
      });

      if (!response) return null;

      const locations = Array.isArray(response) ? response : [response];
      return locations.map(location => ({
        uri: monaco.Uri.parse(location.uri),
        range: toMonacoRange(location.range),
      }));
    },
  });

  monaco.languages.registerDocumentRangeSemanticTokensProvider('lex', {
    getLegend: () => LEGEND,
    async provideDocumentRangeSemanticTokens() {
      return { data: new Uint32Array([]), resultId: undefined };
    },
  });

  lspClient.onNotification('textDocument/publishDiagnostics', params => {
    if (!isPublishDiagnosticsParams(params)) {
      return;
    }
    const uri = monaco.Uri.parse(params.uri);
    const model = monaco.editor.getModel(uri);
    if (!model) return;

    const markers = params.diagnostics
      .filter(isDiagnostic)
      .map(diag => ({
        severity: diag.severity === 1 ? monaco.MarkerSeverity.Error : monaco.MarkerSeverity.Warning,
        ...toMonacoRange(diag.range),
        message: diag.message,
        source: 'Lex LSP',
      }));
    monaco.editor.setModelMarkers(model, 'lex', markers);
  });
}

function normalizeHoverContents(contents: HoverContent | HoverContent[] | undefined): monaco.IMarkdownString[] {
  if (!contents) {
    return [];
  }
  const items = Array.isArray(contents) ? contents : [contents];
  return items
    .map(item => (typeof item === 'string' ? { value: item } : item))
    .filter((item): item is monaco.IMarkdownString => typeof item.value === 'string');
}

function toMonacoRange(range: LspRange): monaco.IRange {
  return {
    startLineNumber: range.start.line + 1,
    startColumn: range.start.character + 1,
    endLineNumber: range.end.line + 1,
    endColumn: range.end.character + 1,
  };
}

function isPublishDiagnosticsParams(value: unknown): value is PublishDiagnosticsParams {
  return Boolean(
    value &&
    typeof value === 'object' &&
    'uri' in value &&
    typeof (value as { uri?: unknown }).uri === 'string' &&
    Array.isArray((value as { diagnostics?: unknown }).diagnostics)
  );
}

function isDiagnostic(value: unknown): value is Diagnostic {
  return Boolean(
    value &&
    typeof value === 'object' &&
    'range' in value &&
    typeof (value as { message?: unknown }).message === 'string'
  );
}
