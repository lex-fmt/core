-- Lex Neovim Plugin
-- ===================
--
-- Main entry point for the Lex language plugin providing:
-- - LSP integration via nvim-lspconfig
-- - Semantic token highlighting (parser-driven, not regex-based)
-- - Filetype detection for .lex files
--
-- See README.lex for user documentation.

local binary_manager = require("lex.binary")
local theme = require("lex.theme")
local debug = require("lex.debug")
local commands = require("lex.commands")

local M = {}

-- Plugin version + bundled lex-lsp version (used by binary manager).
M.version = "0.1.0"
M.lex_lsp_version = "v0.1.0"

-- Resolve which lex-lsp binary to execute.
local function resolve_lsp_cmd(opts)
  if opts.cmd then
    return opts.cmd
  end

  local desired = opts.lex_lsp_version
  if desired == nil then
    desired = M.lex_lsp_version
  end

  if desired and desired ~= "" then
    local path = binary_manager.ensure_binary(desired)
    if path then
      return { path }
    end
  end

  return { "lex-lsp" }
end

-- Setup function called by lazy.nvim or manual setup
function M.setup(opts)
  opts = opts or {}
  local resolved_cmd = resolve_lsp_cmd(opts)
  local theme_name = opts.theme or "monochrome"

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
          cmd = resolved_cmd,
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

    if not lsp_config.cmd then
      lsp_config.cmd = resolved_cmd
    end

    lsp_config.on_attach = function(client, bufnr)
      -- Enable semantic token highlighting
      if client.server_capabilities.semanticTokensProvider then
        vim.lsp.semantic_tokens.start(bufnr, client.id)
        theme.apply(theme_name)
      end

      -- Setup buffer-local keymaps for commands
      commands.setup_keymaps(bufnr)

      -- Preserve user's on_attach callback
      if user_on_attach then
        user_on_attach(client, bufnr)
      end
    end

    lspconfig.lex_lsp.setup(lsp_config)
  end

  -- Setup debug commands
  debug.setup()

  -- Setup user commands (global, not buffer-specific)
  commands.setup()

  -- Setup autocommands for .lex files
  local augroup = vim.api.nvim_create_augroup("LexPlugin", { clear = true })

  vim.api.nvim_create_autocmd("FileType", {
    group = augroup,
    pattern = "lex",
    callback = function()
      -- Comment support - Lex uses annotations for comments
      vim.bo.commentstring = ":: note :: %s"
      vim.bo.comments = ""

      -- Document editing settings - soft wrap at window width
      vim.wo.wrap = true
      vim.wo.linebreak = true
      vim.bo.textwidth = 0
    end,
  })

  -- Disable built-in lex.vim syntax (conflicts with LSP semantic tokens)
  local function disable_lex_syntax()
    if vim.bo.filetype == "lex" and vim.bo.syntax ~= "" then
      vim.bo.syntax = ""
      vim.cmd("syntax clear")
    end
  end

  vim.api.nvim_create_autocmd({ "FileType" }, {
    group = augroup,
    pattern = "lex",
    callback = function()
      vim.schedule(disable_lex_syntax)
    end,
  })

  vim.api.nvim_create_autocmd({ "BufEnter", "BufWinEnter", "Syntax" }, {
    group = augroup,
    pattern = { "*.lex", "lex" },
    callback = function()
      vim.schedule(disable_lex_syntax)
    end,
  })
end

return M
