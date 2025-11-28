# Editor Tooling

    As part of lex's value proposition, we'll be building two high quality edtior plugins for VSCode and Neovim.

    While not entirely possible, we will keep feature parity between both and share as much code as possible. This unfolds as a few guiding principles:

    1. Editor specific codee is only about the interactoin model: how a command is requested, how a buffer behaves. But all logic must be outside of the plugin code. 
    2. That shared logic is to reside in the rust codebase, in the various crates. In common they will all be channeled but the LSP execute command protocol, which allows arbitrary commands to be invoked from lsp clientes, which both plugins are.
    3. Whatever can be addressed by the regular LSP calls (sematic tokens, hover, symbols) should be so, and the execute command reserved for non standard operiations.
    4. As much as possible, we want to mirror the aspects that can be configured by users, regardless of the configuration having diffrent forms.
    5. We have a two  prong automated testing model: unit tests for the logic in the rust code, and shallow integration tests , e2e running on top of the actual editors and the plugins. Thes should only test the integration (things get called and returned and processed as expected, not testing many inputs and variants).


    We will fully develop the initial version of plugins for Neovim and VSCode.
    While in the future these will be best served by dedicated repos, for now, as we are iterating over various layers in binaries, libs and the plugin themselves, they'll be colocated in the master lex repo.  

    The design's goal is to have all logic-heavy lifting done in common rust code, and the plugins themselves being thin wrappers for each editors ui / entry points and interaction models.

    Bellow the work in progress to be done, and at the documents very end the work already done.


2. Shared Architecture

    To avoid duplicating logic across plugins, we use the LSP `workspace/executeCommand` capability. This allows plugins to delegate complex tasks to the `lex-lsp` server.

    Mechanism:
        The server exposes a set of commands (e.g., `lex.echo`). Plugins invoke these commands using their editor's LSP client API.

    Usage:
        VSCode:
            Use `vscode.commands.executeCommand('lex.commandName', args)`.
        Neovim:
            Use `client:exec_cmd({ command = 'lex.commandName', arguments = args })` (or `vim.lsp.buf.execute_command` for older versions).


3. Feature Packs

    The following sections detail the feature packs, starting with the currently implemented set and followed by planned enhancements.


    3.1. Feature Pack 1: Minimal Featureset (Live)

        These are the initial launch features, focusing on core language support and navigation.

        | Category | Feature | Core Logic (Rust) | Neovim (Plugin) | VSCode (Extension) | Notes |
        |----------|---------|-------------------|-----------------|--------------------|-------|
        | Core | Syntax Highlighting | Tree-sitter Grammar (`lex-syntax`) | Tree-sitter Queries (`highlights.scm`) | TextMate Grammar (`lex.tmLanguage.json`) | VSCode may eventually adopt Tree-sitter. |
        | | Formatting | `lex-formatter` via LSP `textDocument/formatting` | `vim.lsp.buf.format` | `vscode.executeFormatDocumentProvider` | Configurable via `lex.toml`. |
        | | Diagnostics | `lex-lsp` (Minimal) | Built-in LSP Diagnostics | Problems View | Currently minimal (parsing errors). |
        | Navigation | Document Symbols | `lex-lsp` `textDocument/documentSymbol` | `vim.lsp.buf.document_symbol` | Outline View | |
        | | Hover Info | `lex-lsp` `textDocument/hover` | `vim.lsp.buf.hover` | Hover Widget | Shows element details/preview. |
        | | Folding | `lex-lsp` `textDocument/foldingRange` | `vim.lsp.buf.folding_range` / `nvim-ufo` | Folding Regions | Based on indentation/sections. |
        | | Go to Definition | `lex-lsp` `textDocument/definition` | `vim.lsp.buf.definition` (`gd`) | Go to Definition (`F12`) | For internal references/citations. |
        | | Find References | `lex-lsp` `textDocument/references` | `vim.lsp.buf.references` (`gr`) | Find All References | |
        | | Semantic Tokens | `lex-lsp` `textDocument/semanticTokens` | `vim.lsp.semantic_tokens` | Semantic Highlighting | Enhanced highlighting (e.g. titles). |


    3.2. Feature Pack 2: Document Handling

        Focuses on interoperability and converting Lex documents to other formats.

        | Category | Feature | Core Logic (Rust) | Neovim (Plugin) | VSCode (Extension) | Notes |
        |----------|---------|-------------------|-----------------|--------------------|-------|
        | Interop | Convert to Markdown | `lex-lsp` `workspace/executeCommand` (`lex.convert`) | `:LexConvert markdown` | `Lex: Convert to Markdown` | Opens result in new buffer (same path + .md). |
        | | Convert to HTML | `lex-lsp` `workspace/executeCommand` (`lex.convert`) | `:LexConvert html` | `Lex: Convert to HTML` | |
        | | Export to PDF | `lex-lsp` `workspace/executeCommand` (`lex.export`) | `:LexExport pdf` | `Lex: Export to PDF` | |


    3.3. Feature Pack 3: Editing Enhancements

        Advanced editing features to improve authoring efficiency.

        | Category | Feature | Core Logic (Rust) | Neovim (Plugin) | VSCode (Extension) | Notes |
        |----------|---------|-------------------|-----------------|--------------------|-------|
        | Completion | Path/URL/Citation | `lex-lsp` `textDocument/completion` | `nvim-cmp` source | IntelliSense Provider | Context-aware completion. |
        | Insertion | Insert Asset | `lex-lsp` `workspace/executeCommand` (`lex.insert_asset`) | `:LexInsertAsset` | `Lex: Insert Asset` | File picker integration. |
        | | Insert Verbatim | `lex-lsp` `workspace/executeCommand` (`lex.insert_verbatim`) | `:LexInsertVerbatim` | `Lex: Insert Verbatim` | Inserts file content as code block. |
        | Annotations | Iterate Annotations | `lex-lsp` `workspace/executeCommand` (`lex.next_annotation`) | `:LexNextAnnotation` | `Lex: Next Annotation` | Jump to next/prev annotation. |
        | | Resolve/Unresolve | `lex-lsp` `workspace/executeCommand` (`lex.resolve_annotation`) | `:LexResolve` | `Lex: Resolve Annotation` | Toggle status. |
        | | Show/Hide | `lex-lsp` `workspace/executeCommand` (`lex.toggle_annotations`) | `:LexToggleAnnotations` | `Lex: Toggle Annotations` | Virtual text / Decorations. |
        | Formatting | Indent on Paste | `lex-formatter` (logic) | `Paste` handler / `indentexpr` | `OnTypeFormatting` | Adjust indentation automatically. |
        | | Tab Shifting | `lex-formatter` (logic) | `<<` / `>>` | `Outdent` / `Indent` | Shift blocks/lists correctly. |
        | | List Ordering | `lex-formatter` (logic) | Auto-format on save/type | Auto-format | Fix nested list numbering. |
        | Comments | Smart Comments | N/A (Client Config) | `commentstring` | `language-configuration.json` | Toggle comments (`gc` / `Ctrl+/`). |


    3.4. Feature Pack 4: Publishing

        Workflows for publishing Lex documents.

        | Category | Feature | Core Logic (Rust) | Neovim (Plugin) | VSCode (Extension) | Notes |
        |----------|---------|-------------------|-----------------|--------------------|-------|
        | Publishing | Template Selection | `lex-cli` / `lex-lsp` | `:LexPublish` (UI Select) | `Lex: Publish` (QuickPick) | Choose output templates. |
        | | Image Sizing | `lex-core` (processing) | N/A | N/A | Part of build process. |
        | | Preview | `lex-lsp` (Live Preview) | `:LexPreview` | `Lex: Open Preview` | Side-by-side preview. |

    3.5. Feature Pack 5: Help & Documentation

        In-editor assistance for the Lex format.

        | Category | Feature | Core Logic (Rust) | Neovim (Plugin) | VSCode (Extension) | Notes |
        |----------|---------|-------------------|-----------------|--------------------|-------|
        | Help | Format Documentation | `lex-lsp` `workspace/executeCommand` (`lex.help`) | `:LexHelp` | `Lex: Show Help` | Show syntax guide/docs. |


4. Configuration

    The Lex formatter can be configured via `lex.toml` or editor settings. The available options are:
        | Option | Type | Default | Description |
        |--------|------|---------|-------------|
        | `session_blank_lines_before` | Integer | `1` | Number of blank lines before a session title. |
        | `session_blank_lines_after` | Integer | `1` | Number of blank lines after a session title. |
        | `normalize_seq_markers` | Boolean | `true` | Whether to normalize list markers (e.g. all bullets to `-`). |
        | `unordered_seq_marker` | Character | `-` | The character to use for unordered list markers. |
        | `max_blank_lines` | Integer | `2` | Maximum number of consecutive blank lines allowed. |
        | `indent_string` | String | `"    "` | String to use for indentation (usually 4 spaces). |
        | `preserve_trailing_blanks` | Boolean | `false` | Whether to preserve trailing blank lines at the end of the document. |
        | `normalize_verbatim_markers` | Boolean | `true` | Whether to normalize verbatim markers to `::`. |
    :: doc.table 

5. Implementation Status

    For a detailed breakdown of the required Rust APIs, their signatures, and current implementation status, please refer to @[editors/required-apis.lex].
