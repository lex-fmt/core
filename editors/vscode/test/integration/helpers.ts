import path from 'node:path';
import * as vscode from 'vscode';

export const TEST_DOCUMENT_PATH = 'documents/getting-started.lex';

export function requireWorkspaceFolder(): vscode.WorkspaceFolder {
  const folder = vscode.workspace.workspaceFolders?.[0];
  if (!folder) {
    throw new Error('Workspace folder should be available during integration tests');
  }

  return folder;
}

export async function openWorkspaceDocument(
  relativePath: string
): Promise<vscode.TextDocument> {
  const folder = requireWorkspaceFolder();
  const documentUri = vscode.Uri.file(
    path.join(folder.uri.fsPath, relativePath)
  );
  const document = await vscode.workspace.openTextDocument(documentUri);
  await vscode.window.showTextDocument(document);
  return document;
}

export async function closeAllEditors(): Promise<void> {
  await vscode.commands.executeCommand('workbench.action.closeAllEditors');
}
