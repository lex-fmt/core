# Lex VS Code Extension

This package hosts the VS Code extension for the Lex language. The extension is a thin wrapper around the `lex-lsp` binary and reuses the LSP for all language features. The TypeScript sources live under `src/` and compile to `out/`, and a small esbuild bundle under `dist/` is produced for marketplace delivery.

## Developer Setup

```
npm install
npm run build
npm test
```

Tests currently cover configuration helpers and a VS Code activation smoke test. Run individual suites with:

```
npm run test:unit
npm run test:integration
./test/run_suite.sh --format=simple
```

The integration runner uses `@vscode/test-electron` to launch a headless VS Code instance. During milestone 3 we set `LEX_VSCODE_SKIP_SERVER=1` (handled automatically by the runner) to bypass the Rust LSP binary until handshake coverage lands.
