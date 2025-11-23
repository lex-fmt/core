import assert from 'node:assert/strict';
import path from 'node:path';
import * as vscode from 'vscode';
import { integrationTest } from './harness.js';

integrationTest('activates extension and tags Lex documents', async () => {
  const extensionId = 'lex.lex-vscode';
  const extension = vscode.extensions.getExtension(extensionId);
  assert.ok(extension, `Extension ${extensionId} should be available`);

  await extension.activate();
  assert.equal(extension.isActive, true, 'Extension should activate without errors');

  const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
  assert.ok(workspaceFolder, 'Workspace folder should be available in integration tests');

  const documentUri = vscode.Uri.file(
    path.join(workspaceFolder.uri.fsPath, 'documents/getting-started.lex')
  );
  const document = await vscode.workspace.openTextDocument(documentUri);
  await vscode.window.showTextDocument(document);

  assert.equal(document.languageId, 'lex', 'Lex documents must use the lex language id');

  await vscode.commands.executeCommand('workbench.action.closeAllEditors');
});
