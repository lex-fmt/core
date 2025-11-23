import assert from 'node:assert/strict';
import * as vscode from 'vscode';
import type { LexExtensionApi } from '../../src/main.js';
import { integrationTest } from './harness.js';
import {
  closeAllEditors,
  FORMATTING_DOCUMENT_PATH,
  openWorkspaceDocument
} from './helpers.js';

integrationTest('applies whole-document formatting fixes indentation', async () => {
  const extension = vscode.extensions.getExtension<LexExtensionApi>('lex.lex-vscode');
  assert.ok(extension, 'Lex extension should be discoverable by VS Code');

  const api = await extension.activate();
  await api?.clientReady();

  const document = await openWorkspaceDocument(FORMATTING_DOCUMENT_PATH);
  const edits = await vscode.commands.executeCommand<
    vscode.TextEdit[] | undefined
  >('vscode.executeFormatDocumentProvider', document.uri, {
    tabSize: 2,
    insertSpaces: true
  });

  assert.ok(edits && edits.length > 0, 'Formatting should produce edits');
  const changed = edits.some(edit => {
    const original = document.getText(edit.range);
    return original !== (edit.newText ?? '');
  });
  assert.ok(changed, 'Formatting should modify the document content');

  await closeAllEditors();
});
