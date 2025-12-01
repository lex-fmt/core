/**
 * Import/Export commands for converting between Lex and other formats.
 * See README.lex "Import & Export Commands" section for full documentation.
 */
import * as vscode from 'vscode';
import { spawn } from 'node:child_process';
import { existsSync, mkdtempSync, writeFileSync, unlinkSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { ExecuteCommandRequest, LanguageClient } from 'vscode-languageclient/node.js';
import type {
  Location as LspLocation,
  WorkspaceEdit as LspWorkspaceEdit
} from 'vscode-languageserver-types';

export interface ConvertOptions {
  cliBinaryPath: string;
  fromFormat: 'lex' | 'markdown';
  toFormat: 'lex' | 'markdown' | 'html';
  targetLanguageId: string;
}

const FORMAT_EXTENSIONS: Record<string, string> = {
  lex: '.lex',
  markdown: '.md',
  html: '.html'
};

async function convertDocument(
  content: string,
  options: ConvertOptions
): Promise<string> {
  const { cliBinaryPath, fromFormat, toFormat } = options;

  if (!existsSync(cliBinaryPath)) {
    throw new Error(
      `Lex CLI binary not found at ${cliBinaryPath}. ` +
        'Configure lex.cliBinaryPath or ensure the bundled binary is available.'
    );
  }

  // Create a temporary file with the content
  const tmpDir = mkdtempSync(join(tmpdir(), 'lex-vscode-'));
  const inputExt = FORMAT_EXTENSIONS[fromFormat] || '.txt';
  const inputPath = join(tmpDir, `input${inputExt}`);

  try {
    writeFileSync(inputPath, content, 'utf-8');

    return await new Promise((resolve, reject) => {
      const args = ['convert', '--to', toFormat, inputPath];
      const proc = spawn(cliBinaryPath, args, {
        stdio: ['pipe', 'pipe', 'pipe']
      });

      let stdout = '';
      let stderr = '';

      proc.stdout.on('data', (data: Buffer) => {
        stdout += data.toString();
      });

      proc.stderr.on('data', (data: Buffer) => {
        stderr += data.toString();
      });

      proc.on('error', (err: Error) => {
        reject(new Error(`Failed to spawn lex CLI: ${err.message}`));
      });

      proc.on('close', (code: number | null) => {
        if (code !== 0) {
          reject(
            new Error(`lex convert failed (exit ${code}): ${stderr || 'unknown error'}`)
          );
          return;
        }
        resolve(stdout);
      });
    });
  } finally {
    // Cleanup temp files
    try {
      unlinkSync(inputPath);
      rmSync(tmpDir, { recursive: true, force: true });
    } catch {
      // Ignore cleanup errors
    }
  }
}

async function openConvertedDocument(
  content: string,
  languageId: string
): Promise<void> {
  const doc = await vscode.workspace.openTextDocument({
    content,
    language: languageId
  });
  await vscode.window.showTextDocument(doc);
}

/**
 * Convert a Lex document to PDF and write it to the specified output file.
 * PDF requires file-based output since it's binary.
 */
async function convertToPdfFile(
  content: string,
  cliBinaryPath: string,
  outputPath: string
): Promise<void> {
  if (!existsSync(cliBinaryPath)) {
    throw new Error(
      `Lex CLI binary not found at ${cliBinaryPath}. ` +
        'Configure lex.cliBinaryPath or ensure the bundled binary is available.'
    );
  }

  // Create a temporary file with the content
  const tmpDir = mkdtempSync(join(tmpdir(), 'lex-vscode-'));
  const inputPath = join(tmpDir, 'input.lex');

  try {
    writeFileSync(inputPath, content, 'utf-8');

    await new Promise<void>((resolve, reject) => {
      const args = ['convert', '--to', 'pdf', '--output', outputPath, inputPath];
      const proc = spawn(cliBinaryPath, args, {
        stdio: ['pipe', 'pipe', 'pipe']
      });

      let stderr = '';

      proc.stderr.on('data', (data: Buffer) => {
        stderr += data.toString();
      });

      proc.on('error', (err: Error) => {
        reject(new Error(`Failed to spawn lex CLI: ${err.message}`));
      });

      proc.on('close', (code: number | null) => {
        if (code !== 0) {
          reject(
            new Error(`lex convert failed (exit ${code}): ${stderr || 'unknown error'}`)
          );
          return;
        }
        resolve();
      });
    });
  } finally {
    // Cleanup temp files
    try {
      unlinkSync(inputPath);
      rmSync(tmpDir, { recursive: true, force: true });
    } catch {
      // Ignore cleanup errors
    }
  }
}

/**
 * Convert Lex content to HTML. Used by both export command and live preview.
 */
export async function convertToHtml(
  content: string,
  cliBinaryPath: string
): Promise<string> {
  return convertDocument(content, {
    cliBinaryPath,
    fromFormat: 'lex',
    toFormat: 'html',
    targetLanguageId: 'html'
  });
}

function getActiveEditorContent(): { content: string; languageId: string } | undefined {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    return undefined;
  }

  return {
    content: editor.document.getText(),
    languageId: editor.document.languageId
  };
}

export function createExportToMarkdownCommand(
  cliBinaryPath: string
): () => Promise<void> {
  return async () => {
    const editorInfo = getActiveEditorContent();
    if (!editorInfo) {
      vscode.window.showErrorMessage('No active editor with content to export.');
      return;
    }

    if (editorInfo.languageId !== 'lex') {
      vscode.window.showErrorMessage(
        'Export to Markdown is only available for .lex files.'
      );
      return;
    }

    try {
      const markdown = await convertDocument(editorInfo.content, {
        cliBinaryPath,
        fromFormat: 'lex',
        toFormat: 'markdown',
        targetLanguageId: 'markdown'
      });
      await openConvertedDocument(markdown, 'markdown');
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      vscode.window.showErrorMessage(`Export failed: ${message}`);
    }
  };
}

export function createImportFromMarkdownCommand(
  cliBinaryPath: string
): () => Promise<void> {
  return async () => {
    const editorInfo = getActiveEditorContent();
    if (!editorInfo) {
      vscode.window.showErrorMessage('No active editor with content to import.');
      return;
    }

    if (editorInfo.languageId !== 'markdown') {
      vscode.window.showErrorMessage(
        'Import from Markdown is only available for .md files.'
      );
      return;
    }

    try {
      const lex = await convertDocument(editorInfo.content, {
        cliBinaryPath,
        fromFormat: 'markdown',
        toFormat: 'lex',
        targetLanguageId: 'lex'
      });
      await openConvertedDocument(lex, 'lex');
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      vscode.window.showErrorMessage(`Import failed: ${message}`);
    }
  };
}

export function createExportToHtmlCommand(
  cliBinaryPath: string
): () => Promise<void> {
  return async () => {
    const editorInfo = getActiveEditorContent();
    if (!editorInfo) {
      vscode.window.showErrorMessage('No active editor with content to export.');
      return;
    }

    if (editorInfo.languageId !== 'lex') {
      vscode.window.showErrorMessage(
        'Export to HTML is only available for .lex files.'
      );
      return;
    }

    try {
      const html = await convertDocument(editorInfo.content, {
        cliBinaryPath,
        fromFormat: 'lex',
        toFormat: 'html',
        targetLanguageId: 'html'
      });
      await openConvertedDocument(html, 'html');
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      vscode.window.showErrorMessage(`Export failed: ${message}`);
    }
  };
}

export function createExportToPdfCommand(
  cliBinaryPath: string
): () => Promise<void> {
  return async () => {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      vscode.window.showErrorMessage('No active editor with content to export.');
      return;
    }

    if (editor.document.languageId !== 'lex') {
      vscode.window.showErrorMessage(
        'Export to PDF is only available for .lex files.'
      );
      return;
    }

    // Suggest a default filename based on the source file
    const sourceUri = editor.document.uri;
    const sourceName = sourceUri.path.split('/').pop() || 'document';
    const defaultName = sourceName.replace(/\.lex$/, '.pdf');

    // Show save dialog
    const saveUri = await vscode.window.showSaveDialog({
      defaultUri: vscode.Uri.file(
        join(sourceUri.fsPath, '..', defaultName)
      ),
      filters: {
        'PDF Documents': ['pdf']
      },
      title: 'Export to PDF'
    });

    if (!saveUri) {
      return; // User cancelled
    }

    try {
      await convertToPdfFile(
        editor.document.getText(),
        cliBinaryPath,
        saveUri.fsPath
      );
      vscode.window.showInformationMessage(`PDF exported to ${saveUri.fsPath}`);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      vscode.window.showErrorMessage(`Export failed: ${message}`);
    }
  };
}

export function registerCommands(
  context: vscode.ExtensionContext,
  cliBinaryPath: string,
  getClient: () => LanguageClient | undefined,
  waitForClientReady: () => Promise<void>
): void {
  context.subscriptions.push(
    vscode.commands.registerCommand(
      'lex.exportToMarkdown',
      createExportToMarkdownCommand(cliBinaryPath)
    ),
    vscode.commands.registerCommand(
      'lex.exportToHtml',
      createExportToHtmlCommand(cliBinaryPath)
    ),
    vscode.commands.registerCommand(
      'lex.exportToPdf',
      createExportToPdfCommand(cliBinaryPath)
    ),
    vscode.commands.registerCommand(
      'lex.importFromMarkdown',
      createImportFromMarkdownCommand(cliBinaryPath)
    ),
    vscode.commands.registerCommand('lex.insertAssetReference', (uri?: vscode.Uri) =>
      insertAssetReference(getClient, waitForClientReady, uri)
    ),
    vscode.commands.registerCommand('lex.insertVerbatimBlock', (uri?: vscode.Uri) =>
      insertVerbatimBlock(getClient, waitForClientReady, uri)
    ),
    vscode.commands.registerCommand('lex.goToNextAnnotation', () =>
      navigateAnnotation('lex.next_annotation', getClient, waitForClientReady)
    ),
    vscode.commands.registerCommand('lex.goToPreviousAnnotation', () =>
      navigateAnnotation('lex.previous_annotation', getClient, waitForClientReady)
    ),
    vscode.commands.registerCommand('lex.resolveAnnotation', () =>
      applyAnnotationEditCommand('lex.resolve_annotation', getClient, waitForClientReady)
    ),
    vscode.commands.registerCommand('lex.toggleAnnotationResolution', () =>
      applyAnnotationEditCommand('lex.toggle_annotations', getClient, waitForClientReady)
    )
  );
}

interface SnippetInsertionPayload {
  text: string;
  cursorOffset: number;
}

function isSnippetInsertionPayload(value: unknown): value is SnippetInsertionPayload {
  return (
    typeof value === 'object' &&
    value !== null &&
    typeof (value as { text?: unknown }).text === 'string' &&
    typeof (value as { cursorOffset?: unknown }).cursorOffset === 'number'
  );
}

async function getReadyClient(
  getClient: () => LanguageClient | undefined,
  waitForClientReady: () => Promise<void>
): Promise<LanguageClient> {
  await waitForClientReady();
  const client = getClient();
  if (!client) {
    throw new Error('Lex language server is not running.');
  }
  return client;
}

async function insertAssetReference(
  getClient: () => LanguageClient | undefined,
  waitForClientReady: () => Promise<void>,
  providedUri?: vscode.Uri
): Promise<void> {
  const fileUri = providedUri ?? (await pickWorkspaceFile('Select asset to insert'));
  if (!fileUri) {
    return;
  }

  await invokeInsertCommand('lex.insert_asset', fileUri, getClient, waitForClientReady);
}

async function insertVerbatimBlock(
  getClient: () => LanguageClient | undefined,
  waitForClientReady: () => Promise<void>,
  providedUri?: vscode.Uri
): Promise<void> {
  const fileUri = providedUri ?? (await pickWorkspaceFile('Select file to embed as verbatim block'));
  if (!fileUri) {
    return;
  }

  await invokeInsertCommand('lex.insert_verbatim', fileUri, getClient, waitForClientReady);
}

async function invokeInsertCommand(
  command: string,
  fileUri: vscode.Uri,
  getClient: () => LanguageClient | undefined,
  waitForClientReady: () => Promise<void>
): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    vscode.window.showErrorMessage('Open a Lex document before running this command.');
    return;
  }

  if (editor.document.languageId !== 'lex') {
    vscode.window.showErrorMessage('Insert commands are only available for .lex documents.');
    return;
  }

  const workspaceFolder = vscode.workspace.getWorkspaceFolder(editor.document.uri);
  if (!workspaceFolder) {
    vscode.window.showErrorMessage('Active document is not inside a workspace folder.');
    return;
  }

  const fileWorkspace = vscode.workspace.getWorkspaceFolder(fileUri);
  if (!fileWorkspace || fileWorkspace.uri.toString() !== workspaceFolder.uri.toString()) {
    vscode.window.showErrorMessage('Please select a file within the current workspace.');
    return;
  }

  try {
    const client = await getReadyClient(getClient, waitForClientReady);
    const position = editor.selection.active;
    const protocolPosition = client.code2ProtocolConverter.asPosition(position);
    const response = (await client.sendRequest(ExecuteCommandRequest.type, {
      command,
      arguments: [editor.document.uri.toString(), protocolPosition, fileUri.fsPath]
    })) as unknown;

    if (!response) {
      vscode.window.showErrorMessage('Command did not return a snippet payload.');
      return;
    }

    if (!isSnippetInsertionPayload(response)) {
      vscode.window.showErrorMessage('Command returned an invalid snippet payload.');
      return;
    }

    await insertSnippet(editor, position, response);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    vscode.window.showErrorMessage(`Failed to insert content: ${message}`);
  }
}

async function insertSnippet(
  editor: vscode.TextEditor,
  position: vscode.Position,
  payload: SnippetInsertionPayload
): Promise<void> {
  const prefix = position.line === 0 && position.character === 0 ? '' : '\n';
  const suffix = '\n';
  const textToInsert = `${prefix}${payload.text}${suffix}`;

  const inserted = await editor.edit(builder => builder.insert(position, textToInsert));
  if (!inserted) {
    throw new Error('Unable to update editor with snippet text.');
  }

  const baseOffset = editor.document.offsetAt(position);
  const cursorPosition = editor.document.positionAt(
    baseOffset + prefix.length + payload.cursorOffset
  );
  editor.selection = new vscode.Selection(cursorPosition, cursorPosition);
  editor.revealRange(new vscode.Range(cursorPosition, cursorPosition));
}

async function navigateAnnotation(
  lspCommand: string,
  getClient: () => LanguageClient | undefined,
  waitForClientReady: () => Promise<void>
): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    vscode.window.showErrorMessage('Open a Lex document before running this command.');
    return;
  }

  if (editor.document.languageId !== 'lex') {
    vscode.window.showErrorMessage('Annotation navigation works only inside Lex documents.');
    return;
  }

  try {
    const client = await getReadyClient(getClient, waitForClientReady);
    const protocolPosition = client.code2ProtocolConverter.asPosition(editor.selection.active);
    const response = (await client.sendRequest(ExecuteCommandRequest.type, {
      command: lspCommand,
      arguments: [editor.document.uri.toString(), protocolPosition]
    })) as unknown;

    if (!response) {
      vscode.window.showInformationMessage('No annotations were found in this document.');
      return;
    }

    const targetLocation = client.protocol2CodeConverter.asLocation(response as LspLocation);
    const targetDocument = await vscode.workspace.openTextDocument(targetLocation.uri);
    const targetEditor = await vscode.window.showTextDocument(targetDocument);
    const targetPosition = targetLocation.range.start;
    targetEditor.selection = new vscode.Selection(targetPosition, targetPosition);
    targetEditor.revealRange(targetLocation.range, vscode.TextEditorRevealType.InCenter);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    vscode.window.showErrorMessage(`Failed to navigate annotations: ${message}`);
  }
}

async function applyAnnotationEditCommand(
  lspCommand: string,
  getClient: () => LanguageClient | undefined,
  waitForClientReady: () => Promise<void>
): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    vscode.window.showErrorMessage('Open a Lex document before running this command.');
    return;
  }

  if (editor.document.languageId !== 'lex') {
    vscode.window.showErrorMessage('Annotation commands are only available for .lex files.');
    return;
  }

  try {
    const client = await getReadyClient(getClient, waitForClientReady);
    const protocolPosition = client.code2ProtocolConverter.asPosition(editor.selection.active);
    const response = (await client.sendRequest(ExecuteCommandRequest.type, {
      command: lspCommand,
      arguments: [editor.document.uri.toString(), protocolPosition]
    })) as unknown;

    if (!response) {
      vscode.window.showInformationMessage('No annotation was resolved at the current position.');
      return;
    }

    const workspaceEdit = await client.protocol2CodeConverter.asWorkspaceEdit(
      response as LspWorkspaceEdit
    );
    if (!workspaceEdit) {
      throw new Error('Language server returned an invalid workspace edit.');
    }

    const applied = await vscode.workspace.applyEdit(workspaceEdit);
    if (!applied) {
      throw new Error('Failed to apply workspace edit.');
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    vscode.window.showErrorMessage(`Failed to update annotation: ${message}`);
  }
}

async function pickWorkspaceFile(title: string): Promise<vscode.Uri | undefined> {
  const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri;
  const selection = await vscode.window.showOpenDialog({
    title,
    canSelectMany: false,
    canSelectFolders: false,
    canSelectFiles: true,
    defaultUri: workspaceRoot,
    openLabel: 'Select'
  });

  return selection?.[0];
}
