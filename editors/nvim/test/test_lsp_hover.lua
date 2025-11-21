-- Test: LSP hover functionality
-- This test verifies that the LSP server can provide hover information
-- Run with: nvim --headless -u test/minimal_init.lua -l test/test_lsp_hover.lua

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
local hover_result = nil

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

-- Create a test .lex file with content that triggers hover
local test_file = vim.fn.tempname() .. ".lex"
local test_content = {
  "# Test document",
  "section: introduction",
  "",
  "  :: callout ::",
  "    This is an annotation.",
  "",
  "  Some text with a [^ref] footnote.",
  "",
  ":: ref ::",
  "  Footnote content here.",
}
vim.fn.writefile(test_content, test_file)

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

-- Move cursor to line 4 (annotation line: "  :: callout ::")
vim.api.nvim_win_set_cursor(0, {4, 5})

-- Request hover information
local params = vim.lsp.util.make_position_params()
local result = vim.lsp.buf_request_sync(0, 'textDocument/hover', params, 2000)

if not result or vim.tbl_isempty(result) then
  print("TEST_FAILED: No hover result returned from LSP")
  vim.fn.delete(test_file)
  vim.cmd("cquit 1")
end

-- Check if we got a hover response
local got_hover = false
for client_id, response in pairs(result) do
  if response.result and response.result.contents then
    got_hover = true
    local contents = response.result.contents
    print("Hover result received:")
    if type(contents) == "table" then
      if contents.value then
        print("  " .. contents.value)
      elseif contents.kind then
        print("  (kind: " .. contents.kind .. ")")
      end
    else
      print("  " .. vim.inspect(contents))
    end
  end
end

-- Clean up
vim.fn.delete(test_file)

if got_hover then
  print("TEST_PASSED: LSP hover functionality working")
  vim.cmd("qall!")
else
  print("TEST_FAILED: Did not receive hover information")
  vim.cmd("cquit 1")
end
