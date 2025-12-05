-- Test: Import markdown command via LSP

local script_path = debug.getinfo(1).source:sub(2)
local test_dir = vim.fn.fnamemodify(script_path, ":p:h")
local plugin_dir = vim.fn.fnamemodify(test_dir, ":h")
local project_root = vim.fn.fnamemodify(plugin_dir, ":h:h")

-- Test that the commands module loads and has the import function
local ok, commands = pcall(require, "lex.commands")
if not ok then
  print("TEST_FAILED: Could not load lex.commands module: " .. tostring(commands))
  vim.cmd("cquit 1")
end

if type(commands.import_markdown) ~= "function" then
  print("TEST_FAILED: commands.import_markdown is not a function")
  vim.cmd("cquit 1")
end

print("TEST_PASSED: commands module exports import_markdown function")

-- Test that user command is registered
commands.setup()

local function command_exists(name)
  local cmds = vim.api.nvim_get_commands({})
  return cmds[name] ~= nil
end

if not command_exists("LexImportMarkdown") then
  print("TEST_FAILED: LexImportMarkdown command not registered")
  vim.cmd("cquit 1")
end

print("TEST_PASSED: LexImportMarkdown user command is registered")

-- Set up LSP for actual import test
vim.filetype.add({ extension = { md = "markdown", lex = "lex" } })

-- Create a temp .lex file to get LSP attached (LSP attaches to .lex files)
local temp_lex = vim.fn.tempname() .. ".lex"
local f = io.open(temp_lex, "w")
if not f then
  print("TEST_FAILED: Could not create temp lex file")
  vim.cmd("cquit 1")
end
f:write("Placeholder")
f:close()

vim.cmd("edit " .. temp_lex)

-- Wait for LSP to attach
local lsp_binary = project_root .. "/target/debug/lex-lsp"
if vim.fn.executable(lsp_binary) ~= 1 then
  print("TEST_FAILED: lex-lsp binary not found at " .. lsp_binary)
  vim.cmd("cquit 1")
end

-- Start LSP client
vim.lsp.start({
  name = "lex_lsp",
  cmd = { lsp_binary },
  root_dir = vim.fn.getcwd(),
  filetypes = { "lex" },
})

-- Wait for LSP to be ready
local max_wait = 5000
local waited = 0
while waited < max_wait do
  local clients = vim.lsp.get_clients({ name = "lex_lsp", bufnr = 0 })
  if #clients > 0 then
    break
  end
  vim.wait(100)
  waited = waited + 100
end

local clients = vim.lsp.get_clients({ name = "lex_lsp", bufnr = 0 })
if #clients == 0 then
  print("TEST_FAILED: LSP client did not attach")
  vim.cmd("cquit 1")
end

print("TEST_PASSED: LSP client attached")

-- Test import via LSP
local md_content = [[# Test

This is a test document for import.

- Item 1
- Item 2
]]

local client = clients[1]
local result = client.request_sync("workspace/executeCommand", {
  command = "lex.import",
  arguments = { "markdown", md_content },
}, 5000, 0)

if not result or not result.result then
  print("TEST_FAILED: lex.import markdown LSP command failed")
  vim.cmd("cquit 1")
end

local lex_output = result.result
if type(lex_output) ~= "string" or lex_output == "" then
  print("TEST_FAILED: lex.import markdown returned empty or non-string result")
  vim.cmd("cquit 1")
end

-- Check that the lex output contains expected content
if not lex_output:match("Test") then
  print("TEST_FAILED: lex output doesn't contain expected content")
  vim.cmd("cquit 1")
end

print("TEST_PASSED: lex.import markdown via LSP works")

-- Clean up
vim.fn.delete(temp_lex)

print("TEST_PASSED: All import markdown tests passed")
vim.cmd("qall!")
