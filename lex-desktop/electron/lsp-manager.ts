import { spawn, ChildProcess } from 'child_process';
import { ipcMain, WebContents, app } from 'electron';
import * as fs from 'fs';
import * as path from 'path';

const DEV_WORKSPACE_ROOT =
  process.env.LEX_WORKSPACE_ROOT ?? path.resolve(process.cwd(), '..');

function resolveDevBinary(binaryName: string): string {
  const override = process.env.LEX_LSP_PATH;
  if (override) {
    return path.resolve(override);
  }
  return path.join(DEV_WORKSPACE_ROOT, 'target', 'debug', binaryName);
}

function resolveLogFile(): string {
  return path.join(app.getPath('userData'), 'lex-desktop-lsp.log');
}

function log(message: string) {
  const write = () => {
    const target = resolveLogFile();
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.appendFileSync(target, `${new Date().toISOString()} - ${message}\n`);
  };

  if (app.isReady()) {
    write();
  } else {
    app.whenReady().then(write).catch(() => {
      // no-op if app shuts down before logging
    });
  }
}

export class LspManager {
  private lspProcess: ChildProcess | null = null;
  private webContents: WebContents | null = null;

  constructor() {
    this.setupIpc();
    log('LspManager initialized');
  }

  setWebContents(webContents: WebContents) {
    this.webContents = webContents;
    // Clear reference when webContents is destroyed to prevent errors
    webContents.on('destroyed', () => {
      this.webContents = null;
    });
  }

  start() {
    if (this.lspProcess) return;

    let lspPath: string;

    const binaryName = process.platform === 'win32' ? 'lex-lsp.exe' : 'lex-lsp';

    if (app.isPackaged) {
      // In production, the binary is in Resources
      lspPath = path.join(process.resourcesPath, binaryName);
    } else {
      lspPath = resolveDevBinary(binaryName);
    }

    this.lspProcess = spawn(lspPath, [], {
      env: process.env,
    });

    this.lspProcess.stdout?.on('data', (data: Buffer) => {
      const msg = data.toString();
      // Truncate large LSP messages for cleaner logs
      const truncated = msg.length > 200 ? msg.slice(0, 200) + '... [truncated]' : msg;
      console.log(`LSP Output: ${truncated}`);
      log(`LSP Output: ${truncated}`);
      // Check if webContents exists and is not destroyed before sending
      if (this.webContents && !this.webContents.isDestroyed()) {
        this.webContents.send('lsp-output', data);
      }
    });

    this.lspProcess.stderr?.on('data', (data: Buffer) => {
      const msg = data.toString();
      console.error(`LSP Stderr: ${msg}`);
      log(`LSP Stderr: ${msg}`);
    });

    this.lspProcess.on('exit', (code) => {
      console.log(`LSP exited with code ${code}`);
      log(`LSP exited with code ${code}`);
      this.lspProcess = null;
    });

    this.lspProcess.on('error', (err) => {
      console.error('Failed to start LSP process:', err);
    });
  }

  setupIpc() {
    ipcMain.on('lsp-input', (_, data: string | Uint8Array) => {
      if (this.lspProcess && this.lspProcess.stdin) {
        this.lspProcess.stdin.write(data);
      }
    });
  }

  stop() {
    if (this.lspProcess) {
      this.lspProcess.kill();
      this.lspProcess = null;
    }
  }
}
