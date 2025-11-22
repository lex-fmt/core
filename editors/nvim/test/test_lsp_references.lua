-- Test: LSP references functionality

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
lspconfig.lex_lsp.setup({
  on_attach = function()
    lsp_attached = true
  end,
})

vim.filetype.add({ extension = { lex = "lex" } })

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

vim.fn.search("\\[Cache\\]", "w")
local params = vim.lsp.util.make_position_params()
local request = {
  textDocument = params.textDocument,
  position = params.position,
  context = { includeDeclaration = true },
}
local result = vim.lsp.buf_request_sync(0, 'textDocument/references', request, 2000)

if not result or vim.tbl_isempty(result) then
  print("TEST_FAILED: No references response")
  vim.cmd("cquit 1")
end

local references
for _, response in pairs(result) do
  if response.result and vim.tbl_islist(response.result) then
    references = response.result
    break
  end
end

if not references or #references < 2 then
  print("TEST_FAILED: Expected at least 2 references, got " .. tostring(references and #references or 0))
  vim.cmd("cquit 1")
end

print("TEST_PASSED: LSP references working")
vim.cmd("qall!")
