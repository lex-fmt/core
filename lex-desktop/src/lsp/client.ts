export type LspMessage = {
  jsonrpc: '2.0';
  id?: number | string;
  method?: string;
  result?: any;
  error?: any;
  params?: any;
};

export class LspClient {
  private requestId = 0;
  private pendingRequests = new Map<number | string, { resolve: (val: any) => void; reject: (err: any) => void }>();
  private notificationHandlers = new Map<string, (params: any) => void>();

  constructor() {
    // @ts-ignore
    window.ipcRenderer.on('lsp-output', (_event, data) => {
      this.handleMessage(data);
    });
  }

  private handleMessage(data: Uint8Array) {
    const text = new TextDecoder().decode(data);
    // The LSP output might contain headers (Content-Length: ...).
    // For simplicity in this manual adapter, we assume the server sends well-formed JSON-RPC
    // but we might need to strip headers if the server sends them.
    // lex-lsp uses tower-lsp which sends headers.
    
    // Simple header parsing strategy:
    // 1. Split by \r\n\r\n to separate headers from body.
    // 2. Parse body.
    // Note: This is a naive implementation. A robust one would buffer chunks.
    
    const parts = text.split('\r\n\r\n');
    if (parts.length < 2) return; // Incomplete or just headers?
    
    const body = parts[1];
    try {
      const message: LspMessage = JSON.parse(body);
      
      if (message.id !== undefined && this.pendingRequests.has(message.id)) {
        // Response to a request
        const { resolve, reject } = this.pendingRequests.get(message.id)!;
        this.pendingRequests.delete(message.id);
        if (message.error) {
          reject(message.error);
        } else {
          resolve(message.result);
        }
      } else if (message.method) {
        // Notification or Request from server
        const handler = this.notificationHandlers.get(message.method);
        if (handler) {
          handler(message.params);
        }
      }
    } catch (e) {
      console.error('Failed to parse LSP message', e);
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
    const payload = `Content-Length: ${json.length}\r\n\r\n${json}`;
    // @ts-ignore
    window.ipcRenderer.send('lsp-input', payload);
  }

  public onNotification(method: string, handler: (params: any) => void) {
    this.notificationHandlers.set(method, handler);
  }
}

export const lspClient = new LspClient();
