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

            // Listen for diagnostics
            this.connection.onNotification('textDocument/publishDiagnostics', (params: any) => {
                const uri = monaco.Uri.parse(params.uri);
                const model = monaco.editor.getModel(uri);
                if (model) {
                    const markers: monaco.editor.IMarkerData[] = params.diagnostics.map((d: any) => ({
                        severity: d.severity === 1 ? monaco.MarkerSeverity.Error :
                                  d.severity === 2 ? monaco.MarkerSeverity.Warning :
                                  d.severity === 3 ? monaco.MarkerSeverity.Info :
                                  monaco.MarkerSeverity.Hint,
                        message: d.message,
                        startLineNumber: d.range.start.line + 1,
                        startColumn: d.range.start.character + 1,
                        endLineNumber: d.range.end.line + 1,
                        endColumn: d.range.end.character + 1,
                        code: d.code ? String(d.code) : undefined,
                        source: d.source
                    }));
                    monaco.editor.setModelMarkers(model, 'lex', markers);
                }
            });

            this.registerProviders();
        })();

        return this.readyPromise;
    }

    private registerProviders() {
        const languageId = 'lex';
        if (!this.connection) return;

        import('./providers/completion').then(m => m.registerCompletionProvider(languageId, this.connection!));
        import('./providers/hover').then(m => m.registerHoverProvider(languageId, this.connection!));
        import('./providers/formatting').then(m => m.registerFormattingProvider(languageId, this.connection!));
        import('./providers/definition').then(m => m.registerDefinitionProvider(languageId, this.connection!));
        import('./providers/semantic_tokens').then(m => m.registerSemanticTokensProvider(languageId, this.connection!));
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
