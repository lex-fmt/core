-- Test: LSP insert verbatim command

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

-- Use fixture
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

-- Test 1: Insert verbatim command returns valid snippet payload for a text file
local uri = vim.uri_from_bufnr(0)
local position = { line = 5, character = 0 }
local file_path = project_root .. "/AGENTS.md"

local result = attached_client.request_sync("workspace/executeCommand", {
  command = "lex.insert_verbatim",
  arguments = { uri, position, file_path },
}, 5000, 0)

if not result then
  print("TEST_FAILED: No insert_verbatim result at all")
  vim.cmd("cquit 1")
end

if result.err then
  print("TEST_FAILED: insert_verbatim error: " .. vim.inspect(result.err))
  vim.cmd("cquit 1")
end

if not result.result then
  print("TEST_FAILED: No insert_verbatim response, result was: " .. vim.inspect(result))
  vim.cmd("cquit 1")
end

local payload = result.result
if type(payload.text) ~= "string" then
  print("TEST_FAILED: insert_verbatim response missing 'text' field")
  vim.cmd("cquit 1")
end

if type(payload.cursorOffset) ~= "number" then
  print("TEST_FAILED: insert_verbatim response missing 'cursorOffset' field")
  vim.cmd("cquit 1")
end

-- The text should contain a verbatim block marker (::)
if not payload.text:match("::") then
  print("TEST_FAILED: insert_verbatim text doesn't contain verbatim marker: " .. payload.text)
  vim.cmd("cquit 1")
end

print("TEST_PASSED: insert_verbatim returns valid snippet payload")

-- Test 2: Verify commands module has the function
local ok, commands = pcall(require, "lex.commands")
if not ok then
  print("TEST_FAILED: Could not load lex.commands module")
  vim.cmd("cquit 1")
end

if type(commands.insert_verbatim) ~= "function" then
  print("TEST_FAILED: commands.insert_verbatim is not a function")
  vim.cmd("cquit 1")
end

print("TEST_PASSED: commands module exports insert_verbatim function")
print("TEST_PASSED: All insert verbatim tests passed")
vim.cmd("qall!")
