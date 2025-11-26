-- Lex Debug Module
-- =================
-- Debug utilities for inspecting Lex highlighting.

local M = {}

-- Inspect semantic token under cursor
-- Shows token type, highlight group, and definition
local function debug_token()
  local bufnr = vim.api.nvim_get_current_buf()
  local cursor = vim.api.nvim_win_get_cursor(0)
  local row, col = cursor[1] - 1, cursor[2] -- 0-indexed

  local lines = {}
  table.insert(lines, "=== LexDebugToken ===")
  table.insert(lines, string.format("Cursor: L%d:C%d", row + 1, col + 1))
  table.insert(lines, string.format("Filetype: %s | Syntax: '%s'", vim.bo.filetype, vim.bo.syntax))

  -- LSP client info
  local clients = vim.lsp.get_clients({ bufnr = bufnr })
  table.insert(lines, "")
  table.insert(lines, "-- LSP Clients --")
  if #clients == 0 then
    table.insert(lines, "  (none attached)")
  else
    for _, client in ipairs(clients) do
      local has_st = client.server_capabilities.semanticTokensProvider and "yes" or "no"
      table.insert(lines, string.format("  %s (id=%d) semantic_tokens=%s", client.name, client.id, has_st))
    end
  end

  -- Get semantic tokens at cursor using Neovim's API
  table.insert(lines, "")
  table.insert(lines, "-- Semantic Tokens at Cursor --")
  local found_token = false

  -- Use vim.inspect_pos for comprehensive highlight info
  local inspect = vim.inspect_pos(bufnr, row, col)

  if inspect.semantic_tokens and #inspect.semantic_tokens > 0 then
    for _, token in ipairs(inspect.semantic_tokens) do
      found_token = true
      local token_type = token.type or token.opts and token.opts.hl_group or "unknown"
      local hl_group = "@lsp.type." .. token_type
      if token.modifiers and #token.modifiers > 0 then
        hl_group = hl_group .. " (modifiers: " .. table.concat(token.modifiers, ", ") .. ")"
      end
      table.insert(lines, string.format("  Type: %s", token_type))
      table.insert(lines, string.format("  HL Group: @lsp.type.%s", token_type))

      -- Get the highlight definition
      local hl_info = vim.api.nvim_get_hl(0, { name = "@lsp.type." .. token_type })
      if hl_info and next(hl_info) then
        local def_parts = {}
        if hl_info.link then table.insert(def_parts, "link=" .. hl_info.link) end
        if hl_info.fg then table.insert(def_parts, string.format("fg=#%06x", hl_info.fg)) end
        if hl_info.bg then table.insert(def_parts, string.format("bg=#%06x", hl_info.bg)) end
        if hl_info.bold then table.insert(def_parts, "bold") end
        if hl_info.italic then table.insert(def_parts, "italic") end
        if hl_info.underline then table.insert(def_parts, "underline") end
        table.insert(lines, string.format("  HL Def: %s", #def_parts > 0 and table.concat(def_parts, " ") or "(empty!)"))
      else
        table.insert(lines, "  HL Def: (NOT DEFINED)")
      end
    end
  end

  if not found_token then
    table.insert(lines, "  (no semantic token at cursor)")
  end

  -- All highlights at position (treesitter, syntax, etc.)
  table.insert(lines, "")
  table.insert(lines, "-- All Highlights at Cursor --")
  if inspect.treesitter and #inspect.treesitter > 0 then
    for _, ts in ipairs(inspect.treesitter) do
      table.insert(lines, string.format("  treesitter: %s", ts.hl_group))
    end
  end
  if inspect.syntax and #inspect.syntax > 0 then
    for _, syn in ipairs(inspect.syntax) do
      table.insert(lines, string.format("  syntax: %s", syn.hl_group))
    end
  end
  if inspect.extmarks and #inspect.extmarks > 0 then
    for _, ext in ipairs(inspect.extmarks) do
      if ext.opts and ext.opts.hl_group then
        table.insert(lines, string.format("  extmark: %s", ext.opts.hl_group))
      end
    end
  end

  -- Output
  local output = table.concat(lines, "\n")
  print(output)

  -- Also copy to clipboard
  vim.fn.setreg("+", output)
  vim.notify("Debug info copied to clipboard", vim.log.levels.INFO)
end

-- Setup debug commands
function M.setup()
  vim.api.nvim_create_user_command("LexDebugToken", debug_token, {
    desc = "Debug semantic token under cursor"
  })
end

return M
