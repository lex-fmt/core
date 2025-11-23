import * as vscode from 'vscode';
import { LanguageClient } from 'vscode-languageclient/node.js';
import {
  buildLexExtensionConfig,
  LEX_CONFIGURATION_SECTION,
  LSP_BINARY_SETTING
} from './config.js';
import { createLexClient } from './client.js';

export interface LexExtensionApi {
  clientReady(): Promise<void>;
}

let client: LanguageClient | undefined;
let resolveClientReady: (() => void) | undefined;
const clientReadyPromise = new Promise<void>(resolve => {
  resolveClientReady = resolve;
});

function signalClientReady(): void {
  resolveClientReady?.();
}

function shouldSkipLanguageClient(): boolean {
  return process.env.LEX_VSCODE_SKIP_SERVER === '1';
}

function createApi(): LexExtensionApi {
  return {
    clientReady: () => clientReadyPromise
  };
}

export async function activate(
  context: vscode.ExtensionContext
): Promise<LexExtensionApi> {
  const config = vscode.workspace.getConfiguration(LEX_CONFIGURATION_SECTION);
  const configuredLspPath = config.get<string | null>(LSP_BINARY_SETTING, null);
  const resolvedConfig = buildLexExtensionConfig(
    context.extensionUri.fsPath,
    configuredLspPath
  );

  if (shouldSkipLanguageClient()) {
    console.info('[lex] Skipping language client startup (LEX_VSCODE_SKIP_SERVER=1).');
    signalClientReady();
    return createApi();
  }

  client = createLexClient(resolvedConfig.lspBinaryPath, context);
  context.subscriptions.push(client);
  await client.start();
  signalClientReady();
  return createApi();
}

export async function deactivate(): Promise<void> {
  if (!client) {
    return;
  }

  await client.stop();
  client = undefined;
}
