-- Debug script to show hover output
-- Run with: nvim --headless -u test/minimal_init.lua -l test/debug_hover.lua

local script_path = debug.getinfo(1).source:sub(2)
local plugin_dir = vim.fn.fnamemodify(script_path, ":p:h:h")
local project_root = vim.fn.fnamemodify(plugin_dir, ":h:h")

-- Set up LSP
local lspconfig_ok, lspconfig = pcall(require, "lspconfig")
if not lspconfig_ok then
  print("ERROR: lspconfig not available")
  vim.cmd("cquit 1")
end

local configs = require("lspconfig.configs")
local lex_lsp_path = project_root .. "/target/debug/lex-lsp"

if vim.fn.filereadable(lex_lsp_path) ~= 1 then
  print("ERROR: lex-lsp binary not found at " .. lex_lsp_path)
  vim.cmd("cquit 1")
end

-- Register lex LSP config
if not configs.lex_lsp then
  configs.lex_lsp = {
    default_config = {
      cmd = { lex_lsp_path },
      filetypes = { "lex" },
      root_dir = function(fname)
        return lspconfig.util.find_git_ancestor(fname) or vim.fn.getcwd()
      end,
      settings = {},
    },
  }
end

local lsp_attached = false

lspconfig.lex_lsp.setup({
  on_attach = function(client, bufnr)
    lsp_attached = true
    print("✓ LSP attached: client=" .. client.name .. ", bufnr=" .. bufnr)
  end,
})

vim.filetype.add({
  extension = {
    lex = "lex",
  },
})

-- Open the file
local test_file = project_root .. "/specs/v1/benchmark/050-lsp-fixture.lex"
print("Opening: " .. test_file)
vim.cmd("edit " .. test_file)

-- Wait for LSP to attach
local max_wait = 5000
local waited = 0
local wait_step = 100

while not lsp_attached and waited < max_wait do
  vim.wait(wait_step)
  waited = waited + wait_step
end

if not lsp_attached then
  print("ERROR: LSP did not attach")
  vim.cmd("cquit 1")
end

vim.wait(500)

-- Test different positions
local test_positions = {
  { line = 5, col = 48, desc = "Reference [^source]" },
  { line = 5, col = 58, desc = "Citation [@spec2025 p.4]" },
  { line = 5, col = 75, desc = "Reference [Cache]" },
  { line = 10, col = 6, desc = "Annotation :: callout ::" },
}

for _, pos in ipairs(test_positions) do
  print("\n=== Testing hover at line " .. pos.line .. ", col " .. pos.col .. " (" .. pos.desc .. ") ===")
  vim.api.nvim_win_set_cursor(0, {pos.line, pos.col})

  local params = vim.lsp.util.make_position_params()
  local result = vim.lsp.buf_request_sync(0, 'textDocument/hover', params, 2000)

  if result and not vim.tbl_isempty(result) then
    for client_id, response in pairs(result) do
      if response.result and response.result.contents then
        local contents = response.result.contents
        if contents.value then
          print("Hover content:\n" .. contents.value)
        else
          print("Hover: " .. vim.inspect(contents))
        end
      else
        print("No hover content at this position")
      end
    end
  else
    print("No hover result at this position")
  end
end

vim.cmd("qall!")
