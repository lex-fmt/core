import assert from 'node:assert/strict';
import test from 'node:test';
import path from 'node:path';
import {
  defaultLspBinaryPath,
  resolveLspBinaryPath
} from '../../src/config.js';

const fakeExtensionPath = path.join('/', 'tmp', 'lex-extension');

test('defaultLspBinaryPath resolves relative binary path inside workspace', () => {
  const expected = path.resolve(fakeExtensionPath, '../../target/debug/lex-lsp');
  assert.equal(defaultLspBinaryPath(fakeExtensionPath), expected);
});

test('resolveLspBinaryPath falls back to default when unset', () => {
  const resolved = resolveLspBinaryPath(fakeExtensionPath, undefined);
  assert.equal(resolved, defaultLspBinaryPath(fakeExtensionPath));
});

test('resolveLspBinaryPath leaves absolute paths untouched', () => {
  const absolute = '/usr/local/bin/lex-lsp';
  const resolved = resolveLspBinaryPath(fakeExtensionPath, absolute);
  assert.equal(resolved, absolute);
});

test('resolveLspBinaryPath resolves relative paths against extension root', () => {
  const relative = './bin/lex-lsp';
  const resolved = resolveLspBinaryPath(fakeExtensionPath, relative);
  assert.equal(resolved, path.resolve(fakeExtensionPath, relative));
});
