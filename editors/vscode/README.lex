Lex VS Code Plugin
==================

Scope
-----
- Thin VS Code extension that shells out to `lex-lsp`; no TypeScript-side language logic.
- Language client + config live under `src/`; integration + fixture workspaces under `test/`.
- All editor features are exercised through LSP requests so behaviour matches the Neovim client and CLI.

Build & Test
------------
```
cargo build --bin lex-lsp
cargo build --bin lex
cd editors/vscode
npm ci
npm run lint && npm run build
npm test               # unit + VS Code integration
./test/run_suite.sh --format=simple
```
CI mirrors the same sequence via `.github/workflows/vscode-plugin.yml`.

Packaging
---------
- `./editors/vscode/scripts/build_extension.sh` builds `lex-lsp` in release mode, copies it into `editors/vscode/resources/lex-lsp`, installs npm dependencies, and runs `npm run bundle`.
- Run `npx vsce package` afterwards to create the VSIX.

Features Covered
----------------
- Activation + handshake with lex-lsp (configurable binary path).
- Semantic tokens, document symbols, hover info, folding ranges.
- Go to definition, find references, document links.
- Whole-document formatting (shares fixtures with lex-lsp tests).

Implementation Notes
--------------------
- `@vscode/test-electron` drives headless VS Code; fixtures live in `test/fixtures/sample-workspace/`.
- BATS wrapper runs `npm test` for CI parity with the Neovim plugin.
- Extension exposes a tiny API (`clientReady`) so tests can await the LSP startup.
- Workspace `.vscode-test/` and `dist/`/`out/` remain gitignored; VSIX packaging stays manual for now.

Workflow
--------
- Each milestone/feature lands on its own branch (`editors/vscode/...`) with a conventional commit (e.g. `test(vscode-m5): cover document formatting`).
- Always verify `npm run lint`, `npm run build`, `npm test`, and `./test/run_suite.sh` before pushing.
