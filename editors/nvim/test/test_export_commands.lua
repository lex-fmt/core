-- Test: Export commands via LSP (Markdown, HTML, PDF)

local script_path = debug.getinfo(1).source:sub(2)
local test_dir = vim.fn.fnamemodify(script_path, ":p:h")
local plugin_dir = vim.fn.fnamemodify(test_dir, ":h")
local project_root = vim.fn.fnamemodify(plugin_dir, ":h:h")

-- Test that the commands module loads and has the export functions
local ok, commands = pcall(require, "lex.commands")
if not ok then
  print("TEST_FAILED: Could not load lex.commands module: " .. tostring(commands))
  vim.cmd("cquit 1")
end

if type(commands.export_markdown) ~= "function" then
  print("TEST_FAILED: commands.export_markdown is not a function")
  vim.cmd("cquit 1")
end

if type(commands.export_html) ~= "function" then
  print("TEST_FAILED: commands.export_html is not a function")
  vim.cmd("cquit 1")
end

if type(commands.export_pdf) ~= "function" then
  print("TEST_FAILED: commands.export_pdf is not a function")
  vim.cmd("cquit 1")
end

print("TEST_PASSED: commands module exports all export functions")

-- Test that user commands are registered
commands.setup()

local function command_exists(name)
  local cmds = vim.api.nvim_get_commands({})
  return cmds[name] ~= nil
end

if not command_exists("LexExportMarkdown") then
  print("TEST_FAILED: LexExportMarkdown command not registered")
  vim.cmd("cquit 1")
end

if not command_exists("LexExportHtml") then
  print("TEST_FAILED: LexExportHtml command not registered")
  vim.cmd("cquit 1")
end

if not command_exists("LexExportPdf") then
  print("TEST_FAILED: LexExportPdf command not registered")
  vim.cmd("cquit 1")
end

print("TEST_PASSED: All export user commands are registered")

-- Set up LSP for actual export tests
vim.filetype.add({ extension = { lex = "lex" } })

-- Create a temp .lex file with some content
local temp_lex = vim.fn.tempname() .. ".lex"
local lex_content = [[1. Test

    This is a test document for export.

    - Item 1
    - Item 2
]]

local f = io.open(temp_lex, "w")
if not f then
  print("TEST_FAILED: Could not create temp file")
  vim.cmd("cquit 1")
end
f:write(lex_content)
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

-- Test export to markdown via LSP
local client = clients[1]
local result = client.request_sync("workspace/executeCommand", {
  command = "lex.export",
  arguments = { "markdown", lex_content },
}, 5000, 0)

if not result or not result.result then
  print("TEST_FAILED: lex.export markdown LSP command failed")
  vim.cmd("cquit 1")
end

local md_output = result.result
if type(md_output) ~= "string" or md_output == "" then
  print("TEST_FAILED: lex.export markdown returned empty or non-string result")
  vim.cmd("cquit 1")
end

if not md_output:match("Test") then
  print("TEST_FAILED: markdown output doesn't contain expected content")
  vim.cmd("cquit 1")
end

print("TEST_PASSED: lex.export markdown via LSP works")

-- Test export to HTML via LSP
result = client.request_sync("workspace/executeCommand", {
  command = "lex.export",
  arguments = { "html", lex_content },
}, 5000, 0)

if not result or not result.result then
  print("TEST_FAILED: lex.export html LSP command failed")
  vim.cmd("cquit 1")
end

local html_output = result.result
if type(html_output) ~= "string" or html_output == "" then
  print("TEST_FAILED: lex.export html returned empty or non-string result")
  vim.cmd("cquit 1")
end

if not html_output:match("<") then
  print("TEST_FAILED: html output doesn't contain HTML tags")
  vim.cmd("cquit 1")
end

print("TEST_PASSED: lex.export html via LSP works")

-- Test export to PDF via LSP (binary format)
local temp_pdf = temp_lex:gsub("%.lex$", ".pdf")
local uri = vim.uri_from_fname(temp_lex)
result = client.request_sync("workspace/executeCommand", {
  command = "lex.export",
  arguments = { "pdf", lex_content, uri, temp_pdf },
}, 10000, 0)

if not result or not result.result then
  print("TEST_FAILED: lex.export pdf LSP command failed")
  vim.cmd("cquit 1")
end

if vim.fn.filereadable(temp_pdf) ~= 1 then
  print("TEST_FAILED: PDF file not created at " .. temp_pdf)
  vim.cmd("cquit 1")
end

print("TEST_PASSED: lex.export pdf via LSP works")

-- Clean up
vim.fn.delete(temp_lex)
vim.fn.delete(temp_pdf)

print("TEST_PASSED: All export command tests passed")
vim.cmd("qall!")
