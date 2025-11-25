Lex Neovim Plugin

Neovim plugin providing LSP integration, semantic highlighting, and editor support for Lex documents.

1. Quick Start

    Install the lex-lsp binary and add this plugin to your Neovim config:
        require("lex").setup()
    :: lua

    For local development with a custom binary:
        require("lex").setup({
            cmd = { "/path/to/lex-lsp" },
        })
    :: lua

2. Configuration Options

    All Options:
        require("lex").setup({
            -- Command to start the LSP server (array)
            cmd = { "lex-lsp" },

            -- Or specify a version to auto-download from GitHub releases
            lex_lsp_version = "v0.1.14",

            -- Debug theme: use exact colors from lex-light.json
            -- Useful for development/testing visual appearance
            debug_theme = false,

            -- Pass additional config to lspconfig
            lsp_config = {
                on_attach = function(client, bufnr)
                    -- Your custom on_attach
                end,
            },
        })
    :: lua

3. Syntax Highlighting

    The plugin uses LSP semantic tokens for all highlighting. No traditional
    Vim syntax files are used.

    3.1. Three-Intensity Model

        Lex highlights use three intensity levels that respect your colorscheme:

        NORMAL:
            Content text that readers focus on. Uses theme's foreground color.
            We control typography only (bold, italic).
            - Session title text (bold)
            - Inline strong/emphasis (bold/italic)
            - Definition subjects (italic)
            - Code and verbatim content

        MUTED:
            Structural elements for navigation. Dimmer than normal.
            Links to `@lex.muted` -> `@punctuation` by default.
            - Session markers (1., 1.1., etc.)
            - List markers (-, 1., etc.)
            - References ([link], [@cite], [^note])

        FAINT:
            Meta-information. Faded like comments.
            Links to `@lex.faint` -> `@comment` by default.
            - Annotations (:: note ::)
            - Verbatim metadata (language, attributes)
            - Inline syntax markers (*, _, `, #, [])

    3.2. Customizing Intensity

        Override the base highlight groups in your config:
            -- Make muted text more visible
            vim.api.nvim_set_hl(0, "@lex.muted", { link = "NonText" })

            -- Make faint text even fainter
            vim.api.nvim_set_hl(0, "@lex.faint", { link = "Conceal" })
        :: lua

        Or override specific token types:
            -- Custom color for references
            vim.api.nvim_set_hl(0, "@lsp.type.Reference", {
                fg = "#5588ff",
                underline = true
            })
        :: lua

    3.3. Debug Theme

        Enable `debug_theme = true` to use exact colors from lex-light.json.
        This is useful for development and visual regression testing.

4. Commands

    :LexDebugToken:
        Inspect the semantic token under cursor. Shows:
        - Token type (e.g., InlineStrong, SessionMarker)
        - Highlight group and its definition
        - All highlights at cursor position
        Output is copied to clipboard for easy sharing.

5. Testing

    Interactive Testing:
        ./editors/nvim/test_phase1_interactive.sh
    :: shell

    Headless Test Suite:
        ./editors/nvim/test/run_suite.sh
    :: shell

    Test Production Highlights:
        nvim --headless -u test/minimal_init.lua \
            -l test/test_production_highlights.lua
    :: shell

6. Troubleshooting

    No Highlighting:
        - Check LSP attached: `:LspInfo`
        - Check filetype: `:set ft?` (should be "lex")
        - Check syntax disabled: `:set syntax?` (should be empty)
        - Run `:LexDebugToken` to inspect token at cursor

    Unexpected Colors:
        - Plugin respects your colorscheme by default
        - Try `debug_theme = true` to see reference colors
        - Check your theme defines @punctuation and @comment

    Built-in lex.vim Conflict:
        Neovim includes syntax for Unix lex/flex tool. The plugin
        automatically disables this by setting `syntax = ""`.

7. Documentation

    Plugin Source:
        - `lua/lex/init.lua` - Main plugin with inline docs
        - `lua/lex/binary.lua` - Binary manager for auto-download

    Architecture:
        - `docs/dev/nvim-fasttrack.lex` - Quick overview
        - `docs/dev/guides/lsp-plugins.lex` - Detailed design

    Highlighting Model:
        - `lex-analysis/src/semantic_tokens.rs` - Token definitions
        - `editors/vscode/themes/lex-light.json` - Reference colors
        - `editors/vscode/test/fixtures/.../semantic-tokens.lex` - Visual test

8. Packaging

    Build release archive:
        ./editors/nvim/scripts/build_plugin.sh --version v0.1.14
    :: shell
