-- Debug script to show document symbols output
-- Run with: nvim --headless -u test/minimal_init.lua -l test/debug_document_symbols.lua

local script_path = debug.getinfo(1).source:sub(2)
local plugin_dir = vim.fn.fnamemodify(script_path, ":p:h:h")
local project_root = vim.fn.fnamemodify(plugin_dir, ":h:h")

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
    print("✓ LSP attached: client=" .. client.name .. ", bufnr=" .. bufnr)
  end,
})

vim.filetype.add({
  extension = {
    lex = "lex",
  },
})

-- Open the file
local test_file = project_root .. "/specs/v1/benchmark/20-ideas-naked.lex"
print("Opening: " .. test_file)
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
  print("ERROR: LSP did not attach")
  vim.cmd("cquit 1")
end

-- Wait for LSP to be ready
vim.wait(500)

-- Set cursor position (line 12, column 5)
vim.api.nvim_win_set_cursor(0, {12, 5})
print("✓ Cursor set to line 12, column 5")

-- Request document symbols
print("\n=== Requesting document symbols ===")
local params = { textDocument = vim.lsp.util.make_text_document_params() }
local result = vim.lsp.buf_request_sync(0, 'textDocument/documentSymbol', params, 2000)

if not result or vim.tbl_isempty(result) then
  print("ERROR: No document symbols returned")
  vim.cmd("cquit 1")
end

print("\n=== DOCUMENT SYMBOLS OUTPUT ===\n")
for client_id, response in pairs(result) do
  if response.result and type(response.result) == "table" then
    print("Symbol count: " .. #response.result)
    print("\nAll symbols:")
    for i, symbol in ipairs(response.result) do
      local indent = ""
      local function print_symbol(sym, depth)
        local prefix = string.rep("  ", depth)
        print(prefix .. "- " .. sym.name .. " [" .. sym.kind .. "]")
        if sym.children then
          for _, child in ipairs(sym.children) do
            print_symbol(child, depth + 1)
          end
        end
      end
      print_symbol(symbol, 0)
    end
    print("\nFull structure (first 3 symbols):")
    for i = 1, math.min(3, #response.result) do
      print(vim.inspect(response.result[i]))
    end
  else
    print("ERROR: No symbols in response")
  end
end

vim.cmd("qall!")
