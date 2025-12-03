import type { IpcRenderer, IpcRendererEvent } from 'electron';

type JsonRpcId = number | string;

interface JsonRpcBase {
  jsonrpc: '2.0';
}

interface JsonRpcSuccess<T = unknown> extends JsonRpcBase {
  id: JsonRpcId;
  result: T;
}

interface JsonRpcErrorPayload<E = unknown> {
  code: number;
  message: string;
  data?: E;
}

interface JsonRpcError<E = unknown> extends JsonRpcBase {
  id: JsonRpcId | null;
  error: JsonRpcErrorPayload<E>;
}

interface JsonRpcNotification<T = unknown> extends JsonRpcBase {
  method: string;
  params?: T;
}

export type LspMessage<T = unknown, E = unknown> =
  | JsonRpcSuccess<T>
  | JsonRpcError<E>
  | JsonRpcNotification<T>;

type PendingRequest = {
  resolve: (value: unknown) => void;
  reject: (error: unknown) => void;
};

export type LspStatus = 'Initializing' | 'Ready' | 'Error';

export class LspClient {
  private ipcRenderer: IpcRenderer;
  private languageId: string;
  private requestId = 0;
  private pendingRequests = new Map<JsonRpcId, PendingRequest>();
  private notificationHandlers = new Map<string, (params: unknown) => void>();
  private statusHandlers: ((status: LspStatus) => void)[] = [];
  public currentStatus: LspStatus = 'Initializing';

  private buffer: Uint8Array = new Uint8Array(0);

  constructor(ipcRenderer: IpcRenderer, languageId: string) {
    this.ipcRenderer = ipcRenderer;
    this.languageId = languageId;
    console.log(`[LspClient] Initializing for language: ${languageId}`);
    this.setStatus('Initializing');

    this.ipcRenderer.on('lsp-output', (_event: IpcRendererEvent, data: Uint8Array) => {
      this.appendBuffer(data);
      this.processBuffer();
    });
  }

  async start() {
    console.log('[LspClient] Starting...');
    // await this.ipcRenderer.invoke('start-lsp'); // Assuming start-lsp is handled by main process automatically or elsewhere
    this.initialize();
  }

  private setStatus(status: LspStatus) {
      this.currentStatus = status;
      this.statusHandlers.forEach(h => h(status));
  }

  public onStatusChange(handler: (status: LspStatus) => void) {
      this.statusHandlers.push(handler);
      handler(this.currentStatus); // Emit current status immediately
  }

  public serverCapabilities: Record<string, unknown> = {};

  public async initialize() {
    console.log('[LspClient] Initializing LSP session...');
    try {
        const response = await this.sendRequest<{ capabilities?: Record<string, unknown> }>('initialize', {
            processId: null,
            rootUri: null,
            capabilities: {},
            clientInfo: {
                name: 'Lex Editor',
                version: '1.0.0'
            }
        });
        console.log('[LspClient] Initialization response:', JSON.stringify(response, null, 2));
        if (response && response.capabilities) {
            this.serverCapabilities = response.capabilities;
        }
        this.sendNotification('initialized', {});
        this.setStatus('Ready');
    } catch (error: unknown) {
        console.error('[LspClient] Initialization failed:', error);
        // If error code is -32600 (Invalid Request), it likely means the server is already initialized.
        if (isJsonRpcErrorPayload(error) && error.code === -32600) {
            console.log('[LspClient] Server returned Invalid Request for initialize. Assuming already initialized.');
            this.setStatus('Ready');
        } else {
            this.setStatus('Error');
            throw error;
        }
    }
  }

  private appendBuffer(data: Uint8Array) {
    const newBuffer = new Uint8Array(this.buffer.length + data.length);
    newBuffer.set(this.buffer);
    newBuffer.set(data, this.buffer.length);
    this.buffer = newBuffer;
  }

  private processBuffer() {
    let shouldContinue = true;
    while (shouldContinue) {
      shouldContinue = this.tryProcessMessage();
    }
  }

  private tryProcessMessage(): boolean {
    const headerEndIndex = findHeaderEnd(this.buffer);
    if (headerEndIndex === -1) {
      return false;
    }

    const headerBytes = this.buffer.slice(0, headerEndIndex);
    const headerString = new TextDecoder().decode(headerBytes);
    const match = headerString.match(/Content-Length: (\d+)/);

    if (!match) {
      console.error('Invalid LSP header, discarding:', headerString);
      this.buffer = this.buffer.slice(headerEndIndex);
      return this.buffer.length > 0;
    }

    const contentLength = Number.parseInt(match[1], 10);
    const totalLength = headerEndIndex + contentLength;
    if (this.buffer.length < totalLength) {
      return false;
    }

    const bodyBytes = this.buffer.slice(headerEndIndex, totalLength);
    this.buffer = this.buffer.slice(totalLength);

    try {
      const bodyString = new TextDecoder().decode(bodyBytes);
      const message = JSON.parse(bodyString) as LspMessage;
      this.handleMessage(message);
    } catch (error) {
      console.error('Failed to parse LSP message', error);
    }

    return this.buffer.length > 0;
  }

  private handleMessage(message: LspMessage) {
    // Only log errors, not every message
    if (message.error) {
      console.log(`[LspClient] Error response id=${message.id}: ${JSON.stringify(message.error)}`);
    }
    if ('id' in message && message.id !== undefined && this.pendingRequests.has(message.id)) {
        // Response to a request
        if (this.pendingRequests.has(message.id)) {
            const { resolve, reject } = this.pendingRequests.get(message.id)!;
            this.pendingRequests.delete(message.id);
            if (message.error) {
                reject(message.error);
            } else {
                resolve((message as JsonRpcSuccess).result);
            }
        }
    } else if (message.method) {
        // Notification or Request from server
        const handler = this.notificationHandlers.get(message.method);
        if (handler) {
            handler(message.params);
        }
    }
  }

  public sendRequest<TResponse = unknown, TParams = unknown>(method: string, params: TParams): Promise<TResponse> {
    const id = this.requestId++;
    const request = {
      jsonrpc: '2.0',
      id,
      method,
      params,
    };
    
    this.send(request);
    
    return new Promise<TResponse>((resolve, reject) => {
      this.pendingRequests.set(id, {
        resolve: value => resolve(value as TResponse),
        reject,
      });
    });
  }

  public sendNotification<TParams = unknown>(method: string, params: TParams) {
    const notification = {
      jsonrpc: '2.0',
      method,
      params,
    };
    this.send(notification);
  }

  private send(message: JsonRpcSuccess | JsonRpcNotification | JsonRpcError) {
    const json = JSON.stringify(message);
    const encoder = new TextEncoder();
    const encoded = encoder.encode(json);
    const payload = `Content-Length: ${encoded.length}\r\n\r\n${json}`;
    this.ipcRenderer.send('lsp-input', payload);
  }

  public onNotification(method: string, handler: (params: unknown) => void) {
    this.notificationHandlers.set(method, handler);
  }
}

function findHeaderEnd(buffer: Uint8Array): number {
  for (let i = 0; i < buffer.length - 3; i++) {
    if (buffer[i] === 13 && buffer[i + 1] === 10 && buffer[i + 2] === 13 && buffer[i + 3] === 10) {
      return i + 4;
    }
  }
  return -1;
}

function isJsonRpcErrorPayload(value: unknown): value is JsonRpcErrorPayload {
  return Boolean(value && typeof value === 'object' && 'code' in value);
}

const rendererIpc = typeof window !== 'undefined' ? window.ipcRenderer : undefined;
if (!rendererIpc) {
  throw new Error('ipcRenderer is not available in this environment');
}

// Export a singleton instance
export const lspClient = new LspClient(rendererIpc, 'lex');
