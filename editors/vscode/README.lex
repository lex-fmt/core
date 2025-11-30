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
CI mirrors the same sequence via `.github/workflows/test-vscode-plugin.yml`.

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
- Monochrome syntax theme for .lex files (adapts to light/dark mode).

Theming
-------
Lex applies a monochrome theme to .lex files that uses typography (bold, italic)
and grayscale intensity levels rather than colors. This reduces visual noise and
keeps focus on the content. The theme automatically adapts to VS Code's light or
dark mode.

See `src/theme.ts` for implementation details and the rationale behind this approach.

Import & Export Commands
------------------------
The extension provides commands to convert between Lex and other formats.
These commands shell out to the `lex` CLI binary and open results in new editors.

### Commands

`Lex: Export to Markdown`
  - Appears in: Command palette (when .lex file is active), editor context menu, editor title context menu
  - Behavior: Converts the active .lex document to Markdown and opens in a new untitled editor
  - Works with: Saved files and unsaved editors (uses buffer content)

`Lex: Import from Markdown`
  - Appears in: Command palette (when .md file is active), editor context menu, editor title context menu
  - Behavior: Converts the active Markdown document to Lex format and opens in a new untitled editor
  - Works with: Saved files and unsaved editors (uses buffer content)

### Configuration

`lex.cliBinaryPath`
  - Path to the `lex` CLI binary used for conversions
  - Default: `./resources/lex` (bundled with extension)
  - Can be absolute or relative to extension directory

### Implementation Details

Commands use temporary files for conversion (lex CLI doesn't support stdin). The flow:
1. Get active editor content (works for unsaved buffers)
2. Write content to a temp file with appropriate extension (.lex or .md)
3. Run `lex convert --to <target> <temp-file>` and capture stdout
4. Clean up temp files and open result as untitled document with appropriate language

Context menu visibility is controlled via `when` clauses:
- Export: shown only when `resourceExtname == .lex` or `editorLangId == lex`
- Import: shown only when `resourceExtname == .md` or `editorLangId == markdown`

See `src/commands.ts` for the full implementation.

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
