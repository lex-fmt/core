# Lex VS Code Extension

This package hosts the VS Code extension for the Lex language. The extension is a thin wrapper around the `lex-lsp` binary and reuses the LSP for all language features. The TypeScript sources live under `src/` and compile to `out/`, and a small esbuild bundle under `dist/` is produced for marketplace delivery.

## Developer Setup

```
cargo build --bin lex-lsp
npm install
npm run build
npm test
```

Tests currently cover configuration helpers, VS Code activation, lex-lsp handshake, semantic tokens, document symbols, and hover information. Run individual suites with:

```
npm run test:unit
npm run test:integration
./test/run_suite.sh --format=simple
```

The integration runner uses `@vscode/test-electron` to launch a headless VS Code instance and spins up the `lex-lsp` binary from `target/debug/lex-lsp`. If the binary is missing, the runner instructs you to build it first.
