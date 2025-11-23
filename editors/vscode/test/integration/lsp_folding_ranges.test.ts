import assert from 'node:assert/strict';
import * as vscode from 'vscode';
import type { LexExtensionApi } from '../../src/main.js';
import { integrationTest } from './harness.js';
import {
  closeAllEditors,
  openWorkspaceDocument,
  SEMANTIC_TOKENS_DOCUMENT_PATH
} from './helpers.js';

integrationTest('provides folding ranges for sessions, lists, and verbatim blocks', async () => {
  const extension = vscode.extensions.getExtension<LexExtensionApi>('lex.lex-vscode');
  assert.ok(extension, 'Lex extension should be discoverable by VS Code');

  const api = await extension.activate();
  await api?.clientReady();

  const document = await openWorkspaceDocument(SEMANTIC_TOKENS_DOCUMENT_PATH);

  const ranges = await vscode.commands.executeCommand<vscode.FoldingRange[] | undefined>(
    'vscode.executeFoldingRangeProvider',
    document.uri
  );
  if (!ranges || ranges.length === 0) {
    throw new Error('Folding range request should return entries');
  }

  const spans = ranges.map(range => ({ start: range.start, end: range.end }));

  const sessionFold = spans.some(span => span.start <= 2 && span.end >= 8);
  assert.ok(sessionFold, 'Session should produce a folding range');

  const listFold = spans.some(span => span.start >= 12 && span.end > span.start);
  assert.ok(listFold, 'Lists should produce folding ranges');

  const verbatimFold = spans.some(span => span.start >= 18 && span.end > span.start);
  assert.ok(verbatimFold, 'Verbatim block should produce folding ranges');

  await closeAllEditors();
});
