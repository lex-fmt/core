-- Test: Verify built-in lex.vim syntax is disabled for .lex files
-- Issue: Neovim has a built-in syntax/lex.vim for Unix lex/flex tool which
--        conflicts with our LSP semantic token highlighting.
--
-- Run: nvim --headless -u test/minimal_init.lua -l test/test_syntax_disabled.lua
--
-- This test checks:
-- 1. vim.bo.syntax is empty (not 'lex')
-- 2. No syntax highlight groups from lex.vim are active at cursor positions

local script_path = debug.getinfo(1).source:sub(2)
local plugin_dir = vim.fn.fnamemodify(script_path, ":p:h:h")
local project_root = vim.fn.fnamemodify(plugin_dir, ":h:h")

local function test_syntax_disabled()
  local errors = {}

  -- Open a lex file
  vim.cmd("edit " .. project_root .. "/specs/v1/benchmark/010-kitchensink.lex")

  -- Wait for filetype detection and autocmds to run
  vim.wait(1000)

  -- Test 1: Check filetype is 'lex'
  if vim.bo.filetype ~= "lex" then
    table.insert(errors, string.format("filetype should be 'lex' but is '%s'", vim.bo.filetype))
  end

  -- Test 2: Check syntax is empty
  if vim.bo.syntax ~= "" then
    table.insert(errors, string.format("syntax should be '' but is '%s'", vim.bo.syntax))
  end

  -- Test 3: Check no syntax highlights at various positions
  -- Move cursor to different positions and check for lex.vim syntax groups
  local test_positions = {
    { 1, 0 },   -- First line
    { 3, 10 },  -- Line 3, col 10
    { 5, 5 },   -- Line 5, col 5
  }

  for _, pos in ipairs(test_positions) do
    local row, col = pos[1] - 1, pos[2]  -- 0-indexed for API
    local line_count = vim.api.nvim_buf_line_count(0)

    if row < line_count then
      local inspect = vim.inspect_pos(0, row, col)

      if inspect.syntax and #inspect.syntax > 0 then
        for _, syn in ipairs(inspect.syntax) do
          -- lex.vim uses groups like lexAbbrvBlock, lexPat, etc.
          if syn.hl_group and syn.hl_group:match("^lex") then
            table.insert(errors, string.format(
              "Found lex.vim syntax group '%s' at L%d:C%d",
              syn.hl_group, row + 1, col + 1
            ))
          end
        end
      end
    end
  end

  return errors
end

-- Run the test
print("=== TEST: Built-in lex.vim syntax disabled ===\n")

local errors = test_syntax_disabled()

print(string.format("filetype: %s", vim.bo.filetype))
print(string.format("syntax: '%s'", vim.bo.syntax))
print("")

if #errors == 0 then
  print("PASSED: Built-in lex.vim syntax is properly disabled")
  vim.cmd("qall!")
else
  print("FAILED:")
  for _, err in ipairs(errors) do
    print("  - " .. err)
  end
  vim.cmd("cquit 1")
end
