-- Test: LSP path completion (@ trigger)

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

-- Test 1: Check that @ is a registered trigger character
local capabilities = attached_client.server_capabilities
if not capabilities.completionProvider then
  print("TEST_FAILED: Server does not support completion")
  vim.cmd("cquit 1")
end

local trigger_chars = capabilities.completionProvider.triggerCharacters or {}
local has_at_trigger = false
for _, char in ipairs(trigger_chars) do
  if char == "@" then
    has_at_trigger = true
    break
  end
end

if not has_at_trigger then
  print("TEST_FAILED: @ is not a registered trigger character. Got: " .. vim.inspect(trigger_chars))
  vim.cmd("cquit 1")
end

print("TEST_PASSED: @ is registered as a trigger character")

-- Test 2: Request completion with @ trigger context
local uri = vim.uri_from_bufnr(0)
local position = { line = 5, character = 0 }

-- Build a completion request with @ trigger context
local params = {
  textDocument = { uri = uri },
  position = position,
  context = {
    triggerKind = 2, -- TriggerCharacter
    triggerCharacter = "@",
  },
}

local result = attached_client.request_sync("textDocument/completion", params, 5000, 0)

if not result then
  print("TEST_FAILED: No completion result")
  vim.cmd("cquit 1")
end

if result.err then
  print("TEST_FAILED: Completion error: " .. vim.inspect(result.err))
  vim.cmd("cquit 1")
end

local completions = result.result
if not completions then
  print("TEST_FAILED: No completion items")
  vim.cmd("cquit 1")
end

-- Handle both array and items formats
local items = completions
if completions.items then
  items = completions.items
end

if #items == 0 then
  print("TEST_FAILED: No completion items returned for @ trigger")
  vim.cmd("cquit 1")
end

-- With @ trigger, should only get file completions (not annotation labels, definitions, etc.)
local has_file_completion = false
local has_non_file_completion = false

for _, item in ipairs(items) do
  -- CompletionItemKind.File = 17
  if item.kind == 17 then
    has_file_completion = true
  else
    -- Check if it's an annotation label or definition (which shouldn't be in @ results)
    if item.detail and (item.detail:match("annotation") or item.detail:match("definition")) then
      has_non_file_completion = true
    end
  end
end

if not has_file_completion then
  print("TEST_FAILED: @ trigger did not return file completions")
  vim.cmd("cquit 1")
end

if has_non_file_completion then
  print("TEST_FAILED: @ trigger returned non-file completions (annotation/definition)")
  vim.cmd("cquit 1")
end

print("TEST_PASSED: @ trigger returns only file completions")
print("TEST_PASSED: All path completion tests passed")
vim.cmd("qall!")
