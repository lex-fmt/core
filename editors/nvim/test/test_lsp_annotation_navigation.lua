-- Test: LSP annotation navigation (next/previous)

local script_path = debug.getinfo(1).source:sub(2)
local test_dir = vim.fn.fnamemodify(script_path, ":p:h")
local plugin_dir = vim.fn.fnamemodify(test_dir, ":h")

local lspconfig_ok, lspconfig = pcall(require, "lspconfig")
if not lspconfig_ok then
  print("TEST_FAILED: lspconfig not available")
  vim.cmd("cquit 1")
end

local configs = require("lspconfig.configs")
local project_root = vim.fn.fnamemodify(plugin_dir, ":h:h")
local lex_lsp_path = project_root .. "/target/debug/lex-lsp"

if vim.fn.filereadable(lex_lsp_path) ~= 1 then
  print("TEST_FAILED: lex-lsp binary not found at " .. lex_lsp_path)
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
      settings = {},
    },
  }
end

local lsp_attached = false
local attached_client = nil
lspconfig.lex_lsp.setup({
  on_attach = function(client)
    lsp_attached = true
    attached_client = client
  end,
})

vim.filetype.add({ extension = { lex = "lex" } })

-- Use fixture with multiple annotations
local fixture = project_root .. "/specs/v1/benchmark/050-lsp-fixture.lex"
if vim.fn.filereadable(fixture) ~= 1 then
  print("TEST_FAILED: fixture not found at " .. fixture)
  vim.cmd("cquit 1")
end

vim.cmd("edit " .. fixture)

local waited = 0
while not lsp_attached and waited < 5000 do
  vim.wait(100)
  waited = waited + 100
end

if not lsp_attached then
  print("TEST_FAILED: LSP did not attach")
  vim.cmd("cquit 1")
end

vim.wait(300)

-- Test 1: Navigate from line 1 to next annotation (should go to callout on line 10)
-- The fixture has annotations at:
-- Line 1: :: doc.note severity=info :: Document preface.
-- Line 10-12: :: callout :: ... ::
-- Line 22: :: shell language=bash

vim.api.nvim_win_set_cursor(0, { 2, 0 }) -- Start on line 2 (after first annotation)

local uri = vim.uri_from_bufnr(0)
local position = { line = 1, character = 0 } -- 0-indexed line 1

local result = attached_client.request_sync("workspace/executeCommand", {
  command = "lex.next_annotation",
  arguments = { uri, position },
}, 5000, 0)

if not result or not result.result then
  print("TEST_FAILED: No next_annotation response")
  vim.cmd("cquit 1")
end

local location = result.result
if not location or not location.range then
  print("TEST_FAILED: Invalid location response format")
  vim.cmd("cquit 1")
end

-- The next annotation after line 1 should be at line 10 (0-indexed: 9)
local expected_line = 9
if location.range.start.line ~= expected_line then
  print(string.format(
    "TEST_FAILED: Expected next annotation on line %d but got %d",
    expected_line,
    location.range.start.line
  ))
  vim.cmd("cquit 1")
end

print("TEST_PASSED: next_annotation working")

-- Test 2: Navigate backwards from line 22 to previous annotation (should go to callout)
position = { line = 21, character = 0 } -- 0-indexed line 21 (before the shell annotation line)

result = attached_client.request_sync("workspace/executeCommand", {
  command = "lex.previous_annotation",
  arguments = { uri, position },
}, 5000, 0)

if not result or not result.result then
  print("TEST_FAILED: No previous_annotation response")
  vim.cmd("cquit 1")
end

location = result.result
if not location or not location.range then
  print("TEST_FAILED: Invalid previous_annotation location format")
  vim.cmd("cquit 1")
end

-- The previous annotation before line 21 should be at line 10 (0-indexed: 9)
expected_line = 9
if location.range.start.line ~= expected_line then
  print(string.format(
    "TEST_FAILED: Expected previous annotation on line %d but got %d",
    expected_line,
    location.range.start.line
  ))
  vim.cmd("cquit 1")
end

print("TEST_PASSED: previous_annotation working")

-- Test 3: Test that commands module loads and has the functions
local ok, commands = pcall(require, "lex.commands")
if not ok then
  print("TEST_FAILED: Could not load lex.commands module")
  vim.cmd("cquit 1")
end

if type(commands.next_annotation) ~= "function" then
  print("TEST_FAILED: commands.next_annotation is not a function")
  vim.cmd("cquit 1")
end

if type(commands.previous_annotation) ~= "function" then
  print("TEST_FAILED: commands.previous_annotation is not a function")
  vim.cmd("cquit 1")
end

print("TEST_PASSED: commands module exports annotation navigation functions")
print("TEST_PASSED: All annotation navigation tests passed")
vim.cmd("qall!")
