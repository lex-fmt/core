Lex Neovim Plugin

Neovim plugin for reading and writing Lex, the plain-text format for ideas, documents.


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

            -- Additional lspconfig options
            lsp_config = {
                on_attach = function(client, bufnr)
                    -- Your custom on_attach
                end,
            },
        })
    :: lua

3. Themes

    Lex is a strong opinionated format about legibility and ergonomics, and breaks common expectations by setting it's own theme. Lex's mission is to make reading and writing plain text, richly formatted documents, with less clutter.

    Colorful syntax highlighting is critical on languages with significant keywors and syntax. As Lex is about avoiding all that it uses type style and only changes color for comments. Hence the recommended way to read and write Lex is the default monochrome theme (which adapts to light and dark modes). 


    In case you'd rather have your native theming, this can be enabled by: 
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

    The list of groups and highlights: 


  5. The Lex-lsp Binrary

    By default the Lex plugin will download and install the lex-lsp binary post install or on updates, when needed. If you'd rather use another version you can specify a version by: 

    require("lex").setup({
            -- Or use a custom binary path
            cmd = { "/path/to/lex-lsp" },

        })
    :: lua



