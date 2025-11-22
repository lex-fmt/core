import path from 'node:path';

export const LEX_CONFIGURATION_SECTION = 'lex';
export const LSP_BINARY_SETTING = 'lspBinaryPath';
const DEFAULT_RELATIVE_BINARY = '../../target/debug/lex-lsp';

export interface LexExtensionConfig {
  lspBinaryPath: string;
}

export function defaultLspBinaryPath(extensionPath: string): string {
  return path.resolve(extensionPath, DEFAULT_RELATIVE_BINARY);
}

export function resolveLspBinaryPath(
  extensionPath: string,
  configuredPath?: string | null
): string {
  if (!configuredPath || configuredPath.trim() === '') {
    return defaultLspBinaryPath(extensionPath);
  }

  if (path.isAbsolute(configuredPath)) {
    return configuredPath;
  }

  return path.resolve(extensionPath, configuredPath);
}

export function buildLexExtensionConfig(
  extensionPath: string,
  configuredPath?: string | null
): LexExtensionConfig {
  return {
    lspBinaryPath: resolveLspBinaryPath(extensionPath, configuredPath)
  };
}
