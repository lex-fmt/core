#!/usr/bin/env bash
# Visual test for syntax highlighting with LSP semantic tokens
# Opens a .lex file with LSP and semantic tokens enabled for visual inspection

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

LEX_FILE="${1:-specs/v1/benchmark/050-lsp-fixture.lex}"

# Make file path absolute if relative
if [[ ! "$LEX_FILE" = /* ]]; then
    LEX_FILE="$PROJECT_ROOT/$LEX_FILE"
fi

if [ ! -f "$LEX_FILE" ]; then
    echo "Error: File not found: $LEX_FILE"
    exit 1
fi

# Create temp lua script with LSP and semantic tokens enabled
TEMP_SCRIPT=$(mktemp /tmp/nvim_visual_XXXXXX.lua)
trap "rm -f $TEMP_SCRIPT" EXIT

cat > "$TEMP_SCRIPT" <<'EOF'
local project_root = "PROJECT_ROOT_PLACEHOLDER"
local plugin_dir = project_root .. "/editors/nvim"
vim.opt.rtp:prepend(plugin_dir)

-- Enable colors and syntax
vim.opt.termguicolors = true
vim.cmd("syntax on")

-- Set up LSP
local lspconfig = require("lspconfig")
local configs = require("lspconfig.configs")
local lex_lsp_path = project_root .. "/target/debug/lex-lsp"

if vim.fn.filereadable(lex_lsp_path) ~= 1 then
  print("ERROR: lex-lsp binary not found at " .. lex_lsp_path)
  vim.cmd("cquit 1")
end

if not configs.lex_lsp then
  configs.lex_lsp = {
    default_config = {
      cmd = { lex_lsp_path },
      filetypes = { "lex" },
      root_dir = function(fname)
        return lspconfig.util.find_git_ancestor(fname) or vim.fn.getcwd()
      end,
    },
  }
end

local lsp_attached = false

lspconfig.lex_lsp.setup({
  on_attach = function(client, bufnr)
    lsp_attached = true
    print("LSP attached to buffer " .. bufnr)

    -- Enable semantic token highlighting
    if client.server_capabilities.semanticTokensProvider then
      vim.lsp.semantic_tokens.start(bufnr, client.id)
      print("Semantic tokens enabled")
    else
      print("WARNING: No semantic token support")
    end

    -- Apply theme
    local ok, theme = pcall(require, "themes.lex-dark")
    if ok and type(theme.apply) == "function" then
      theme.apply()
      print("Theme applied")
    end
  end,
})

vim.filetype.add({ extension = { lex = "lex" } })

-- Open the file
local test_file = "FILE_PLACEHOLDER"
vim.cmd("edit " .. test_file)

-- Wait for LSP to attach
local max_wait = 5000
local waited = 0
while not lsp_attached and waited < max_wait do
  vim.wait(100)
  waited = waited + 100
end

if not lsp_attached then
  print("ERROR: LSP did not attach within timeout")
  vim.cmd("cquit 1")
end

-- Wait for semantic tokens to be applied
vim.wait(500)

print("")
print("=== Semantic Token Highlighting Status ===")
print("File: " .. test_file)
print("LSP attached: yes")

-- Check for semantic token namespace
local namespaces = vim.api.nvim_get_namespaces()
local has_semantic_ns = false
for name, _ in pairs(namespaces) do
  if name:match("semantic_tokens") then
    has_semantic_ns = true
    print("Semantic token namespace: " .. name)
    break
  end
end

if not has_semantic_ns then
  print("WARNING: Semantic token namespace not found")
end

print("")
print("Opening file for visual inspection...")
print("Check if you see:")
print("  - Session titles in different colors")
print("  - Bold/emphasized text highlighted")
print("  - Code blocks with syntax colors")
print("  - References and citations highlighted")
EOF

# Replace placeholders
sed -i '' "s|PROJECT_ROOT_PLACEHOLDER|$PROJECT_ROOT|g" "$TEMP_SCRIPT"
sed -i '' "s|FILE_PLACEHOLDER|$LEX_FILE|g" "$TEMP_SCRIPT"

# Run nvim with the temp script
echo "Opening $LEX_FILE with LSP and semantic tokens..."
echo "Press 'q' to quit when done inspecting"
echo ""

NVIM_APPNAME=lex-test nvim --noplugin -u "$SCRIPT_DIR/minimal_init.lua" -l "$TEMP_SCRIPT" 2>&1
