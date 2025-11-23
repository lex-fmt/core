import path from 'node:path';
import * as vscode from 'vscode';

export const TEST_DOCUMENT_PATH = 'documents/getting-started.lex';
export const SEMANTIC_TOKENS_DOCUMENT_PATH = 'documents/semantic-tokens.lex';
export const HOVER_DOCUMENT_PATH = 'documents/semantic-tokens.lex';
export const NAVIGATION_DOCUMENT_PATH = 'documents/semantic-tokens.lex';
export const FORMATTING_DOCUMENT_PATH = 'documents/formatting.lex';

export interface PositionMatch {
  line: number;
  character: number;
}

export function findPosition(
  document: vscode.TextDocument,
  searchText: string
): PositionMatch | undefined {
  const text = document.getText();
  const index = text.indexOf(searchText);
  if (index === -1) {
    return undefined;
  }

  const prefix = text.slice(0, index);
  const line = (prefix.match(/\n/g) || []).length;
  const lastLineBreak = prefix.lastIndexOf('\n');
  const character = index - (lastLineBreak + 1);
  return { line, character };
}

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
