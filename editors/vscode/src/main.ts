import * as vscode from 'vscode';
import { LanguageClient } from 'vscode-languageclient/node';
import {
  buildLexExtensionConfig,
  LEX_CONFIGURATION_SECTION,
  LSP_BINARY_SETTING
} from './config.js';
import { createLexClient } from './client.js';

let client: LanguageClient | undefined;

export async function activate(context: vscode.ExtensionContext) {
  const config = vscode.workspace.getConfiguration(LEX_CONFIGURATION_SECTION);
  const configuredLspPath = config.get<string | null>(LSP_BINARY_SETTING, null);
  const resolvedConfig = buildLexExtensionConfig(
    context.extensionUri.fsPath,
    configuredLspPath
  );

  client = createLexClient(resolvedConfig.lspBinaryPath, context);
  context.subscriptions.push(client);
  await client.start();
}

export async function deactivate(): Promise<void> {
  if (!client) {
    return;
  }

  await client.stop();
  client = undefined;
}
