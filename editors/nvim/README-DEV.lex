Lex Neovim Plugin

Neovim plugin for editing Lex documents with LSP-powered syntax highlighting.

1. Installation

    1.1. With lazy.nvim

        {
            "arthur-debert/lex",
            ft = "lex",
            dependencies = { "neovim/nvim-lspconfig" },
            config = function()
                require("lex").setup()
            end,
        }
    :: lua

    1.2. With packer.nvim

        use {
            "arthur-debert/lex",
            requires = { "neovim/nvim-lspconfig" },
            config = function()
                require("lex").setup()
            end,
        }
    :: lua

    The plugin auto-downloads the lex-lsp binary on first use.

2. Configuration

        require("lex").setup({
            -- Theme: "monochrome" (default) or "native"
            theme = "monochrome",

            -- LSP binary version (auto-downloaded from GitHub)
            lex_lsp_version = "v0.1.14",

            -- Or use a custom binary path
            cmd = { "/path/to/lex-lsp" },

            -- Additional lspconfig options
            lsp_config = {
                on_attach = function(client, bufnr)
                    -- Your custom on_attach
                end,
            },
        })
    :: lua

3. Themes

    3.1. Monochrome (default)

        Grayscale highlighting designed to keep focus on your writing.
        Four intensity levels adapt to your dark/light mode setting:

        - Normal: Full contrast for content you read (black/white)
        - Muted: Medium gray for structural markers (1., -, etc.)
        - Faint: Light gray for meta-information (annotations)
        - Faintest: Barely visible for syntax markers (*, _, `)

    3.2. Native

        Uses your colorscheme's colors by linking to standard treesitter groups:

            require("lex").setup({
                theme = "native",
            })
        :: lua

        Mappings:
        - Headings -> @markup.heading
        - Bold/Italic -> @markup.strong, @markup.italic
        - Code -> @markup.raw
        - Links -> @markup.link
        - Annotations -> @comment

4. Customization

    Even with monochrome theme, you can override specific highlights:

        -- After lex.setup(), add your overrides:

        -- Change reference color to blue
        vim.api.nvim_set_hl(0, "@lsp.type.Reference", {
            fg = "#5588ff",
            underline = true
        })

        -- Make annotations green instead of gray
        vim.api.nvim_set_hl(0, "@lsp.type.AnnotationLabel", {
            fg = "#22aa22"
        })
    :: lua

    Override base intensity groups to change all elements at that level:

        vim.api.nvim_set_hl(0, "@lex.muted", { fg = "#666666" })
        vim.api.nvim_set_hl(0, "@lex.faint", { fg = "#999999" })
    :: lua

5. Commands

    :LexDebugToken
        Inspect the semantic token under cursor. Useful for debugging
        highlighting issues or finding the right group name to override.

6. Troubleshooting

    No highlighting at all:
        1. Check LSP is attached: `:LspInfo`
        2. Check filetype: `:set ft?` should show "lex"
        3. Check syntax disabled: `:set syntax?` should be empty
        4. Run `:LexDebugToken` on text that should be highlighted

    Colors don't match expected:
        1. Check your background setting: `:set background?`
        2. Monochrome theme adapts to dark/light mode
        3. Try `theme = "native"` to use your colorscheme

    Still seeing colored syntax (not monochrome):
        Built-in Neovim has a lex.vim for Unix lex/flex. The plugin
        disables this automatically, but some plugin managers may
        re-enable it. Check `:set syntax?` is empty.

7. Token Reference

    Content (normal intensity):
        SessionTitleText, DefinitionSubject, DefinitionContent,
        InlineStrong, InlineEmphasis, InlineCode, InlineMath,
        VerbatimContent, ListItemText

    Structure (muted intensity):
        SessionMarker, ListMarker, Reference, ReferenceCitation,
        ReferenceFootnote

    Meta (faint intensity):
        AnnotationLabel, AnnotationParameter, AnnotationContent,
        VerbatimSubject, VerbatimLanguage, VerbatimAttribute

    Markers (faintest intensity):
        InlineMarker_strong_start, InlineMarker_emphasis_start,
        InlineMarker_code_start, InlineMarker_math_start,
        InlineMarker_ref_start (and corresponding _end variants)
