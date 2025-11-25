-- Full debug: check LSP attachment, semantic tokens, and highlights
-- Run: nvim --headless -u test/minimal_init.lua -l test/debug_full.lua

local script_path = debug.getinfo(1).source:sub(2)
local plugin_dir = vim.fn.fnamemodify(script_path, ":p:h:h")
local project_root = vim.fn.fnamemodify(plugin_dir, ":h:h")

print("=== LEX DEBUG ===")
print("Plugin dir: " .. plugin_dir)
print("Project root: " .. project_root)

-- Open test file
local test_file = project_root .. "/specs/v1/benchmark/010-kitchensink.lex"
print("\nOpening: " .. test_file)
vim.cmd("edit " .. test_file)

print("Filetype: " .. vim.bo.filetype)
print("Syntax: '" .. vim.bo.syntax .. "'")

-- Wait for LSP to attach
print("\nWaiting for LSP...")
local max_wait = 5000
local waited = 0
local clients = {}

while waited < max_wait do
  vim.wait(100)
  waited = waited + 100
  clients = vim.lsp.get_clients({ bufnr = 0 })
  if #clients > 0 then
    break
  end
end

print("\n=== LSP CLIENTS ===")
if #clients == 0 then
  print("NO LSP CLIENTS ATTACHED!")
  print("This means semantic tokens won't work.")
  vim.cmd("cquit 1")
end

for _, client in ipairs(clients) do
  print(string.format("Client: %s (id=%d)", client.name, client.id))
  print("  Semantic tokens: " .. tostring(client.server_capabilities.semanticTokensProvider ~= nil))

  -- Check if semantic tokens are active for this buffer
  local bufnr = vim.api.nvim_get_current_buf()

  -- Force start semantic tokens
  if client.server_capabilities.semanticTokensProvider then
    print("  Starting semantic tokens...")
    vim.lsp.semantic_tokens.start(bufnr, client.id)
  end
end

-- Wait for semantic tokens to be processed
print("\nWaiting for semantic tokens to be applied...")
vim.wait(1000)

-- Check highlights at various positions
print("\n=== HIGHLIGHT CHECK ===")
local bufnr = vim.api.nvim_get_current_buf()

local test_positions = {
  {3, 30, "should be InlineStrong (*all major features*)"},
  {6, 18, "should be InlineEmphasis (_definition_)"},
  {8, 0, "should be DefinitionSubject (Root Definition:)"},
  {17, 0, "should be SessionMarker (1.)"},
  {17, 3, "should be SessionTitleText (Primary Session)"},
}

for _, pos in ipairs(test_positions) do
  local row, col, desc = pos[1] - 1, pos[2], pos[3]
  local inspect = vim.inspect_pos(bufnr, row, col)

  print(string.format("\nL%d:C%d - %s", row + 1, col + 1, desc))

  if inspect.semantic_tokens and #inspect.semantic_tokens > 0 then
    for _, token in ipairs(inspect.semantic_tokens) do
      print("  RAW TOKEN: " .. vim.inspect(token))
      local token_type = token.type or token.opts and token.opts.hl_group or "unknown"
      print(string.format("  FOUND: %s", tostring(token_type)))
    end
  else
    print("  NO SEMANTIC TOKEN")
    -- Check what extmarks exist at this position
    local extmarks = vim.api.nvim_buf_get_extmarks(bufnr, -1, {row, col}, {row, col + 1}, {details = true})
    if #extmarks > 0 then
      print("  EXTMARKS at position:")
      for _, em in ipairs(extmarks) do
        print("    " .. vim.inspect(em))
      end
    end
  end
end

-- Check if debug_theme highlights are set
print("\n=== DEBUG THEME HIGHLIGHTS ===")
local hl_checks = {
  "@lsp.type.SessionTitle",
  "@lsp.type.InlineStrong",
  "@lsp.type.InlineEmphasis",
}

for _, name in ipairs(hl_checks) do
  local hl = vim.api.nvim_get_hl(0, { name = name })
  local info = ""
  if hl.fg then info = string.format("fg=#%06x", hl.fg) end
  if hl.bold then info = info .. " bold" end
  if hl.italic then info = info .. " italic" end
  if hl.link then info = "link=" .. hl.link end
  if info == "" then info = "(empty)" end
  print(string.format("  %s: %s", name, info))
end

print("\n=== DONE ===")
vim.cmd("qall!")
