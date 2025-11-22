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

The plugin includes headless tests that can run without a UI:

```bash
# Run all headless tests
./test/run_headless.sh
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
│   └── lex/            # Plugin code
│       └── init.lua    # Main plugin entry point
├── ftdetect/           # Filetype detection
│   └── lex.lua
└── test/               # Headless tests
    └── run_headless.sh
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
