-- Test: LSP document links functionality

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

local fixture = plugin_dir .. "/test/fixtures/example.lex"
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

local params = { textDocument = vim.lsp.util.make_text_document_params() }
local result = vim.lsp.buf_request_sync(0, 'textDocument/documentLink', params, 2000)

if not result or vim.tbl_isempty(result) then
  print("TEST_FAILED: No documentLink response")
  vim.cmd("cquit 1")
end

local links
for _, response in pairs(result) do
  if response.result and vim.tbl_islist(response.result) then
    links = response.result
    break
  end
end

if not links or #links < 2 then
  print("TEST_FAILED: Expected at least 2 document links")
  vim.cmd("cquit 1")
end

local has_external = false
local has_local = false
for _, link in ipairs(links) do
  if link.target then
    local target = link.target
    if target:match("https://example.com") then
      has_external = true
    elseif target:match("guide%.lex") then
      has_local = true
    end
  end
end

if not (has_external and has_local) then
  print("TEST_FAILED: Missing expected link targets")
  vim.cmd("cquit 1")
end

print("TEST_PASSED: LSP document links working")
vim.cmd("qall!")
