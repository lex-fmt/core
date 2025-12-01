import assert from 'node:assert/strict';
import * as vscode from 'vscode';
import { integrationTest } from './harness.js';
import {
  closeAllEditors,
  openWorkspaceDocument,
  writeWorkspaceFile,
  removeWorkspacePath,
  delay
} from './helpers.js';

const TEMP_RESOLVE_PATH = 'documents/tmp-annotations/resolve.lex';
const RESOLVE_DOC = `1. Review\n\n    :: note ::\n        Pending\n    ::\n`;

integrationTest('resolves annotation at cursor', async () => {
  await writeWorkspaceFile(TEMP_RESOLVE_PATH, RESOLVE_DOC);
  const document = await openWorkspaceDocument(TEMP_RESOLVE_PATH);
  const editor = vscode.window.activeTextEditor;
  assert.ok(editor, 'Editor should be active');

  await delay(200);
  await vscode.commands.executeCommand('lex.goToNextAnnotation');
  assert.equal(editor.selection.active.line, 2, 'Navigation should land on annotation header');
  const headerPosition = editor.selection.active;
  const insideHeader = headerPosition.with({ character: headerPosition.character + 4 });
  editor.selection = new vscode.Selection(insideHeader, insideHeader);

  editor.selection = new vscode.Selection(new vscode.Position(2, 0), new vscode.Position(2, 0));
  await vscode.commands.executeCommand('lex.resolveAnnotation');
  const updatedText = document.getText();
  assert.ok(
    updatedText.includes(':: note status=resolved ::'),
    'Annotation header should include status=resolved'
  );

  await closeAllEditors();
  await removeWorkspacePath('documents/tmp-annotations');
});
