-- Test: Export commands (Markdown, HTML, PDF)

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

-- Test that the CLI binary can be found
local lex_cli_path = project_root .. "/target/debug/lex"
if vim.fn.executable(lex_cli_path) ~= 1 then
  print("TEST_FAILED: lex CLI binary not found at " .. lex_cli_path)
  vim.cmd("cquit 1")
end

print("TEST_PASSED: lex CLI binary found")

-- Test actual export functionality by creating a temp file and exporting it
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
vim.wait(100)

-- Test export to markdown
local temp_md = temp_lex:gsub("%.lex$", ".md")
local md_result = vim.system({ lex_cli_path, "convert", "--to", "markdown", temp_lex }, { text = true }):wait()

if md_result.code ~= 0 then
  print("TEST_FAILED: lex convert --to markdown failed: " .. (md_result.stderr or "unknown error"))
  vim.cmd("cquit 1")
end

if not md_result.stdout or md_result.stdout == "" then
  print("TEST_FAILED: lex convert --to markdown returned empty output")
  vim.cmd("cquit 1")
end

-- Check that the markdown output contains expected content
if not md_result.stdout:match("Test") then
  print("TEST_FAILED: markdown output doesn't contain expected content")
  vim.cmd("cquit 1")
end

print("TEST_PASSED: lex convert --to markdown works")

-- Test export to HTML
local html_result = vim.system({ lex_cli_path, "convert", "--to", "html", temp_lex }, { text = true }):wait()

if html_result.code ~= 0 then
  print("TEST_FAILED: lex convert --to html failed: " .. (html_result.stderr or "unknown error"))
  vim.cmd("cquit 1")
end

if not html_result.stdout or html_result.stdout == "" then
  print("TEST_FAILED: lex convert --to html returned empty output")
  vim.cmd("cquit 1")
end

-- Check that the HTML output contains expected content
if not html_result.stdout:match("<") then
  print("TEST_FAILED: html output doesn't contain HTML tags")
  vim.cmd("cquit 1")
end

print("TEST_PASSED: lex convert --to html works")

-- Test export to PDF
local temp_pdf = temp_lex:gsub("%.lex$", ".pdf")
local pdf_result = vim.system({ lex_cli_path, "convert", "--to", "pdf", "--output", temp_pdf, temp_lex }, { text = true }):wait()

if pdf_result.code ~= 0 then
  print("TEST_FAILED: lex convert --to pdf failed: " .. (pdf_result.stderr or "unknown error"))
  vim.cmd("cquit 1")
end

if vim.fn.filereadable(temp_pdf) ~= 1 then
  print("TEST_FAILED: PDF file not created")
  vim.cmd("cquit 1")
end

print("TEST_PASSED: lex convert --to pdf works")

-- Clean up
vim.fn.delete(temp_lex)
vim.fn.delete(temp_md)
vim.fn.delete(temp_pdf)

print("TEST_PASSED: All export command tests passed")
vim.cmd("qall!")
