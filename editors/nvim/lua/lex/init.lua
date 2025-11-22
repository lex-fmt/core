-- Lex Neovim plugin
-- Main entry point for the Lex language plugin

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

    -- Auto-start LSP for .lex files
    lspconfig.lex_lsp.setup(opts.lsp_config or {})
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
