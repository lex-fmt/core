-- Debug script: Inspect semantic tokens and highlight groups
-- Run with: nvim --headless -u test/minimal_init.lua -l test/debug_semantic_tokens.lua

local script_path = debug.getinfo(1).source:sub(2)
local plugin_dir = vim.fn.fnamemodify(script_path, ":p:h:h")
local project_root = vim.fn.fnamemodify(plugin_dir, ":h:h")
local lex_lsp_path = project_root .. "/target/debug/lex-lsp"

-- Use the lex plugin with debug_theme enabled
local lex = require("lex")
local lsp_attached = false
local client_ref = nil

lex.setup({
  cmd = { lex_lsp_path },
  debug_theme = true,
  lsp_config = {
    on_attach = function(client, bufnr)
      lsp_attached = true
      client_ref = client
    end,
  },
})

vim.filetype.add({ extension = { lex = "lex" } })

-- Open test file
local test_file = project_root .. "/specs/v1/benchmark/050-lsp-fixture.lex"
vim.cmd("edit " .. test_file)

-- Wait for LSP
local waited = 0
while not lsp_attached and waited < 5000 do
  vim.wait(100)
  waited = waited + 100
end

if not lsp_attached then
  print("ERROR: LSP did not attach")
  vim.cmd("cquit 1")
end

vim.wait(500)

-- Get semantic tokens with legend
local params = { textDocument = vim.lsp.util.make_text_document_params() }
local result = vim.lsp.buf_request_sync(0, 'textDocument/semanticTokens/full', params, 3000)

-- Get the token legend from server capabilities
local legend = client_ref.server_capabilities.semanticTokensProvider.legend
print("=== SEMANTIC TOKEN LEGEND ===")
print("Token types:")
for i, tt in ipairs(legend.tokenTypes) do
  print(string.format("  %d: %s", i - 1, tt))
end

print("\n=== SEMANTIC TOKENS (decoded) ===")
for _, response in pairs(result) do
  if response.result and response.result.data then
    local data = response.result.data
    local lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)

    -- Decode delta-encoded tokens
    local line = 0
    local col = 0
    local count = 0
    for i = 1, #data, 5 do
      local delta_line = data[i]
      local delta_col = data[i + 1]
      local length = data[i + 2]
      local token_type_idx = data[i + 3]
      local token_modifiers = data[i + 4]

      if delta_line > 0 then
        line = line + delta_line
        col = delta_col
      else
        col = col + delta_col
      end

      local token_type = legend.tokenTypes[token_type_idx + 1] or "UNKNOWN"
      local text = ""
      if lines[line + 1] then
        text = string.sub(lines[line + 1], col + 1, col + length)
      end

      count = count + 1
      if count <= 30 then
        print(string.format("  L%d:C%d len=%d type=%s text=%q",
          line + 1, col + 1, length, token_type, text))
      end
    end
    print(string.format("\n... Total: %d tokens", count))
  end
end

-- Check highlight groups
print("\n=== HIGHLIGHT GROUP STATUS ===")
local hl_groups = {
  "@lsp.type.DocumentTitle",
  "@lsp.type.SessionMarker",
  "@lsp.type.SessionTitleText",
  "@lsp.type.DefinitionSubject",
  "@lsp.type.ListMarker",
  "@lsp.type.AnnotationLabel",
  "@lsp.type.InlineStrong",
  "@lsp.type.InlineEmphasis",
  "@lsp.type.InlineCode",
  "@lsp.type.VerbatimContent",
  "@lsp.type.InlineMarker_strong_start",
  "@lsp.type.InlineMarker_emphasis_start",
}

for _, hl in ipairs(hl_groups) do
  local exists = vim.fn.hlexists(hl) == 1
  local info = vim.api.nvim_get_hl(0, { name = hl })
  local details = ""
  if info.link then
    details = "-> " .. info.link
  elseif info.fg then
    details = string.format("fg=#%06x", info.fg)
  else
    details = vim.inspect(info)
  end
  print(string.format("  %s: exists=%s %s", hl, tostring(exists), details))
end

print("\nDone.")
vim.cmd("qall!")
