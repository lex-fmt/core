/**
 * Import/Export commands for converting between Lex and other formats.
 * See README.lex "Import & Export Commands" section for full documentation.
 */
import * as vscode from 'vscode';
import { spawn } from 'node:child_process';
import { existsSync, mkdtempSync, writeFileSync, unlinkSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

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

export function registerCommands(
  context: vscode.ExtensionContext,
  cliBinaryPath: string
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
      'lex.importFromMarkdown',
      createImportFromMarkdownCommand(cliBinaryPath)
    )
  );
}
