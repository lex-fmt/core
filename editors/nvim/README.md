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

- [x] Filetype detection for `.lex` files
- [x] LSP server integration
- [ ] Hover documentation
- [ ] Go to definition
- [ ] Symbol search
- [ ] Semantic token highlighting
- [ ] Diagnostics
