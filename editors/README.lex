# Editor Tooling

    As part of lex's value proposition, we'll be building two high quality editor plugins for VSCode and Neovim.

    While not entirely possible, we will keep feature parity between both and share as much code as possible. This unfolds as a few guiding principles:

    1. Editor specific code is only about the interaction model: how a command is requested, how a buffer behaves. But all logic must be outside of the plugin code.
    2. That shared logic is to reside in the rust codebase, in the various crates. In common they will all be channeled by the LSP execute command protocol, which allows arbitrary commands to be invoked from LSP clients, which both plugins are.
    3. Whatever can be addressed by the regular LSP calls (semantic tokens, hover, symbols) should be so, and the execute command reserved for non standard operations.
    4. As much as possible, we want to mirror the aspects that can be configured by users, regardless of the configuration having different forms.
    5. We have a two prong automated testing model: unit tests for the logic in the rust code, and shallow integration tests, e2e running on top of the actual editors and the plugins. These should only test the integration (things get called and returned and processed as expected, not testing many inputs and variants).


    We will fully develop the initial version of plugins for Neovim and VSCode.
    While in the future these will be best served by dedicated repos, for now, as we are iterating over various layers in binaries, libs and the plugin themselves, they'll be colocated in the master lex repo.

    The design's goal is to have all logic-heavy lifting done in common rust code, and the plugins themselves being thin wrappers for each editors UI / entry points and interaction models.

    Below the work in progress to be done, and at the documents very end the work already done.


2. Shared Architecture

    To avoid duplicating logic across plugins, we use the LSP `workspace/executeCommand` capability. This allows plugins to delegate complex tasks to the `lex-lsp` server.

    Mechanism:
        The server exposes a set of commands (e.g., `lex.echo`). Plugins invoke these commands using their editor's LSP client API.

    Usage:
        VSCode:
            Use `vscode.commands.executeCommand('lex.commandName', args)`.
        Neovim:
            Use `client:exec_cmd({ command = 'lex.commandName', arguments = args })` (or `vim.lsp.buf.execute_command` for older versions).

    Guarding Execute Commands:
        VS Code wraps every execute-command invocation in a helper that waits for the
        language client to finish starting before sending the request. This avoids the
        "connection got disposed" errors that can happen if a command is triggered as
        soon as the extension activates.


3. Features

    The following sections detail the feature groups organized by functionality.

## Feature Matrix
| Feature Group | Feature | LSP Method / Command | Implementation | Status | VS Code | Neovim |
| --- | --- | --- | --- | --- | --- | --- |
| **Syntax** | Syntax Highlighting | `textDocument/semanticTokens` | `lex-analysis/src/semantic_tokens.rs` | Done | Done | Done |
|  | Document Symbols | `textDocument/documentSymbol` | `lex-analysis/src/document_symbols.rs` | Done | Done | Done |
|  | Folding | `textDocument/foldingRange` | `lex-analysis/src/folding_ranges.rs` | Done | Done | Done |
|  | Hover | `textDocument/hover` | `lex-analysis/src/hover.rs` | Done | Done | Done |
|  | Semantic Tokens | `textDocument/semanticTokens/full` | `lex-analysis/src/semantic_tokens.rs` | Done | Done | Done |
|  | Diagnostics | `textDocument/publishDiagnostics` | (Postponed) | Base Rust |  |  |
| **Navigation** | Go to Definition | `textDocument/definition` | `lex-analysis/src/go_to_definition.rs` | Done | Done | Done |
|  | Find References | `textDocument/references` | `lex-analysis/src/references.rs` | Done | Done | Done |
|  | Document Links | `textDocument/documentLink` | `lex-lsp/src/features/document_links.rs` | Done | Done | Done |
|  | Next/Prev Annotation | `lex.next_annotation` | `lex-lsp/src/features/commands.rs` | Done | Done | Done |
| **Formatting** | Formatting | `textDocument/formatting` | `lex-lsp/src/features/formatting.rs` | Done | Done | Done |
|  | Range Formatting | `textDocument/rangeFormatting` | `lex-lsp/src/features/formatting.rs` | Done | Done | Done |
| **Editing** | Insert Asset | `lex.insert_asset` | `lex-lsp/src/features/commands.rs` | Done | Done | Done* |
|  | Insert Verbatim | `lex.insert_verbatim` | `lex-lsp/src/features/commands.rs` | Done | Done | Done* |
|  | Completion (Paths) | `textDocument/completion` | `lex-analysis/src/completion.rs` | Done | Done | Done |
|  | Completion (Refs) | `textDocument/completion` | `lex-analysis/src/completion.rs` | Done | Done | Done |
|  | Resolve Annotation | `lex.resolve_annotation` | `lex-lsp/src/features/commands.rs` | Done | Done | Done |
|  | Toggle Annotations | `lex.toggle_annotations` | `lex-lsp/src/features/commands.rs` | Done | Done | Done |
| **Interop** | Import Markdown | `lex.import` | `lex-lsp/src/features/commands.rs` | Done | Done | Done* |
|  | Export Markdown | `lex.export` | `lex-lsp/src/features/commands.rs` | Done | Done | Done* |
|  | Export HTML | `lex.export` | `lex-lsp/src/features/commands.rs` | Done | Done | Done* |
|  | Export PDF | `lex.export` | `lex-lsp/src/features/commands.rs` | Done | Done | Done* |
|  | Preview as HTML | (Client-side) | VS Code: `preview.ts` | Done | Done | N/A |

    * Neovim adaptations required - see `editors/nvim/README-DEV.lex` for details.

    Neovim Adaptations Summary:
    - File pickers: Telescope integration with vim.ui.input fallback
    - Export commands: Default to same directory with new extension
    - Preview: Not available in terminal; use export + external browser
    - Path completion: Moved to LSP for cross-editor support

4. Configuration

    The Lex formatter can be configured via `lex.toml` or editor settings. The available options are:

        | Option                       | Type      | Default  | Description                                                |
        | ---------------------------- | --------- | -------- | ---------------------------------------------------------- |
        | `session_blank_lines_before` | Integer   | `1`      | Number of blank lines before a session title.              |
        | `session_blank_lines_after`  | Integer   | `1`      | Number of blank lines after a session title.               |
        | `normalize_seq_markers`      | Boolean   | `true`   | Whether to normalize list markers (e.g. all bullets to -). |
        | `unordered_seq_marker`       | Character | `-`      | The character to use for unordered list markers.           |
        | `max_blank_lines`            | Integer   | `2`      | Maximum number of consecutive blank lines allowed.         |
        | `indent_string`              | String    | `"    "` | String to use for indentation (usually 4 spaces).          |
        | `preserve_trailing_blanks`   | Boolean   | `false`  | Whether to preserve trailing blank lines at end of doc.    |
        | `normalize_verbatim_markers` | Boolean   | `true`   | Whether to normalize verbatim markers to `::`.             |
    :: doc.table

5. Implementation Status

    For a detailed breakdown of the required Rust APIs, their signatures, and current implementation status, please refer to @editors/required-apis.lex.
