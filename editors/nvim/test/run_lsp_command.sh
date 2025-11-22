#!/usr/bin/env bash
# Run an LSP command on a Lex file at a specific cursor position
# Usage: ./run_lsp_command.sh <file.lex> <line,col> <lua_command>
# Example: ./run_lsp_command.sh specs/v1/benchmark/050-lsp-fixture.lex 5,48 "vim.lsp.buf.hover()"

set -e

if [ $# -ne 3 ]; then
    echo "Usage: $0 <file.lex> <line,col> <lua_command>"
    echo ""
    echo "Examples:"
    echo "  $0 specs/v1/benchmark/050-lsp-fixture.lex 5,48 'vim.lsp.buf.hover()'"
    echo "  $0 specs/v1/benchmark/050-lsp-fixture.lex 12,5 'vim.lsp.buf.document_symbol()'"
    echo "  $0 specs/v1/benchmark/050-lsp-fixture.lex 1,0 'vim.lsp.buf_request_sync(0, \"textDocument/semanticTokens/full\", {textDocument = vim.lsp.util.make_text_document_params()}, 2000)'"
    exit 1
fi

LEX_FILE="$1"
POSITION="$2"
LUA_COMMAND="$3"

# Parse position
IFS=',' read -r LINE COL <<< "$POSITION"

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Make file path absolute if relative
if [[ ! "$LEX_FILE" = /* ]]; then
    LEX_FILE="$PROJECT_ROOT/$LEX_FILE"
fi

if [ ! -f "$LEX_FILE" ]; then
    echo "Error: File not found: $LEX_FILE"
    exit 1
fi

# Create temp lua script
TEMP_SCRIPT=$(mktemp /tmp/nvim_lsp_XXXXXX.lua)
trap "rm -f $TEMP_SCRIPT" EXIT

cat > "$TEMP_SCRIPT" <<'EOF'
local project_root = "PROJECT_ROOT_PLACEHOLDER"

-- Add plugin directory to runtimepath so we can load theme modules
-- This allows require("themes.lex-dark") to work in the on_attach callback
local plugin_dir = project_root .. "/editors/nvim"
vim.opt.rtp:prepend(plugin_dir)

-- Enable colors and syntax highlighting for visual testing
vim.opt.termguicolors = true  -- Required for 24-bit colors in themes
vim.cmd("syntax on")           -- Required for syntax highlighting

-- Set up LSP
local lspconfig_ok, lspconfig = pcall(require, "lspconfig")
if not lspconfig_ok then
  print("ERROR: lspconfig not available")
  vim.cmd("cquit 1")
end

local configs = require("lspconfig.configs")
local lex_lsp_path = project_root .. "/target/debug/lex-lsp"

if vim.fn.filereadable(lex_lsp_path) ~= 1 then
  print("ERROR: lex-lsp binary not found at " .. lex_lsp_path)
  print("Please build with: cargo build --bin lex-lsp")
  vim.cmd("cquit 1")
end

-- Register lex LSP config
if not configs.lex_lsp then
  configs.lex_lsp = {
    default_config = {
      cmd = { lex_lsp_path },
      filetypes = { "lex" },
      root_dir = function(fname)
        return lspconfig.util.find_git_ancestor(fname) or vim.fn.getcwd()
      end,
      settings = {},
    },
  }
end

local lsp_attached = false

lspconfig.lex_lsp.setup({
  on_attach = function(client, bufnr)
    lsp_attached = true

    -- CRITICAL: Enable semantic token highlighting for .lex files
    -- .lex files rely on LSP semantic tokens (not traditional syntax files)
    -- Without this, you'll see either wrong highlighting (C syntax) or no highlighting
    if client.server_capabilities.semanticTokensProvider then
      vim.lsp.semantic_tokens.start(bufnr, client.id)
    end

    -- Apply theme to define highlight groups for semantic tokens
    -- The theme maps semantic token types to colors (@lsp.type.lexSessionTitle, etc.)
    local ok, theme = pcall(require, "themes.lex-dark")
    if ok and type(theme.apply) == "function" then
      theme.apply()
    end
  end,
})

vim.filetype.add({
  extension = {
    lex = "lex",
  },
})

-- Open the file
local test_file = "FILE_PLACEHOLDER"
vim.cmd("edit " .. test_file)

-- Wait for LSP to attach
local max_wait = 5000
local waited = 0
local wait_step = 100

while not lsp_attached and waited < max_wait do
  vim.wait(wait_step)
  waited = waited + wait_step
end

if not lsp_attached then
  print("ERROR: LSP did not attach within timeout")
  vim.cmd("cquit 1")
end

-- Wait for LSP to be ready
vim.wait(500)

-- Set cursor position
vim.api.nvim_win_set_cursor(0, {LINE_PLACEHOLDER, COL_PLACEHOLDER})

-- Run the command
print("File: " .. test_file)
print("Cursor: line " .. LINE_PLACEHOLDER .. ", col " .. COL_PLACEHOLDER)
print("")
print("=== COMMAND OUTPUT ===")
print("")

local result = COMMAND_PLACEHOLDER

-- Pretty print the result
if result then
  if type(result) == "table" then
    print(vim.inspect(result))
  else
    print(result)
  end
else
  print("(no result)")
end

vim.cmd("qall!")
EOF

# Replace placeholders
sed -i '' "s|PROJECT_ROOT_PLACEHOLDER|$PROJECT_ROOT|g" "$TEMP_SCRIPT"
sed -i '' "s|FILE_PLACEHOLDER|$LEX_FILE|g" "$TEMP_SCRIPT"
sed -i '' "s|LINE_PLACEHOLDER|$LINE|g" "$TEMP_SCRIPT"
sed -i '' "s|COL_PLACEHOLDER|$COL|g" "$TEMP_SCRIPT"
sed -i '' "s|COMMAND_PLACEHOLDER|$LUA_COMMAND|g" "$TEMP_SCRIPT"

# Run nvim with the temp script
NVIM_APPNAME=lex-test nvim --headless -u "$SCRIPT_DIR/minimal_init.lua" -l "$TEMP_SCRIPT" 2>&1
