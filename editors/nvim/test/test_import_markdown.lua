-- Test: Import markdown command

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

-- Test that the CLI binary can be found
local lex_cli_path = project_root .. "/target/debug/lex"
if vim.fn.executable(lex_cli_path) ~= 1 then
  print("TEST_FAILED: lex CLI binary not found at " .. lex_cli_path)
  vim.cmd("cquit 1")
end

print("TEST_PASSED: lex CLI binary found")

-- Test actual import functionality by creating a temp markdown file
vim.filetype.add({ extension = { md = "markdown" } })

-- Create a temp .md file with some content
local temp_md = vim.fn.tempname() .. ".md"
local md_content = [[# Test

This is a test document for import.

- Item 1
- Item 2
]]

local f = io.open(temp_md, "w")
if not f then
  print("TEST_FAILED: Could not create temp file")
  vim.cmd("cquit 1")
end
f:write(md_content)
f:close()

-- Test that lex convert --to lex works
local lex_result = vim.system({ lex_cli_path, "convert", "--to", "lex", temp_md }, { text = true }):wait()

if lex_result.code ~= 0 then
  print("TEST_FAILED: lex convert --to lex failed: " .. (lex_result.stderr or "unknown error"))
  vim.cmd("cquit 1")
end

if not lex_result.stdout or lex_result.stdout == "" then
  print("TEST_FAILED: lex convert --to lex returned empty output")
  vim.cmd("cquit 1")
end

-- Check that the lex output contains expected content
if not lex_result.stdout:match("Test") then
  print("TEST_FAILED: lex output doesn't contain expected content")
  vim.cmd("cquit 1")
end

print("TEST_PASSED: lex convert --to lex works")

-- Clean up
vim.fn.delete(temp_md)

print("TEST_PASSED: All import markdown tests passed")
vim.cmd("qall!")
