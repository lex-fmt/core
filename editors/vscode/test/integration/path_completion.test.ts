import assert from 'node:assert/strict';
import * as vscode from 'vscode';
import type { LexExtensionApi } from '../../src/main.js';
import { integrationTest } from './harness.js';
import {
  closeAllEditors,
  delay,
  openWorkspaceDocument,
  TEST_DOCUMENT_PATH,
  typeText
} from './helpers.js';

integrationTest('provides path completion items when typing @', async () => {
  const document = await openWorkspaceDocument(TEST_DOCUMENT_PATH);
  const editor = vscode.window.activeTextEditor;
  assert.ok(editor, 'Editor should be active');

  // Move to end of document and insert @ to trigger completion
  const lastLine = document.lineCount - 1;
  const lastChar = document.lineAt(lastLine).text.length;
  const endPosition = new vscode.Position(lastLine, lastChar);

  await editor.edit(editBuilder => {
    editBuilder.insert(endPosition, '\n@');
  });

  // Position cursor after the @
  const newPosition = new vscode.Position(lastLine + 1, 1);
  editor.selection = new vscode.Selection(newPosition, newPosition);

  // Trigger completion
  const completions = await vscode.commands.executeCommand<vscode.CompletionList>(
    'vscode.executeCompletionItemProvider',
    document.uri,
    newPosition,
    '@'
  );

  assert.ok(completions, 'Completions should be returned');
  assert.ok(completions.items.length > 0, 'Should return at least one completion item');

  // Verify we get file/folder completions from the documents directory
  const fileItems = completions.items.filter(
    item =>
      item.kind === vscode.CompletionItemKind.File ||
      item.kind === vscode.CompletionItemKind.Folder
  );
  assert.ok(fileItems.length > 0, 'Should include file or folder completions');

  // Check that known files from the test workspace appear
  const labels = completions.items.map(item =>
    typeof item.label === 'string' ? item.label : item.label.label
  );
  const hasLexFile = labels.some(label => label.endsWith('.lex') || label.endsWith('.md'));
  assert.ok(hasLexFile, 'Should include .lex or .md files from the workspace');

  await closeAllEditors();
});

integrationTest('shows the path completion suggest widget when typing @', async () => {
  const document = await openWorkspaceDocument(TEST_DOCUMENT_PATH);
  const editor = vscode.window.activeTextEditor;
  assert.ok(editor, 'Editor should be active');

  const lastLine = document.lineCount - 1;
  const lastChar = document.lineAt(lastLine).text.length;
  const endPosition = new vscode.Position(lastLine, lastChar);

  await editor.edit(editBuilder => {
    editBuilder.insert(endPosition, '\n');
  });

  const completionLine = lastLine + 1;
  const completionPosition = new vscode.Position(completionLine, 0);
  editor.selection = new vscode.Selection(completionPosition, completionPosition);

  const extension = vscode.extensions.getExtension<LexExtensionApi>('lex.lex-vscode');
  assert.ok(extension, 'Lex extension should be registered');
  const api = extension.isActive ? extension.exports : await extension.activate();
  assert.ok(api, 'Lex extension API should be available');
  await api.clientReady();

  const beforeTrigger = api.pathCompletionDiagnostics().pathSuggestTriggerCount;

  await typeText('@');
  const diags = await waitForPathSuggestTrigger(api, beforeTrigger, 2000);

  assert.equal(
    document.lineAt(completionLine).text,
    '@',
    'Typing @ should only insert the trigger character before suggestions fill in'
  );
  assert.ok(
    diags.lastCompletionItems.some(label => label.endsWith('.lex') || label.endsWith('.md')),
    'Path completion should surface workspace files after the widget appears'
  );

  await closeAllEditors();
});

async function waitForPathSuggestTrigger(
  api: LexExtensionApi,
  baseline: number,
  timeoutMs: number
): Promise<ReturnType<LexExtensionApi['pathCompletionDiagnostics']>> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const diags = api.pathCompletionDiagnostics();
    if (diags.pathSuggestTriggerCount > baseline) {
      return diags;
    }

    await delay(50);
  }

  throw new Error('Path completion suggest widget did not trigger after typing @');
}
