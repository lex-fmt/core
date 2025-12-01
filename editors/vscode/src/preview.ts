/**
 * Live HTML preview for Lex documents.
 * See README.lex "Preview" section for full documentation.
 */
import * as vscode from 'vscode';
import { convertToHtml } from './commands.js';

const PREVIEW_VIEW_TYPE = 'lexPreview';
const DEBOUNCE_MS = 400;

interface PreviewState {
  panel: vscode.WebviewPanel;
  sourceUri: vscode.Uri;
  disposables: vscode.Disposable[];
}

const activePreviews = new Map<string, PreviewState>();

function getPreviewTitle(uri: vscode.Uri): string {
  const filename = uri.path.split('/').pop() || 'Untitled';
  return `Preview: ${filename}`;
}

function debounce<T extends (...args: unknown[]) => void>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout> | undefined;
  return (...args: Parameters<T>) => {
    if (timeoutId) {
      clearTimeout(timeoutId);
    }
    timeoutId = setTimeout(() => fn(...args), delay);
  };
}

function wrapHtmlForWebview(html: string): string {
  // Wrap the converted HTML in a basic document structure with styling
  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline';">
  <style>
    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
      line-height: 1.6;
      padding: 20px;
      max-width: 800px;
      margin: 0 auto;
      color: var(--vscode-editor-foreground);
      background-color: var(--vscode-editor-background);
    }
    h1, h2, h3, h4, h5, h6 {
      margin-top: 1.5em;
      margin-bottom: 0.5em;
    }
    pre, code {
      font-family: var(--vscode-editor-font-family), 'Menlo', 'Monaco', 'Courier New', monospace;
      background-color: var(--vscode-textBlockQuote-background);
      border-radius: 3px;
    }
    code {
      padding: 0.2em 0.4em;
    }
    pre {
      padding: 1em;
      overflow-x: auto;
    }
    pre code {
      padding: 0;
      background: none;
    }
    blockquote {
      border-left: 4px solid var(--vscode-textBlockQuote-border);
      margin: 1em 0;
      padding-left: 1em;
      color: var(--vscode-textBlockQuote-foreground);
    }
    a {
      color: var(--vscode-textLink-foreground);
    }
    a:hover {
      color: var(--vscode-textLink-activeForeground);
    }
    table {
      border-collapse: collapse;
      margin: 1em 0;
    }
    th, td {
      border: 1px solid var(--vscode-panel-border);
      padding: 0.5em 1em;
    }
    th {
      background-color: var(--vscode-textBlockQuote-background);
    }
    hr {
      border: none;
      border-top: 1px solid var(--vscode-panel-border);
      margin: 2em 0;
    }
    .error {
      color: var(--vscode-errorForeground);
      background-color: var(--vscode-inputValidation-errorBackground);
      border: 1px solid var(--vscode-inputValidation-errorBorder);
      padding: 1em;
      border-radius: 4px;
    }
  </style>
</head>
<body>
${html}
</body>
</html>`;
}

async function updatePreview(
  panel: vscode.WebviewPanel,
  document: vscode.TextDocument,
  cliBinaryPath: string
): Promise<void> {
  try {
    const html = await convertToHtml(document.getText(), cliBinaryPath);
    panel.webview.html = wrapHtmlForWebview(html);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    panel.webview.html = wrapHtmlForWebview(
      `<div class="error"><strong>Preview Error:</strong> ${escapeHtml(message)}</div>`
    );
  }
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function createPreview(
  document: vscode.TextDocument,
  cliBinaryPath: string,
  viewColumn: vscode.ViewColumn
): PreviewState {
  const panel = vscode.window.createWebviewPanel(
    PREVIEW_VIEW_TYPE,
    getPreviewTitle(document.uri),
    viewColumn,
    {
      enableScripts: false,
      retainContextWhenHidden: true
    }
  );

  const disposables: vscode.Disposable[] = [];

  // Initial render
  void updatePreview(panel, document, cliBinaryPath);

  // Debounced update function
  const debouncedUpdate = debounce(() => {
    // Find the current document (it may have been closed and reopened)
    const currentDoc = vscode.workspace.textDocuments.find(
      d => d.uri.toString() === document.uri.toString()
    );
    if (currentDoc) {
      void updatePreview(panel, currentDoc, cliBinaryPath);
    }
  }, DEBOUNCE_MS);

  // Listen for document changes
  disposables.push(
    vscode.workspace.onDidChangeTextDocument(e => {
      if (e.document.uri.toString() === document.uri.toString()) {
        debouncedUpdate();
      }
    })
  );

  // Update title if document is renamed
  disposables.push(
    vscode.workspace.onDidRenameFiles(e => {
      for (const file of e.files) {
        if (file.oldUri.toString() === document.uri.toString()) {
          panel.title = getPreviewTitle(file.newUri);
        }
      }
    })
  );

  // Clean up when panel is closed
  panel.onDidDispose(() => {
    for (const d of disposables) {
      d.dispose();
    }
    activePreviews.delete(document.uri.toString());
  });

  return { panel, sourceUri: document.uri, disposables };
}

export function createShowPreviewCommand(
  cliBinaryPath: string,
  beside: boolean
): () => void {
  return () => {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      vscode.window.showErrorMessage('No active editor.');
      return;
    }

    const document = editor.document;
    if (document.languageId !== 'lex') {
      vscode.window.showErrorMessage('Preview is only available for .lex files.');
      return;
    }

    const uriKey = document.uri.toString();
    const existing = activePreviews.get(uriKey);

    if (existing) {
      // Reveal existing preview
      existing.panel.reveal();
      return;
    }

    // Create new preview
    const viewColumn = beside
      ? vscode.ViewColumn.Beside
      : vscode.ViewColumn.Active;

    const state = createPreview(document, cliBinaryPath, viewColumn);
    activePreviews.set(uriKey, state);
  };
}

export function registerPreviewCommands(
  context: vscode.ExtensionContext,
  cliBinaryPath: string
): void {
  context.subscriptions.push(
    vscode.commands.registerCommand(
      'lex.showPreview',
      createShowPreviewCommand(cliBinaryPath, false)
    ),
    vscode.commands.registerCommand(
      'lex.showPreviewToSide',
      createShowPreviewCommand(cliBinaryPath, true)
    )
  );
}
