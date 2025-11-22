-- Test: LSP document symbols functionality
-- This test verifies that the LSP server can provide document symbols for outline/navigation
-- Run with: nvim --headless -u test/minimal_init.lua -l test/test_lsp_document_symbols.lua

-- Get paths
local script_path = debug.getinfo(1).source:sub(2)
local plugin_dir = vim.fn.fnamemodify(script_path, ":p:h:h")

-- Set up LSP
local lspconfig_ok, lspconfig = pcall(require, "lspconfig")
if not lspconfig_ok then
  print("TEST_FAILED: lspconfig not available")
  vim.cmd("cquit 1")
end

local configs = require("lspconfig.configs")

-- Find the lex-lsp binary
local project_root = vim.fn.fnamemodify(plugin_dir, ":h:h")
local lex_lsp_path = project_root .. "/target/debug/lex-lsp"

if vim.fn.filereadable(lex_lsp_path) ~= 1 then
  print("TEST_FAILED: lex-lsp binary not found at " .. lex_lsp_path)
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

-- Track if LSP attached
local lsp_attached = false

-- Setup LSP with callback
lspconfig.lex_lsp.setup({
  on_attach = function(client, bufnr)
    lsp_attached = true
    print("LSP attached: client=" .. client.name .. ", bufnr=" .. bufnr)
  end,
})

-- Register .lex filetype
vim.filetype.add({
  extension = {
    lex = "lex",
  },
})

-- Use verified LSP fixture from specs
local test_file = project_root .. "/specs/v1/benchmark/050-lsp-fixture.lex"

if vim.fn.filereadable(test_file) ~= 1 then
  print("TEST_FAILED: LSP fixture not found at " .. test_file)
  vim.cmd("cquit 1")
end

-- Open the file
vim.cmd("edit " .. test_file)

-- Wait for LSP to attach (with timeout)
local max_wait = 5000 -- 5 seconds
local waited = 0
local wait_step = 100

while not lsp_attached and waited < max_wait do
  vim.wait(wait_step)
  waited = waited + wait_step
end

if not lsp_attached then
  print("TEST_FAILED: LSP did not attach within timeout")
  vim.fn.delete(test_file)
  vim.cmd("cquit 1")
end

-- Wait a bit more for LSP to be fully ready
vim.wait(500)

-- Request document symbols
local params = { textDocument = vim.lsp.util.make_text_document_params() }
local result = vim.lsp.buf_request_sync(0, 'textDocument/documentSymbol', params, 2000)

if not result or vim.tbl_isempty(result) then
  print("TEST_FAILED: No document symbols result returned from LSP")
  vim.fn.delete(test_file)
  vim.cmd("cquit 1")
end

-- Check if we got document symbols
local got_symbols = false
local symbol_count = 0
for client_id, response in pairs(result) do
  if response.result and type(response.result) == "table" then
    got_symbols = true
    symbol_count = #response.result
    print("Document symbols received:")
    print("  Symbol count: " .. symbol_count)
    if symbol_count > 0 then
      print("  Sample symbols:")
      for i = 1, math.min(3, symbol_count) do
        local symbol = response.result[i]
        print("    " .. (symbol.name or "(unnamed)") .. " [" .. (symbol.kind or "?") .. "]")
        if symbol.children and #symbol.children > 0 then
          print("      Children: " .. #symbol.children)
        end
      end
    end
  end
end

if got_symbols and symbol_count > 0 then
  print("TEST_PASSED: LSP document symbols functionality working")
  vim.cmd("qall!")
else
  print("TEST_FAILED: Did not receive valid document symbols")
  vim.cmd("cquit 1")
end
