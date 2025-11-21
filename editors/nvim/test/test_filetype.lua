-- Test: Filetype detection for .lex files
-- This test verifies that .lex files are correctly identified

-- Add plugin directory to runtime path
local script_path = debug.getinfo(1).source:sub(2)
local plugin_dir = vim.fn.fnamemodify(script_path, ":p:h:h")
vim.opt.rtp:prepend(plugin_dir)

-- Load the plugin
local lex = require("lex")
lex.setup()

-- Create a test .lex file
local test_file = vim.fn.tempname() .. ".lex"
vim.fn.writefile({ "# Test .lex file", "section: test" }, test_file)

-- Open the file
vim.cmd("edit " .. test_file)

-- Wait a moment for filetype detection
vim.wait(100)

-- Check the filetype
local filetype = vim.bo.filetype

if filetype ~= "lex" then
  print("TEST_FAILED: Expected filetype 'lex', got '" .. tostring(filetype) .. "'")
  vim.fn.delete(test_file)
  vim.cmd("cquit 1")
end

-- Clean up
vim.fn.delete(test_file)

print("TEST_PASSED: Filetype detection works correctly")
vim.cmd("qall!")
