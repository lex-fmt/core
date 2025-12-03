import * as monaco from 'monaco-editor';
import { ProtocolConnection } from 'vscode-languageserver-protocol/browser';

export function registerHoverProvider(languageId: string, connection: ProtocolConnection) {
    monaco.languages.registerHoverProvider(languageId, {
        provideHover: async (model, position) => {
            if (!connection) return null;
            const params = {
                textDocument: { uri: model.uri.toString() },
                position: { line: position.lineNumber - 1, character: position.column - 1 }
            };
            try {
                const result = await connection.sendRequest('textDocument/hover', params) as any;
                if (!result || !result.contents) return null;
                return {
                    contents: Array.isArray(result.contents) 
                        ? result.contents.map((c: any) => ({ value: typeof c === 'string' ? c : c.value }))
                        : [{ value: typeof result.contents === 'string' ? result.contents : result.contents.value }]
                };
            } catch (e) {
                console.error('[LSP] Hover failed:', e);
                return null;
            }
        }
    });
}
