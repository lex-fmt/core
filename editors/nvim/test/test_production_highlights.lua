-- Test: Production mode highlight groups
-- Run: nvim --headless -u test/minimal_init.lua -l test/test_production_highlights.lua

local script_path = debug.getinfo(1).source:sub(2)
local plugin_dir = vim.fn.fnamemodify(script_path, ":p:h:h")
local project_root = vim.fn.fnamemodify(plugin_dir, ":h:h")

-- Open a lex file to trigger LSP and highlights
vim.cmd("edit " .. project_root .. "/specs/v1/benchmark/010-kitchensink.lex")
vim.wait(2000) -- Wait for LSP

print("=== PRODUCTION MODE HIGHLIGHTS ===\n")

-- Check base groups exist
print("-- Base intensity groups --")
local base_groups = { "@lex.muted", "@lex.faint" }
for _, name in ipairs(base_groups) do
  local hl = vim.api.nvim_get_hl(0, { name = name })
  local info = hl.link and ("link=" .. hl.link) or vim.inspect(hl)
  print(string.format("  %s: %s", name, info))
end

-- Check semantic token highlights
print("\n-- NORMAL intensity (typography only, inherits Normal fg) --")
local normal_groups = {
  { "@lsp.type.SessionTitleText", "bold" },
  { "@lsp.type.DefinitionSubject", "italic" },
  { "@lsp.type.InlineStrong", "bold" },
  { "@lsp.type.InlineEmphasis", "italic" },
  { "@lsp.type.InlineMath", "italic" },
}
for _, item in ipairs(normal_groups) do
  local name, expected = item[1], item[2]
  local hl = vim.api.nvim_get_hl(0, { name = name, link = false })
  local has_style = (expected == "bold" and hl.bold) or (expected == "italic" and hl.italic)
  local no_fg = hl.fg == nil
  local status = (has_style and no_fg) and "OK" or "FAIL"
  print(string.format("  %s: %s=%s, fg=%s [%s]", name, expected, tostring(hl[expected] or false), tostring(hl.fg), status))
end

print("\n-- MUTED intensity (color from @lex.muted + typography) --")
local muted_groups = {
  { "@lsp.type.SessionMarker", "italic" },
  { "@lsp.type.ListMarker", "italic" },
  { "@lsp.type.Reference", "underline" },
}
for _, item in ipairs(muted_groups) do
  local name, expected = item[1], item[2]
  local hl = vim.api.nvim_get_hl(0, { name = name, link = false })
  local has_style = (expected == "italic" and hl.italic) or (expected == "underline" and hl.underline)
  local has_fg = hl.fg ~= nil
  local status = (has_style and has_fg) and "OK" or "FAIL"
  print(string.format("  %s: %s=%s, fg=%s [%s]", name, expected, tostring(hl[expected] or false), hl.fg and string.format("#%06x", hl.fg) or "nil", status))
end

print("\n-- FAINT intensity (links to @lex.faint) --")
local faint_groups = {
  "@lsp.type.AnnotationLabel",
  "@lsp.type.AnnotationParameter",
  "@lsp.type.VerbatimLanguage",
}
for _, name in ipairs(faint_groups) do
  local hl = vim.api.nvim_get_hl(0, { name = name })
  local is_linked = hl.link == "@lex.faint"
  local status = is_linked and "OK" or "FAIL"
  print(string.format("  %s: link=%s [%s]", name, tostring(hl.link), status))
end

print("\n-- Inline markers (faint + italic) --")
local hl = vim.api.nvim_get_hl(0, { name = "@lsp.type.InlineMarker_strong_start", link = false })
local has_italic = hl.italic == true
local has_fg = hl.fg ~= nil
local status = (has_italic and has_fg) and "OK" or "FAIL"
print(string.format("  @lsp.type.InlineMarker_strong_start: italic=%s, fg=%s [%s]",
  tostring(hl.italic), hl.fg and string.format("#%06x", hl.fg) or "nil", status))

print("\n=== DONE ===")
vim.cmd("qall!")
