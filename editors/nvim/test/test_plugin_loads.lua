-- Test: Plugin loads successfully
-- This test verifies that the Lex plugin can be loaded

-- Add plugin directory to runtime path
local script_path = debug.getinfo(1).source:sub(2)
local plugin_dir = vim.fn.fnamemodify(script_path, ":p:h:h")
vim.opt.rtp:prepend(plugin_dir)

-- Attempt to load the plugin
local ok, lex = pcall(require, "lex")

if not ok then
  print("TEST_FAILED: Could not load lex plugin")
  print("Error: " .. tostring(lex))
  vim.cmd("cquit 1")
end

-- Check that the plugin has expected fields
if type(lex) ~= "table" then
  print("TEST_FAILED: Plugin did not return a table")
  vim.cmd("cquit 1")
end

if type(lex.setup) ~= "function" then
  print("TEST_FAILED: Plugin does not have a setup function")
  vim.cmd("cquit 1")
end

if type(lex.version) ~= "string" then
  print("TEST_FAILED: Plugin does not have a version string")
  vim.cmd("cquit 1")
end

print("TEST_PASSED: Plugin loaded successfully")
print("Version: " .. lex.version)
vim.cmd("qall!")
