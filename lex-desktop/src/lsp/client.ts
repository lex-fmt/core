export type LspMessage = {
  jsonrpc: '2.0';
  id?: number | string;
  method?: string;
  result?: any;
  error?: any;
  params?: any;
};

export class LspClient {
  // @ts-ignore
  private ipcRenderer: any;
  // @ts-ignore
  private languageId: string;
  private requestId = 0;
  private pendingRequests = new Map<number | string, { resolve: (val: any) => void; reject: (err: any) => void }>();
  private notificationHandlers = new Map<string, (params: any) => void>();

  private buffer: Uint8Array = new Uint8Array(0);

  constructor(ipcRenderer: any, languageId: string) {
    this.ipcRenderer = ipcRenderer;
    this.languageId = languageId;

    // @ts-ignore
    window.ipcRenderer.on('lsp-output', (_event, data: Uint8Array) => {
        this.appendBuffer(data);
        this.processBuffer();
    });
  }

  async start() {
    // await this.ipcRenderer.invoke('start-lsp'); // Assuming start-lsp is handled by main process automatically or elsewhere
    this.initialize();
  }

  private initialize() {
    // Placeholder for actual LSP initialization logic
    // Example: send an initialize request
    this.sendRequest('initialize', {
      processId: null,
      rootUri: null,
      capabilities: {},
      clientInfo: {
        name: 'CodeMirror LSP Client',
        version: '1.0.0'
      }
    }).then(response => {
      // console.log('[LspClient] Initialization response:', response);
    }).catch(error => {
      console.error('[LspClient] Initialization failed:', error);
    });
  }

  private appendBuffer(data: Uint8Array) {
    const newBuffer = new Uint8Array(this.buffer.length + data.length);
    newBuffer.set(this.buffer);
    newBuffer.set(data, this.buffer.length);
    this.buffer = newBuffer;
  }

  private processBuffer() {
    while (true) {
        // Find header end (\r\n\r\n = 13 10 13 10)
        let headerEndIndex = -1;
        for (let i = 0; i < this.buffer.length - 3; i++) {
            if (this.buffer[i] === 13 && this.buffer[i+1] === 10 && this.buffer[i+2] === 13 && this.buffer[i+3] === 10) {
                headerEndIndex = i + 4;
                break;
            }
        }

        if (headerEndIndex === -1) break; // Not enough data for a full header yet

        const headerBytes = this.buffer.slice(0, headerEndIndex);
        const headerString = new TextDecoder().decode(headerBytes);
        const match = headerString.match(/Content-Length: (\d+)/);
        
        if (!match) {
            // Invalid header, discard this part of the buffer and continue
            console.error('Invalid LSP header, discarding:', headerString);
            this.buffer = this.buffer.slice(headerEndIndex);
            continue;
        }

        const contentLength = parseInt(match[1], 10);
        
        if (this.buffer.length < headerEndIndex + contentLength) break; // Not enough data for the full message body

        const bodyBytes = this.buffer.slice(headerEndIndex, headerEndIndex + contentLength);
        this.buffer = this.buffer.slice(headerEndIndex + contentLength);

        try {
            const bodyString = new TextDecoder().decode(bodyBytes);
            const message: LspMessage = JSON.parse(bodyString);
            this.handleMessage(message);
        } catch (e) {
            console.error('Failed to parse LSP message', e);
        }
    }
  }

  private handleMessage(message: LspMessage) {
    if (message.id !== undefined) {
        // Response to a request
        if (this.pendingRequests.has(message.id)) {
            const { resolve, reject } = this.pendingRequests.get(message.id)!;
            this.pendingRequests.delete(message.id);
            if (message.error) {
                reject(message.error);
            } else {
                resolve(message.result);
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

  public sendRequest(method: string, params: any): Promise<any> {
    const id = this.requestId++;
    const request = {
      jsonrpc: '2.0',
      id,
      method,
      params,
    };
    
    this.send(request);
    
    return new Promise((resolve, reject) => {
      this.pendingRequests.set(id, { resolve, reject });
    });
  }

  public sendNotification(method: string, params: any) {
    const notification = {
      jsonrpc: '2.0',
      method,
      params,
    };
    this.send(notification);
  }

  private send(message: any) {
    const json = JSON.stringify(message);
    const encoder = new TextEncoder();
    const encoded = encoder.encode(json);
    const payload = `Content-Length: ${encoded.length}\r\n\r\n${json}`;
    // @ts-ignore
    window.ipcRenderer.send('lsp-input', payload);
  }

  public onNotification(method: string, handler: (params: any) => void) {
    this.notificationHandlers.set(method, handler);
  }
}

// Export a singleton instance for backward compatibility
// Note: This instance might not be fully initialized if used before Editor.tsx sets it up
// Ideally, we should refactor to dependency injection or context
export const lspClient = new LspClient((window as any).ipcRenderer || {}, 'lex');
