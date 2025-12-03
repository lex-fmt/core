import { createProtocolConnection, ProtocolConnection, Logger, InitializeParams, InitializeRequest, InitializedNotification } from 'vscode-languageserver-protocol/browser';
import { IpcMessageReader, IpcMessageWriter } from './ipc-connection';
import * as monaco from 'monaco-editor';

export class LspClient {
    private connection: ProtocolConnection | null = null;
    private readyPromise: Promise<void> | null = null;

    constructor() {
    }

    public start(): Promise<void> {
        if (this.readyPromise) return this.readyPromise;

        this.readyPromise = (async () => {
            console.log('[LspClient] Starting SimpleLspClient...');
            
            const reader = new IpcMessageReader(window.ipcRenderer);
            const writer = new IpcMessageWriter(window.ipcRenderer);
            
            const logger: Logger = {
                error: (message) => console.error('[LSP]', message),
                warn: (message) => console.warn('[LSP]', message),
                info: (message) => console.info('[LSP]', message),
                log: (message) => console.log('[LSP]', message)
            };

            this.connection = createProtocolConnection(reader, writer, logger);
            this.connection.listen();

            // Initialize
            const initParams: InitializeParams = {
                processId: null,
                rootUri: null,
                capabilities: {
                    textDocument: {
                        synchronization: {
                            dynamicRegistration: true,
                            willSave: false,
                            willSaveWaitUntil: false,
                            didSave: false
                        },
                        completion: {
                            dynamicRegistration: true,
                            completionItem: {
                                snippetSupport: true,
                                commitCharactersSupport: true,
                                documentationFormat: ['markdown', 'plaintext'],
                                deprecatedSupport: true,
                                preselectSupport: true
                            },
                            contextSupport: true
                        },
                        hover: {
                            dynamicRegistration: true,
                            contentFormat: ['markdown', 'plaintext']
                        },
                        signatureHelp: {
                            dynamicRegistration: true,
                            signatureInformation: {
                                documentationFormat: ['markdown', 'plaintext']
                            }
                        },
                        definition: { dynamicRegistration: true },
                        references: { dynamicRegistration: true },
                        documentHighlight: { dynamicRegistration: true },
                        documentSymbol: {
                            dynamicRegistration: true,
                            symbolKind: {
                                valueSet: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26]
                            }
                        },
                        codeAction: {
                            dynamicRegistration: true,
                            codeActionLiteralSupport: {
                                codeActionKind: {
                                    valueSet: ['', 'quickfix', 'refactor', 'refactor.extract', 'refactor.inline', 'refactor.rewrite', 'source', 'source.organizeImports']
                                }
                            }
                        },
                        codeLens: { dynamicRegistration: true },
                        formatting: { dynamicRegistration: true },
                        rangeFormatting: { dynamicRegistration: true },
                        onTypeFormatting: { dynamicRegistration: true },
                        rename: { dynamicRegistration: true },
                        documentLink: { dynamicRegistration: true },
                        typeDefinition: { dynamicRegistration: true },
                        implementation: { dynamicRegistration: true },
                        colorProvider: { dynamicRegistration: true },
                        foldingRange: { dynamicRegistration: true },
                        selectionRange: { dynamicRegistration: true },
                        publishDiagnostics: { relatedInformation: true },
                        semanticTokens: {
                            dynamicRegistration: true,
                            tokenTypes: [
                                'DocumentTitle', 'SessionMarker', 'SessionTitleText', 'DefinitionSubject', 'DefinitionContent',
                                'ListMarker', 'ListItemText', 'AnnotationLabel', 'AnnotationParameter', 'AnnotationContent',
                                'InlineStrong', 'InlineEmphasis', 'InlineCode', 'InlineMath', 'Reference', 'ReferenceCitation',
                                'ReferenceFootnote', 'VerbatimSubject', 'VerbatimLanguage', 'VerbatimAttribute', 'VerbatimContent',
                                'InlineMarker_strong_start', 'InlineMarker_strong_end', 'InlineMarker_emphasis_start', 'InlineMarker_emphasis_end',
                                'InlineMarker_code_start', 'InlineMarker_code_end', 'InlineMarker_math_start', 'InlineMarker_math_end',
                                'InlineMarker_ref_start', 'InlineMarker_ref_end',
                                // Standard types as fallback
                                'comment', 'string', 'keyword', 'number', 'regexp', 'operator', 'namespace',
                                'type', 'struct', 'class', 'interface', 'enum', 'typeParameter', 'function',
                                'method', 'decorator', 'macro', 'variable', 'parameter', 'property', 'label'
                            ],
                            tokenModifiers: ['declaration', 'definition', 'readonly', 'static', 'deprecated', 'abstract', 'async', 'modification', 'documentation', 'defaultLibrary'],
                            formats: ['relative'],
                            requests: {
                                range: true,
                                full: {
                                    delta: true
                                }
                            }
                        }
                    },
                    workspace: {
                        applyEdit: true,
                        workspaceEdit: {
                            documentChanges: true
                        },
                        didChangeConfiguration: { dynamicRegistration: true },
                        didChangeWatchedFiles: { dynamicRegistration: true },
                        symbol: { dynamicRegistration: true },
                        executeCommand: { dynamicRegistration: true }
                    }
                }
            };

            console.log('[LspClient] Sending initialize request...');
            const result = await this.connection.sendRequest(InitializeRequest.type, initParams);
            console.log('[LspClient] Initialize result:', result);

            await this.connection.sendNotification(InitializedNotification.type, {});
            console.log('[LspClient] Initialized');

            this.registerProviders();
        })();

        return this.readyPromise;
    }

    private registerProviders() {
        const languageId = 'lex';

        // Completion
        monaco.languages.registerCompletionItemProvider(languageId, {
            provideCompletionItems: async (model, position, context) => {
                if (!this.connection) return { suggestions: [] };
                
                const params = {
                    textDocument: { uri: model.uri.toString() },
                    position: { line: position.lineNumber - 1, character: position.column - 1 },
                    context: {
                        triggerKind: context.triggerKind === monaco.languages.CompletionTriggerKind.TriggerCharacter ? 2 : 1,
                        triggerCharacter: context.triggerCharacter
                    }
                };

                try {
                    const result = await this.connection.sendRequest('textDocument/completion', params) as any;
                    const items = Array.isArray(result) ? result : result.items;
                    return {
                        suggestions: items.map((item: any) => ({
                            label: item.label,
                            kind: item.kind ? item.kind - 1 : monaco.languages.CompletionItemKind.Text,
                            insertText: item.insertText || item.label,
                            insertTextRules: item.insertTextFormat === 2 ? monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet : undefined,
                            documentation: item.documentation,
                            detail: item.detail,
                            range: item.textEdit ? {
                                startLineNumber: item.textEdit.range.start.line + 1,
                                startColumn: item.textEdit.range.start.character + 1,
                                endLineNumber: item.textEdit.range.end.line + 1,
                                endColumn: item.textEdit.range.end.character + 1
                            } : undefined
                        }))
                    };
                } catch (e) {
                    console.error('[LSP] Completion failed:', e);
                    return { suggestions: [] };
                }
            }
        });

        // Hover
        monaco.languages.registerHoverProvider(languageId, {
            provideHover: async (model, position) => {
                if (!this.connection) return null;
                const params = {
                    textDocument: { uri: model.uri.toString() },
                    position: { line: position.lineNumber - 1, character: position.column - 1 }
                };
                try {
                    const result = await this.connection.sendRequest('textDocument/hover', params) as any;
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

        // Formatting
        monaco.languages.registerDocumentFormattingEditProvider(languageId, {
            provideDocumentFormattingEdits: async (model, options) => {
                if (!this.connection) return [];
                const params = {
                    textDocument: { uri: model.uri.toString() },
                    options: {
                        tabSize: options.tabSize,
                        insertSpaces: options.insertSpaces
                    }
                };
                try {
                    const result = await this.connection.sendRequest('textDocument/formatting', params) as any[];
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
                if (!this.connection) return [];
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
                    const result = await this.connection.sendRequest('textDocument/rangeFormatting', params) as any[];
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

        // Definition
        monaco.languages.registerDefinitionProvider(languageId, {
            provideDefinition: async (model, position) => {
                if (!this.connection) return null;
                const params = {
                    textDocument: { uri: model.uri.toString() },
                    position: { line: position.lineNumber - 1, character: position.column - 1 }
                };
                try {
                    const result = await this.connection.sendRequest('textDocument/definition', params) as any;
                    if (!result) return null;
                    const locations = Array.isArray(result) ? result : [result];
                    return locations.map((loc: any) => ({
                        uri: monaco.Uri.parse(loc.uri),
                        range: {
                            startLineNumber: loc.range.start.line + 1,
                            startColumn: loc.range.start.character + 1,
                            endLineNumber: loc.range.end.line + 1,
                            endColumn: loc.range.end.character + 1
                        }
                    }));
                } catch (e) {
                    console.error('[LSP] Definition failed:', e);
                    return null;
                }
            }
        });

        // Semantic Tokens
        monaco.languages.registerDocumentSemanticTokensProvider(languageId, {
            getLegend: () => ({
                tokenTypes: [
                    'DocumentTitle', 'SessionMarker', 'SessionTitleText', 'DefinitionSubject', 'DefinitionContent',
                    'ListMarker', 'ListItemText', 'AnnotationLabel', 'AnnotationParameter', 'AnnotationContent',
                    'InlineStrong', 'InlineEmphasis', 'InlineCode', 'InlineMath', 'Reference', 'ReferenceCitation',
                    'ReferenceFootnote', 'VerbatimSubject', 'VerbatimLanguage', 'VerbatimAttribute', 'VerbatimContent',
                    'InlineMarker_strong_start', 'InlineMarker_strong_end', 'InlineMarker_emphasis_start', 'InlineMarker_emphasis_end',
                    'InlineMarker_code_start', 'InlineMarker_code_end', 'InlineMarker_math_start', 'InlineMarker_math_end',
                    'InlineMarker_ref_start', 'InlineMarker_ref_end',
                    // Standard types
                    'comment', 'string', 'keyword', 'number', 'regexp', 'operator', 'namespace',
                    'type', 'struct', 'class', 'interface', 'enum', 'typeParameter', 'function',
                    'method', 'decorator', 'macro', 'variable', 'parameter', 'property', 'label'
                ],
                tokenModifiers: ['declaration', 'definition', 'readonly', 'static', 'deprecated', 'abstract', 'async', 'modification', 'documentation', 'defaultLibrary']
            }),
            provideDocumentSemanticTokens: async (model) => {
                console.log('[SemanticTokens] Provider triggered');
                if (!this.connection) return null;
                const params = {
                    textDocument: { uri: model.uri.toString() }
                };
                try {
                    const result = await this.connection.sendRequest('textDocument/semanticTokens/full', params) as any;
                    if (!result || !result.data) return null;
                    console.log(`[SemanticTokens] Received tokens: ${result.data.length}`);
                    return {
                        data: new Uint32Array(result.data)
                    };
                } catch (e) {
                    console.error('[LSP] Semantic Tokens failed:', e);
                    return null;
                }
            },
            releaseDocumentSemanticTokens: () => {}
        });
    }

    public async sendRequest<R, P = any>(method: string, params: P): Promise<R> {
        if (!this.readyPromise) {
            this.start();
        }
        await this.readyPromise;
        if (!this.connection) throw new Error('Client not initialized');
        // @ts-ignore
        return this.connection.sendRequest(method, params);
    }

    public async onNotification<P = any>(method: string, handler: (params: P) => void): Promise<void> {
        if (!this.readyPromise) {
            this.start();
        }
        await this.readyPromise;
        if (!this.connection) {
            console.warn('LSP client not started, cannot register notification handler');
            return;
        }
        // @ts-ignore
        this.connection.onNotification(method, handler);
    }

    public async sendNotification<P = any>(method: string, params: P): Promise<void> {
        if (!this.readyPromise) {
            this.start();
        }
        await this.readyPromise;
        if (!this.connection) {
            console.warn('LSP client not started, cannot send notification');
            return;
        }
        // @ts-ignore
        this.connection.sendNotification(method, params);
    }
}

export const lspClient = new LspClient();
