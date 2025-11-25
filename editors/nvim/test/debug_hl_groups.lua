-- Check what's happening with highlight groups
-- Run: nvim --headless -u test/minimal_init.lua -l test/debug_hl_groups.lua

local script_path = debug.getinfo(1).source:sub(2)
local plugin_dir = vim.fn.fnamemodify(script_path, ":p:h:h")
local project_root = vim.fn.fnamemodify(plugin_dir, ":h:h")

-- Open a lex file to trigger LSP and highlights
vim.cmd("edit " .. project_root .. "/specs/v1/benchmark/010-kitchensink.lex")
vim.wait(2000) -- Wait for LSP

print("=== HIGHLIGHT GROUP CHAIN ===\n")

local groups = {
  "InlineStrong",
  "InlineEmphasis",
  "SessionMarker",
  "SessionTitleText",
  "DefinitionSubject",
}

for _, name in ipairs(groups) do
  print(name .. ":")

  -- Check filetype-specific group
  local lex_group = "@lsp.type." .. name .. ".lex"
  local lex_hl = vim.api.nvim_get_hl(0, { name = lex_group })
  print(string.format("  %s: %s", lex_group, vim.inspect(lex_hl)))

  -- Check base group
  local base_group = "@lsp.type." .. name
  local base_hl = vim.api.nvim_get_hl(0, { name = base_group })
  print(string.format("  %s: %s", base_group, vim.inspect(base_hl)))

  -- Check if hlexists
  print(string.format("  hlexists(%s): %d", lex_group, vim.fn.hlexists(lex_group)))
  print(string.format("  hlexists(%s): %d", base_group, vim.fn.hlexists(base_group)))
  print("")
end

-- Check what the default colorscheme defines
print("=== COLORSCHEME INFO ===")
print("Current colorscheme: " .. vim.g.colors_name or "(none)")
print("termguicolors: " .. tostring(vim.opt.termguicolors:get()))

vim.cmd("qall!")
