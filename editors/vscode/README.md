# Lex VS Code Extension

This package hosts the VS Code extension for the Lex language. The extension is a thin wrapper around the `lex-lsp` binary and reuses the LSP for all language features. The TypeScript sources live under `src/` and compile to `out/`, and a small esbuild bundle under `dist/` is produced for marketplace delivery.

## Developer Setup

```
npm install
npm run build
npm test
```

Tests currently cover configuration helpers and serve as infrastructure smoke tests.
