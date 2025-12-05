-- Test: LSP go-to-definition functionality

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

-- Create a test .lex file
local test_file = vim.fn.tempname() .. ".lex"
local test_content = {
  "# Test document",
  "",
  "See [MyDef].",
  "",
  "MyDef:",
  "    Definition content.",
}
vim.fn.writefile(test_content, test_file)

vim.cmd("edit " .. test_file)

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

vim.fn.search("MyDef\\]", "w")
local params = vim.lsp.util.make_position_params()
local result = vim.lsp.buf_request_sync(0, 'textDocument/definition', params, 2000)

if not result or vim.tbl_isempty(result) then
  print("TEST_FAILED: No definition response")
  vim.cmd("cquit 1")
end

local location
for _, response in pairs(result) do
  if response.result then
    if vim.tbl_islist(response.result) then
      location = response.result[1]
    elseif response.result.range then
      location = response.result
    end
  end
end

if not location or not location.range then
  print("TEST_FAILED: Invalid definition response format")
  vim.cmd("cquit 1")
end

local definition_line
for idx, line in ipairs(vim.fn.getline(1, vim.fn.line('$'))) do
  if line:match("^%s*MyDef:") then
    definition_line = idx - 1 -- zero-based
    break
  end
end

if not definition_line then
  print("TEST_FAILED: Could not determine definition line")
  vim.cmd("cquit 1")
end

if location.range.start.line ~= definition_line then
  print(string.format(
    "TEST_FAILED: Expected definition on line %d but got %d",
    definition_line,
    location.range.start.line
  ))
  vim.cmd("cquit 1")
end

print("TEST_PASSED: LSP go-to-definition working")
vim.cmd("qall!")
