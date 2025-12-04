import { createProtocolConnection, ProtocolConnection, Logger, InitializeParams, InitializeRequest, InitializedNotification } from 'vscode-languageserver-protocol/browser';
import { IpcMessageReader, IpcMessageWriter } from './ipc-connection';
import * as monaco from 'monaco-editor';
import { LspPublishDiagnosticsParams } from './types';

export class LspClient {
    private connection: ProtocolConnection | null = null;
    private readyPromise: Promise<void> | null = null;
    private isDisposed = false;
    private retryCount = 0;
    private readonly maxRetries = 5;
    private readonly baseRetryDelay = 1000;
    private reconnectTimer: ReturnType<typeof setTimeout> | null = null;

    constructor() {
    }

    public start(): Promise<void> {
        if (this.readyPromise) return this.readyPromise;

        this.readyPromise = this.initialize();
        return this.readyPromise;
    }

    private async initialize(): Promise<void> {
        if (this.isDisposed) return;

        console.log(`[LspClient] Starting SimpleLspClient (Attempt ${this.retryCount + 1}/${this.maxRetries + 1})...`);
        
        try {
            const reader = new IpcMessageReader(window.ipcRenderer);
            const writer = new IpcMessageWriter(window.ipcRenderer);
            
            const logger: Logger = {
                error: (message) => console.error('[LSP]', message),
                warn: (message) => console.warn('[LSP]', message),
                info: (message) => console.info('[LSP]', message),
                log: (message) => console.log('[LSP]', message)
            };

            this.connection = createProtocolConnection(reader, writer, logger);
            
            this.connection.onClose(() => {
                console.warn('[LspClient] Connection closed.');
                this.handleConnectionLoss();
            });

            this.connection.onError((error) => {
                console.error('[LspClient] Connection error:', error);
                this.handleConnectionLoss();
            });

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

            // Reset retry count on successful connection
            this.retryCount = 0;

            // Listen for diagnostics
            this.connection.onNotification('textDocument/publishDiagnostics', (params: LspPublishDiagnosticsParams) => {
                const uri = monaco.Uri.parse(params.uri);
                const model = monaco.editor.getModel(uri);
                if (model) {
                    const markers: monaco.editor.IMarkerData[] = params.diagnostics.map((d) => ({
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
        } catch (error) {
            console.error('[LspClient] Initialization failed:', error);
            this.handleConnectionLoss();
            throw error;
        }
    }

    private handleConnectionLoss() {
        if (this.isDisposed) return;

        this.connection = null;
        this.readyPromise = null;

        if (this.retryCount < this.maxRetries) {
            const delay = this.baseRetryDelay * Math.pow(2, this.retryCount);
            console.log(`[LspClient] Reconnecting in ${delay}ms...`);
            this.retryCount++;
            
            if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
            this.reconnectTimer = setTimeout(() => {
                this.start().catch(err => console.error('[LspClient] Reconnection failed:', err));
            }, delay);
        } else {
            console.error('[LspClient] Max retries exceeded. Giving up.');
        }
    }

    public dispose() {
        this.isDisposed = true;
        if (this.reconnectTimer) {
            clearTimeout(this.reconnectTimer);
            this.reconnectTimer = null;
        }
        if (this.connection) {
            this.connection.dispose();
            this.connection = null;
        }
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

    public async sendRequest<R, P = unknown>(method: string, params: P): Promise<R> {
        if (!this.readyPromise) {
            this.start();
        }
        await this.readyPromise;
        if (!this.connection) throw new Error('Client not initialized');
        const start = performance.now();
        try {
            const result = await this.connection.sendRequest(method, params) as R;
            const duration = performance.now() - start;
            console.log(`[LspClient] ${method} responded in ${duration.toFixed(1)}ms`);
            return result as R;
        } catch (error) {
            const duration = performance.now() - start;
            console.error(`[LspClient] ${method} failed after ${duration.toFixed(1)}ms`, error);
            throw error;
        }
    }

    public async onNotification<P = unknown>(method: string, handler: (params: P) => void): Promise<void> {
        if (!this.readyPromise) {
            this.start();
        }
        await this.readyPromise;
        if (!this.connection) {
            console.warn('LSP client not started, cannot register notification handler');
            return;
        }
        this.connection.onNotification(method, handler);
    }

    public async sendNotification<P = unknown>(method: string, params: P): Promise<void> {
        if (!this.readyPromise) {
            this.start();
        }
        await this.readyPromise;
        if (!this.connection) {
            console.warn('LSP client not started, cannot send notification');
            return;
        }
        this.connection.sendNotification(method, params);
    }
}

export const lspClient = new LspClient();
