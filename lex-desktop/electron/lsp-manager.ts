import { spawn, ChildProcess } from 'child_process';
import { ipcMain, WebContents, app } from 'electron';
import * as fs from 'fs';
import * as path from 'path';

const LOG_FILE = '/tmp/lex-desktop-lsp.log';

function log(msg: string) {
  fs.appendFileSync(LOG_FILE, `${new Date().toISOString()} - ${msg}\n`);
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

    if (app.isPackaged) {
      // In production, the binary is in Resources/lex-lsp
      lspPath = path.join(process.resourcesPath, 'lex-lsp');
    } else {
      // Hardcoded path for dev environment
      lspPath = '/Users/adebert/h/lex/target/debug/lex-lsp';
    }

    this.lspProcess = spawn(lspPath, [], {
      env: process.env,
    });

    this.lspProcess.stdout?.on('data', (data: Buffer) => {
      const msg = data.toString();
      console.log(`LSP Output: ${msg}`);
      log(`LSP Output: ${msg}`);
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
