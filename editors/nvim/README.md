# Lex Neovim Plugin

This directory contains the Neovim plugin for the Lex language, providing LSP integration, syntax highlighting, and symbol support.

## Setup

### Prerequisites

- Neovim >= 0.9.0
- `lex-lsp` binary in your PATH (or configured explicitly)

### Minimal Configuration

The plugin requires a minimal Neovim setup with LSP support. We use a LazyVim-style configuration that includes:

- LSP client capabilities
- Treesitter for syntax highlighting
- Basic LSP UI components

#### Bootstrap Config

The plugin can be tested headlessly using a minimal config located in `config/init.lua`. This config:

1. Sets up lazy.nvim package manager
2. Installs necessary LSP dependencies
3. Registers the Lex LSP server
4. Configures file type detection for `.lex` files

To use this config for testing:

```bash
# Run Neovim with isolated config
NVIM_APPNAME=lex-test nvim -u config/init.lua
```

### Headless Testing

The plugin ships with a Bats test suite that runs Neovim headlessly:

```bash
# Run all headless tests (pretty output)
./test/run_suite.sh --format=simple

# CI-friendly JUnit output
./test/run_suite.sh --format=junit
```

### Interactive Testing

To test LSP features interactively:

```bash
# Open a Lex file with the LSP enabled
NVIM_APPNAME=lex-test nvim -u editors/nvim/test/minimal_init.lua specs/v1/benchmark/050-lsp-fixture.lex
```

Then use these keybindings:
- `K` - Show hover information
- `:lua vim.lsp.buf.document_symbol()` - Show document outline
- `zc` / `zo` / `za` - Fold/unfold/toggle sections

### Debugging LSP Commands

Use the command runner to test any LSP command:

```bash
# Test hover at a specific position
./test/run_lsp_command.sh specs/v1/benchmark/050-lsp-fixture.lex 5,75 \
  'vim.lsp.buf_request_sync(0, "textDocument/hover", vim.lsp.util.make_position_params(), 2000)'

# Test document symbols
./test/run_lsp_command.sh specs/v1/benchmark/20-ideas-naked.lex 12,5 \
  'vim.lsp.buf_request_sync(0, "textDocument/documentSymbol", {textDocument = vim.lsp.util.make_text_document_params()}, 2000)'

# Test semantic tokens
./test/run_lsp_command.sh specs/v1/benchmark/050-lsp-fixture.lex 1,0 \
  'vim.lsp.buf_request_sync(0, "textDocument/semanticTokens/full", {textDocument = vim.lsp.util.make_text_document_params()}, 2000)'
```

## Development

The plugin follows standard Neovim plugin structure:

```
editors/nvim/
├── README.md           # This file
├── config/             # Minimal test configurations
│   └── init.lua        # Bootstrap config for testing
├── lua/
│   ├── lex/            # Plugin code
│   │   └── init.lua    # Main plugin entry point
│   └── themes/         # Light/Dark highlight themes
│       ├── lex-dark.lua
│       └── lex-light.lua
├── ftdetect/           # Filetype detection
│   └── lex.lua
└── test/               # Headless tests
    ├── lex_nvim_plugin.bats
    └── *.lua scripts used by Bats

## Themes

Lex uses dedicated themes that borrow the Markdown color palette to feel familiar while
highlighting the extra structural cues Lex provides. Themes live in `lua/themes/` and are
automatically selected based on `vim.o.background` (`lex-dark` for dark backgrounds,
`lex-light` otherwise). To override this behavior:

```lua
require("lex").setup({
  theme = "lex-dark",   -- or "lex-light"
})

-- Disable automatic theming entirely
-- require("lex").setup({ theme = false })
```
```

## Features

### Phase 1 (Complete)
- [x] Filetype detection for `.lex` files
- [x] LSP server integration
- [x] Hover documentation
- [x] Document symbols (outline/navigation)
- [x] Semantic token highlighting
- [x] Folding ranges

### Phase 2 (Planned)
- [ ] Go to definition
- [ ] Find references
- [ ] Document links

### Phase 3+ (Future)
- [ ] Document formatting
- [ ] Diagnostics
