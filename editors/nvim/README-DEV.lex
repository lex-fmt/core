Lex Neovim Plugin - Development Guide

This document covers the architecture and adaptation decisions for the Neovim plugin.
For user documentation, see README.lex.

1. Architecture

    The Neovim plugin follows the same architecture as VS Code: a thin client layer that delegates all logic to the lex-lsp server via LSP and workspace/executeCommand.

    Core principle: No Lua-side language logic. All features are driven by LSP.

2. VS Code to Neovim Adaptations

    Some VS Code features require adaptation for Neovim's terminal-based UI.

    2.1. File Pickers

        VS Code uses native OS file dialogs via `vscode.window.showOpenDialog()`.

        Neovim approach:
        - Primary: Telescope integration when available (de facto standard)
        - Fallback: `vim.ui.input()` for path entry (works everywhere)

        Detection is automatic:
            local has_telescope, telescope = pcall(require, 'telescope.builtin')
            if has_telescope then
                -- Use telescope.find_files with custom attach_mappings
            else
                -- Fall back to vim.ui.input with path completion
            end
        :: lua

    2.2. Export Commands (Markdown, HTML, PDF)

        VS Code shows a save dialog for export destination.

        Neovim approach:
        - Default output path: same directory and base name as source, with new extension
        - Example: `notes.lex` exports to `notes.md`, `notes.html`, or `notes.pdf`
        - User can override via command argument: `:LexExportPdf ~/exports/doc.pdf`

    2.3. Completion UI

        VS Code uses `vscode.languages.registerCompletionItemProvider()`.

        Neovim approach:
        - LSP-native: All completions come from lex-lsp via textDocument/completion
        - Works automatically with any completion plugin (nvim-cmp, coq, etc.)
        - Falls back to built-in `<C-x><C-o>` omnifunc if no plugin installed

        Path completion (@-trigger) is handled server-side in lex-lsp, so it works
        identically across all editors.

    2.4. Notifications

        VS Code uses `vscode.window.showInformationMessage()` etc.

        Neovim approach:
        - Uses `vim.notify()` which integrates with nvim-notify if installed
        - Falls back to built-in message display otherwise

    2.5. Live HTML Preview

        VS Code uses WebviewPanel for inline preview.

        Neovim: Not available in terminal.
        - Feature is intentionally omitted
        - Users can use `:LexExportHtml` + external browser if needed

3. Commands

    All commands use the `:Lex` prefix for discoverability.

    Navigation:
        :LexNextAnnotation      Jump to next annotation (]a mapping)
        :LexPrevAnnotation      Jump to previous annotation ([a mapping)

    Editing:
        :LexInsertAsset         Insert asset reference at cursor
        :LexInsertVerbatim      Insert verbatim block from file
        :LexResolveAnnotation   Resolve annotation at cursor
        :LexToggleAnnotations   Toggle all annotations resolved/unresolved

    Export/Import:
        :LexExportMarkdown [path]   Export to Markdown
        :LexExportHtml [path]       Export to HTML
        :LexExportPdf [path]        Export to PDF
        :LexImportMarkdown          Import current Markdown buffer to Lex

    Debug:
        :LexDebugToken          Show semantic token under cursor

4. Default Keymaps

    The plugin sets up these buffer-local mappings for .lex files:

        ]a      Go to next annotation
        [a      Go to previous annotation

    Additional mappings can be configured via lsp_config.on_attach.

5. Testing

    The test suite supports both minimal (no plugins) and full (Telescope + nvim-cmp) configurations.

    5.1. Running Tests

        cd editors/nvim
        ./test/run_suite.sh              # Run all tests
        ./test/run_suite.sh --minimal    # Run with bare Neovim (no plugins)
        ./test/run_suite.sh --full       # Run with Telescope + nvim-cmp

    5.2. Test Configurations

        Minimal config (test/minimal_init.lua):
        - Only nvim-lspconfig required
        - Tests vim.ui.input fallbacks
        - Tests built-in completion

        Full config (test/full_init.lua):
        - Includes Telescope and nvim-cmp
        - Tests Telescope file picker integration
        - Tests nvim-cmp completion display

    5.3. Test Structure

        test/
        ├── minimal_init.lua          # Minimal Neovim config
        ├── full_init.lua             # Full plugin config
        ├── fixtures/                 # Test documents
        │   ├── example.lex
        │   └── formatting.lex
        ├── test_*.lua                # Individual test files
        └── run_suite.sh              # Test runner

6. Plugin Structure

    lua/lex/
    ├── init.lua          # Main entry point, setup()
    ├── binary.lua        # Binary download/management
    ├── theme.lua         # Monochrome/native theme application
    ├── commands.lua      # User commands (export, import, etc.)
    ├── navigation.lua    # Annotation navigation
    └── debug.lua         # Debug utilities

7. LSP Execute Commands

    These commands are invoked via vim.lsp.buf.execute_command() or client:exec_cmd():

    Navigation:
        lex.next_annotation(uri, position) -> Location | null
        lex.previous_annotation(uri, position) -> Location | null

    Editing:
        lex.insert_asset(uri, position, assetPath) -> SnippetPayload
        lex.insert_verbatim(uri, position, filePath) -> SnippetPayload
        lex.resolve_annotation(uri, position) -> WorkspaceEdit | null
        lex.toggle_annotations(uri, position) -> WorkspaceEdit | null

    Conversion (via lex CLI, not LSP):
        lex convert --to markdown <file>
        lex convert --to html <file>
        lex convert --to pdf --output <out> <file>
