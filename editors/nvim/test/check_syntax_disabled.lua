-- Quick check that syntax is disabled for lex files
-- Run: nvim --headless -u test/minimal_init.lua -l test/check_syntax_disabled.lua

local script_path = debug.getinfo(1).source:sub(2)
local plugin_dir = vim.fn.fnamemodify(script_path, ":p:h:h")
local project_root = vim.fn.fnamemodify(plugin_dir, ":h:h")

-- Setup lex plugin
local lex = require("lex")
lex.setup({
  cmd = { project_root .. "/target/debug/lex-lsp" },
  debug_theme = true,
})

-- Open a lex file
vim.cmd("edit " .. project_root .. "/specs/v1/benchmark/010-kitchensink.lex")

-- Wait for filetype detection
vim.wait(500)

print("filetype: " .. vim.bo.filetype)
print("syntax: '" .. vim.bo.syntax .. "'")

if vim.bo.syntax == "" then
  print("TEST_PASSED: Built-in syntax disabled")
else
  print("TEST_FAILED: syntax should be empty but is '" .. vim.bo.syntax .. "'")
  vim.cmd("cquit 1")
end

vim.cmd("qall!")
