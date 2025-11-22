-- Lex Neovim plugin
-- Main entry point for the Lex language plugin
--
-- This plugin provides LSP integration for Lex documents, including:
-- - Automatic LSP server registration and connection
-- - Semantic token highlighting (parser-driven, not regex-based)
-- - Theme support with Markdown-inspired color schemes
-- - Filetype detection for .lex files
--
-- Usage:
--   require("lex").setup({
--     theme = "lex-dark",  -- or "lex-light", or false to disable
--     cmd = {"lex-lsp"},   -- command to start LSP server
--   })
--
-- See docs/dev/nvim-fasttrack.lex for architecture overview
-- See docs/dev/guides/lsp-plugins.lex for detailed design documentation

local M = {}

local function apply_theme(theme_name)
  if theme_name == false then
    return
  end

  local default_theme = (vim.o.background == "light") and "lex-light" or "lex-dark"
  local selected = theme_name or default_theme
  local ok, theme = pcall(require, "themes." .. selected)
  if ok and type(theme.apply) == "function" then
    theme.apply()
  end
end

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

  apply_theme(opts.theme)

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

      if client.server_capabilities.documentFormattingProvider then
        vim.keymap.set("n", "<leader>lf", function()
          vim.lsp.buf.format({ async = true })
        end, { buffer = bufnr, desc = "Format current Lex document" })
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
      -- Set buffer-local options for .lex files
      vim.bo.commentstring = "# %s"
      vim.bo.comments = ":#"
    end,
  })
end

return M
