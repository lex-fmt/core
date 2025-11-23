import assert from 'node:assert/strict';
import * as vscode from 'vscode';
import type { LexExtensionApi } from '../../src/main.js';
import { integrationTest } from './harness.js';
import {
  closeAllEditors,
  openWorkspaceDocument,
  SEMANTIC_TOKENS_DOCUMENT_PATH
} from './helpers.js';

function flattenSymbols(symbols: vscode.DocumentSymbol[]): vscode.DocumentSymbol[] {
  const result: vscode.DocumentSymbol[] = [];
  for (const symbol of symbols) {
    result.push(symbol);
    if (symbol.children && symbol.children.length > 0) {
      result.push(...flattenSymbols(symbol.children));
    }
  }
  return result;
}

integrationTest('provides hierarchical document symbols', async () => {
  const extension = vscode.extensions.getExtension<LexExtensionApi>('lex.lex-vscode');
  assert.ok(extension, 'Lex extension should be discoverable by VS Code');

  const api = await extension.activate();
  await api?.clientReady();

  const document = await openWorkspaceDocument(SEMANTIC_TOKENS_DOCUMENT_PATH);
  const symbols = await vscode.commands.executeCommand<vscode.DocumentSymbol[] | undefined>(
    'vscode.executeDocumentSymbolProvider',
    document.uri
  );

  if (!symbols || symbols.length === 0) {
    throw new Error('Document symbols request should return entries');
  }

  assert.ok(symbols.length >= 2, 'Document should report multiple top-level symbols');

  const flattened = flattenSymbols(symbols);
  const titles = flattened.map(symbol => symbol.name);
  assert.ok(
    titles.some(name => name.includes('Intro')),
    'Outline should include the Intro session'
  );
  assert.ok(
    titles.some(name => name.includes('List')),
    'Outline should include list items'
  );
  assert.ok(
    titles.some(name => name.includes('Verbatim: CLI Example')),
    'Verbatim blocks should surface as document symbols'
  );

  assert.ok(
    flattened.some(symbol => symbol.children && symbol.children.length > 0),
    'At least one symbol should expose nested children'
  );

  await closeAllEditors();
});
