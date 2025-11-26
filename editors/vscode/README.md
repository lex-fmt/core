# Lex VS Code Extension

This package hosts the VS Code extension for the Lex language. The extension is a thin wrapper around the `lex-lsp` binary and reuses the LSP for all language features. The TypeScript sources live under `src/` and compile to `out/`, and a small esbuild bundle under `dist/` is produced for marketplace delivery.

## Developer Setup


```
cargo build --bin lex-lsp
cd editors/vscode
npm install
npm run build
npm test
```

Tests currently cover configuration helpers, VS Code activation, lex-lsp handshake, semantic tokens, document symbols, hover information, folding ranges, navigation (definitions/references), document links, and document formatting. Run individual suites with:

```
npm run test:unit
npm run test:integration
npm run test:vsix
./test/run_suite.sh --format=simple
```

The integration runner uses `@vscode/test-electron` to launch a headless VS Code instance and spins up the `lex-lsp` binary from `target/debug/lex-lsp`. If the binary is missing, the runner instructs you to build it first. The `npm run test:vsix` command packages the extension with `vsce`, installs the VSIX into a clean VS Code profile, opens a sample `.lex` document, and asserts that the shipped extension activates. Set `LEX_VSIX_KEEP_PROFILE=1` when running it if you want to inspect the cached VS Code profile after execution; otherwise it is cleaned automatically.

## Building the Extension Bundle

Use the helper script to build `lex-lsp` in release mode, copy it into the extension’s `resources/` directory, and bundle the TypeScript sources:

```
./editors/vscode/scripts/build_extension.sh
```

This produces `resources/lex-lsp` (the path shipped in the VSIX) and refreshes `dist/extension.js`. Follow up with `npx vsce package` if you want to produce a distributable VSIX.
