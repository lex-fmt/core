-- Quick syntax highlighting verification script
-- Usage: nvim -u test/minimal_init.lua --headless -l test/check_syntax.lua <file>

local file = arg[1] or "AGENTS.md"

-- Open the file
vim.cmd("edit " .. file)

-- Wait for syntax to load
vim.wait(200)

-- Get info
local filetype = vim.bo.filetype
local syntax = vim.bo.syntax
local syntax_output = vim.fn.execute("syntax list")
local lines = vim.split(syntax_output, "\n")

-- Print results
print("=== Syntax Highlighting Check ===")
print("File: " .. vim.fn.expand("%:p"))
print("Filetype: " .. filetype)
print("Syntax: " .. syntax)
print("Syntax on: " .. (vim.g.syntax_on and "yes" or "no"))
print("Termguicolors: " .. tostring(vim.o.termguicolors))
print("")
print("Syntax definitions: " .. #lines .. " lines")

if #lines > 5 then
  print("")
  print("Sample (first 5 items):")
  for i = 1, 5 do
    print("  " .. lines[i])
  end
end

-- Determine success
local success = filetype ~= "" and syntax ~= "" and #lines > 10
print("")
if success then
  print("✓ SUCCESS: Syntax highlighting is working")
  vim.cmd("qall!")
else
  print("✗ FAILED: Syntax highlighting not working properly")
  vim.cmd("cquit 1")
end
