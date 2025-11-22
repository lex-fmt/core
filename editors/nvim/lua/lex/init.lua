-- Lex Neovim plugin
-- Main entry point for the Lex language plugin
--
-- This plugin provides LSP integration for Lex documents, including:
-- - Automatic LSP server registration and connection
-- - Semantic token highlighting (parser-driven, not regex-based)
-- - Filetype detection for .lex files
--
-- Usage:
--   require("lex").setup({
--     cmd = {"lex-lsp"},   -- command to start LSP server
--   })
--
-- See docs/dev/nvim-fasttrack.lex for architecture overview
-- See docs/dev/guides/lsp-plugins.lex for detailed design documentation

local M = {}

-- Plugin version
M.version = "0.1.0"

-- Setup function called by lazy.nvim or manual setup
function M.setup(opts)
  opts = opts or {}

  -- Register .lex filetype
  vim.filetype.add({
    extension = {
      lex = "lex",
    },
  })

  -- Setup LSP if lspconfig is available
  local ok, lspconfig = pcall(require, "lspconfig")
  if ok then
    local configs = require("lspconfig.configs")

    -- Register lex-lsp server config if not already registered
    if not configs.lex_lsp then
      configs.lex_lsp = {
        default_config = {
          cmd = opts.cmd or { "lex-lsp" },
          filetypes = { "lex" },
          root_dir = function(fname)
            return lspconfig.util.find_git_ancestor(fname) or vim.fn.getcwd()
          end,
          settings = opts.settings or {},
        },
      }
    end

    -- Auto-start LSP for .lex files with semantic token support
    local lsp_config = opts.lsp_config or {}
    local user_on_attach = lsp_config.on_attach

    -- Ensure cmd is passed to the LSP config if provided
    if opts.cmd and not lsp_config.cmd then
      lsp_config.cmd = opts.cmd
    end

    lsp_config.on_attach = function(client, bufnr)
      -- CRITICAL: Enable semantic token highlighting for .lex files
      -- .lex files don't use traditional Vim syntax files - they rely on LSP semantic tokens
      -- Without this call, .lex files will fall back to generic C syntax (wrong highlighting)
      -- or show no highlighting at all. This must be called in on_attach after LSP connects.
      if client.server_capabilities.semanticTokensProvider then
        vim.lsp.semantic_tokens.start(bufnr, client.id)
      end

      -- Preserve user's on_attach callback if they provided one
      if user_on_attach then
        user_on_attach(client, bufnr)
      end
    end

    lspconfig.lex_lsp.setup(lsp_config)
  end

  -- Setup autocommands for .lex files
  local augroup = vim.api.nvim_create_augroup("LexPlugin", { clear = true })

  vim.api.nvim_create_autocmd("FileType", {
    group = augroup,
    pattern = "lex",
    callback = function()
      -- Comment support - Lex uses annotations for comments
      vim.bo.commentstring = ":: comment :: %s"
      vim.bo.comments = ""

      -- Document editing settings - soft wrap at window width
      vim.wo.wrap = true        -- Soft wrap long lines at window width
      vim.wo.linebreak = true   -- Break at word boundaries, not mid-word
      vim.bo.textwidth = 0      -- No hard wrapping (no auto line breaks)
    end,
  })
end

return M
