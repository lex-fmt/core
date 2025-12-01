import * as vscode from 'vscode';
import * as path from 'node:path';
import * as fs from 'node:fs';

export interface PathCompletionDiagnostics {
  pathSuggestTriggerCount: number;
  lastCompletionItems: string[];
}

let pathSuggestTriggerCount = 0;
let lastCompletionItems: string[] = [];

export function registerPathCompletion(context: vscode.ExtensionContext): void {
  const provider = vscode.languages.registerCompletionItemProvider(
    { language: 'lex', scheme: 'file' },
    new PathCompletionProvider(),
    '@',
    '/'
  );
  context.subscriptions.push(provider);

  const suggestTrigger = vscode.workspace.onDidChangeTextDocument(event => {
    if (!shouldTriggerSuggest(event)) {
      return;
    }

    pathSuggestTriggerCount += 1;
    setTimeout(() => {
      void vscode.commands.executeCommand('editor.action.triggerSuggest');
    }, 0);
  });
  context.subscriptions.push(suggestTrigger);
}

class PathCompletionProvider implements vscode.CompletionItemProvider {
  provideCompletionItems(
    document: vscode.TextDocument,
    position: vscode.Position
  ): vscode.CompletionItem[] | undefined {
    const lineText = document.lineAt(position.line).text;
    const textBeforeCursor = lineText.substring(0, position.character);

    // Find the @ that starts this path reference
    const atMatch = textBeforeCursor.match(/@([^\s]*)$/);
    if (!atMatch) {
      recordCompletionItems();
      return undefined;
    }

    const pathFragment = atMatch[1];
    const documentDir = path.dirname(document.uri.fsPath);

    // Determine base directory and partial name to complete
    let baseDir: string;
    let partial: string;

    if (pathFragment.startsWith('/')) {
      // Absolute path - use root
      const lastSlash = pathFragment.lastIndexOf('/');
      baseDir = pathFragment.substring(0, lastSlash) || '/';
      partial = pathFragment.substring(lastSlash + 1);
    } else {
      // Relative path - start from document directory
      const lastSlash = pathFragment.lastIndexOf('/');
      if (lastSlash === -1) {
        baseDir = documentDir;
        partial = pathFragment;
      } else {
        baseDir = path.join(documentDir, pathFragment.substring(0, lastSlash));
        partial = pathFragment.substring(lastSlash + 1);
      }
    }

    // Read directory contents
    let entries: fs.Dirent[];
    try {
      entries = fs.readdirSync(baseDir, { withFileTypes: true });
    } catch {
      recordCompletionItems();
      return undefined;
    }

    // Filter by partial match and create completion items
    const items: vscode.CompletionItem[] = [];
    const partialLower = partial.toLowerCase();

    for (const entry of entries) {
      // Skip hidden files unless user is explicitly typing them
      if (entry.name.startsWith('.') && !partial.startsWith('.')) {
        continue;
      }

      if (!entry.name.toLowerCase().startsWith(partialLower)) {
        continue;
      }

      const isDirectory = entry.isDirectory();
      const kind = isDirectory
        ? vscode.CompletionItemKind.Folder
        : vscode.CompletionItemKind.File;

      const item = new vscode.CompletionItem(entry.name, kind);

      // Calculate what to insert after the current path fragment
      let insertSuffix = entry.name.substring(partial.length);
      if (isDirectory) {
        insertSuffix += '/';
      }

      item.insertText = insertSuffix;
      item.filterText = entry.name;
      item.sortText = (isDirectory ? '0' : '1') + entry.name;

      items.push(item);
    }

    recordCompletionItems(items);
    return items;
  }
}

function shouldTriggerSuggest(event: vscode.TextDocumentChangeEvent): boolean {
  if (event.document.languageId !== 'lex') {
    return false;
  }

  const activeEditor = vscode.window.activeTextEditor;
  if (!activeEditor || activeEditor.document !== event.document) {
    return false;
  }

  const change = event.contentChanges[event.contentChanges.length - 1];
  if (!change) {
    return false;
  }

  return change.text === '@' && change.rangeLength === 0;
}

function recordCompletionItems(items?: vscode.CompletionItem[]): void {
  if (!items) {
    lastCompletionItems = [];
    return;
  }

  lastCompletionItems = items.map(item =>
    typeof item.label === 'string' ? item.label : item.label.label
  );
}

export function getPathCompletionDiagnostics(): PathCompletionDiagnostics {
  return {
    pathSuggestTriggerCount,
    lastCompletionItems
  };
}
