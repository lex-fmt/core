-- Test: LSP document and range formatting functionality

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
local lex_cli_path = project_root .. "/target/debug/lex"

local function fail(msg)
  print("TEST_FAILED: " .. msg)
  vim.cmd("cquit 1")
end

if vim.fn.filereadable(lex_lsp_path) ~= 1 then
  fail("lex-lsp binary not found at " .. lex_lsp_path)
end

if vim.fn.filereadable(lex_cli_path) ~= 1 then
  fail("lex CLI binary not found at " .. lex_cli_path)
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
lspconfig.lex_lsp.setup({
  on_attach = function()
    lsp_attached = true
  end,
})

vim.filetype.add({ extension = { lex = "lex" } })

local fixture = plugin_dir .. "/test/fixtures/formatting.lex"
if vim.fn.filereadable(fixture) ~= 1 then
  fail("formatting fixture not found at " .. fixture)
end

vim.cmd("edit " .. fixture)

local waited = 0
while not lsp_attached and waited < 5000 do
  vim.wait(100)
  waited = waited + 100
end

if not lsp_attached then
  fail("LSP did not attach")
end

local function canonical_from_cli(lines)
  local tmp_in = vim.fn.tempname() .. ".lex"
  local tmp_out = vim.fn.tempname() .. ".lex"
  vim.fn.writefile(lines, tmp_in)
  local output = vim.fn.system({ lex_cli_path, tmp_in, "--to", "lex", "--output", tmp_out })
  vim.fn.delete(tmp_in)
  if vim.v.shell_error ~= 0 then
    vim.fn.delete(tmp_out)
    fail("lex CLI failed: " .. output)
  end
  local formatted = vim.fn.readfile(tmp_out)
  vim.fn.delete(tmp_out)
  return formatted
end

local util = vim.lsp.util

local function apply_and_assert(method, params)
  local result = vim.lsp.buf_request_sync(0, method, params, 4000)
  if not result or vim.tbl_isempty(result) then
    fail("no response for " .. method)
  end
  local applied = false
  local bufnr = vim.api.nvim_get_current_buf()
  for _, response in pairs(result) do
    if response.result and not vim.tbl_isempty(response.result) then
      util.apply_text_edits(response.result, bufnr, 'utf-16')
      applied = true
    end
  end
  if not applied then
    fail("server returned no edits for " .. method)
  end
end

local function buffer_text()
  return vim.api.nvim_buf_get_lines(0, 0, -1, false)
end

local messy_full = {
  "Section One:",
  "",
  "    - keep   ",
  "    - align me",
  "",
  "",
  "",
  "",
  "Section Two:",
  "",
  "  - fix me",
  "  - also me",
  "",
  "",
}

vim.api.nvim_buf_set_lines(0, 0, -1, false, messy_full)
local expected_full = canonical_from_cli(messy_full)
apply_and_assert('textDocument/formatting', util.make_formatting_params({}))

local formatted_lines = buffer_text()
if #formatted_lines ~= #expected_full then
  fail(string.format(
    "expected %d lines after formatting, got %d",
    #expected_full,
    #formatted_lines
  ))
end

for idx, line in ipairs(expected_full) do
  if formatted_lines[idx] ~= line then
    fail(string.format(
      "document formatting mismatch on line %d (expected '%s', got '%s')",
      idx,
      line,
      formatted_lines[idx] or "<nil>"
    ))
  end
end

-- Range formatting test: currently range formatting does full document replacement
-- (incremental range formatting can be added once the formatter matures)
local messy_range = {
  "Section One:",
  "",
  "    - keep",
  "    - align me",
  "",
  "",
  "",
  "Section Two:",
  "",
  "",
  "    - fix me   ",
  "  -    also me",
  "",
  "",
}

vim.api.nvim_buf_set_lines(0, 0, -1, false, messy_range)

local range_params = {
  textDocument = util.make_text_document_params(0),
  range = {
    start = { line = 7, character = 0 },
    ["end"] = { line = 13, character = 0 },
  },
  options = {
    tabSize = 4,
    insertSpaces = true,
  },
}

apply_and_assert('textDocument/rangeFormatting', range_params)

-- Range formatting currently does full document replacement, so we expect
-- the entire document to be formatted (same as document formatting)
local after_range = buffer_text()
local expected_range = canonical_from_cli(messy_range)

if #after_range ~= #expected_range then
  fail(string.format(
    "expected %d lines after range formatting, got %d",
    #expected_range,
    #after_range
  ))
end

for idx, line in ipairs(expected_range) do
  if after_range[idx] ~= line then
    fail(string.format(
      "range formatting mismatch on line %d (expected '%s', got '%s')",
      idx,
      line,
      after_range[idx] or "<nil>"
    ))
  end
end

print("TEST_PASSED: LSP formatting working")
vim.cmd("qall!")
