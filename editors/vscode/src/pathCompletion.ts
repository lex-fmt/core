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
    '/',
    '.'
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

    // Calculate the range from @ to cursor position (used for all items)
    const atPosition = position.character - atMatch[0].length;
    const replaceRange = new vscode.Range(
      position.line,
      atPosition,
      position.line,
      position.character
    );

    // Add ".." for navigating to parent directory if partial matches
    if ('..'.startsWith(partialLower) && baseDir !== '/') {
      const parentItem = new vscode.CompletionItem('..', vscode.CompletionItemKind.Folder);
      const pathPrefix = pathFragment.includes('/')
        ? pathFragment.substring(0, pathFragment.lastIndexOf('/') + 1)
        : '';
      parentItem.insertText = '@' + pathPrefix + '../';
      parentItem.range = replaceRange;
      parentItem.filterText = '@' + pathPrefix + '..';
      parentItem.sortText = '00..'; // Sort before other folders
      parentItem.detail = 'Parent directory';
      items.push(parentItem);
    }

    // Add "/" for absolute path when at root level (no path fragment yet)
    if (pathFragment === '' || '/'.startsWith(pathFragment)) {
      const rootItem = new vscode.CompletionItem('/', vscode.CompletionItemKind.Folder);
      rootItem.insertText = '@/';
      rootItem.range = replaceRange;
      rootItem.filterText = '@/';
      rootItem.sortText = '000/'; // Sort before .. and other folders
      rootItem.detail = 'Absolute path from root';
      items.push(rootItem);
    }

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

      // Build the full text that will replace @...
      let fullPath: string;
      if (pathFragment.startsWith('/')) {
        const dirPart = pathFragment.substring(0, pathFragment.lastIndexOf('/') + 1);
        fullPath = dirPart + entry.name;
      } else {
        const lastSlash = pathFragment.lastIndexOf('/');
        if (lastSlash === -1) {
          fullPath = entry.name;
        } else {
          fullPath = pathFragment.substring(0, lastSlash + 1) + entry.name;
        }
      }
      if (isDirectory) {
        fullPath += '/';
      }

      item.insertText = '@' + fullPath;
      item.range = replaceRange;
      // filterText must match what user types: @partial
      item.filterText = '@' + (pathFragment.includes('/') ? pathFragment.substring(0, pathFragment.lastIndexOf('/') + 1) : '') + entry.name;
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
