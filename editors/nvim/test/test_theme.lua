-- Test: Theme application
-- Ensures that requesting the dark theme assigns highlight groups for semantic tokens

local script_path = debug.getinfo(1).source:sub(2)
local plugin_dir = vim.fn.fnamemodify(script_path, ":p:h:h")
vim.opt.rtp:prepend(plugin_dir)

local lex = require("lex")
lex.setup({ theme = "lex-dark" })

local function has_highlight(group)
  local ok, hl = pcall(vim.api.nvim_get_hl, 0, { name = group, link = true })
  return ok and (hl.link ~= nil or hl.fg ~= nil)
end

-- Test the new standard semantic token types
local targets = {
  "@lsp.type.markup.heading",  -- Session titles
  "@lsp.type.string",          -- Inline code, verbatim blocks
  "@lsp.type.markup.underline",  -- References
}

for _, group in ipairs(targets) do
  if not has_highlight(group) then
    print("TEST_FAILED: missing highlight for " .. group)
    vim.cmd("cquit 1")
  end
end

print("TEST_PASSED: Theme highlights assigned")
vim.cmd("qall!")
