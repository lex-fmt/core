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
    There is not much configurations for lex:
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
    Override base intensity groups to change all elements at that level:
        vim.api.nvim_set_hl(0, "@lex.muted", { fg = "#666666" })
        vim.api.nvim_set_hl(0, "@lex.faint", { fg = "#999999" })
    :: lua

    The list of groups and highlights:

    Base intensity groups (override these to change all elements at that level):
        @lex.normal   - Full contrast content text
        @lex.muted    - Medium gray structural elements
        @lex.faint    - Light gray meta-information
        @lex.faintest - Barely visible syntax markers

    Content tokens (normal intensity):
        @lsp.type.SessionTitleText   - Session heading text (bold)
        @lsp.type.DefinitionSubject  - Term being defined (italic)
        @lsp.type.DefinitionContent  - Definition body text
        @lsp.type.ListItemText       - Text after list markers
        @lsp.type.InlineStrong       - Bold text between *markers*
        @lsp.type.InlineEmphasis     - Italic text between _markers_
        @lsp.type.InlineCode         - Code between `markers`
        @lsp.type.InlineMath         - Math between #markers#
        @lsp.type.VerbatimContent    - Code block content

    Structural tokens (muted intensity):
        @lsp.type.SessionTitle       - Full session header line
        @lsp.type.SessionMarker      - The 1., 1.1., A. prefix (italic)
        @lsp.type.ListMarker         - Bullet or number prefix (italic)
        @lsp.type.Reference          - Cross-references [like this]
        @lsp.type.ReferenceCitation  - Citations [@like this]
        @lsp.type.ReferenceFootnote  - Footnotes [^like this]

    Meta tokens (faint intensity):
        @lsp.type.AnnotationLabel     - The :: label :: part
        @lsp.type.AnnotationParameter - Parameters like key=value
        @lsp.type.AnnotationContent   - Content inside annotations
        @lsp.type.VerbatimSubject     - Label before :: in code blocks
        @lsp.type.VerbatimLanguage    - Language identifier after ::
        @lsp.type.VerbatimAttribute   - Attributes like language=bash

    Marker tokens (faintest intensity):
        @lsp.type.InlineMarker_strong_start    - Opening *
        @lsp.type.InlineMarker_strong_end      - Closing *
        @lsp.type.InlineMarker_emphasis_start  - Opening _
        @lsp.type.InlineMarker_emphasis_end    - Closing _
        @lsp.type.InlineMarker_code_start      - Opening `
        @lsp.type.InlineMarker_code_end        - Closing `
        @lsp.type.InlineMarker_math_start      - Opening #
        @lsp.type.InlineMarker_math_end        - Closing #
        @lsp.type.InlineMarker_ref_start       - Opening [
        @lsp.type.InlineMarker_ref_end         - Closing ]


5. The Lex-lsp Binrary

    By default the Lex plugin will download and install the lex-lsp binary post install or on updates, when needed. If you'd rather use another version you can specify a version by: 

      require("lex").setup({
            -- Or use a custom binary path
            cmd = { "/path/to/lex-lsp" },

        })
    :: lua


6. The Lex format

  Lex is designed to make writing structured techinical documents with no toolling a breeze. 
