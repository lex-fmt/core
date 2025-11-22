-- Minimal Neovim config for testing Lex plugin
-- This config can be used headlessly: nvim -u config/init.lua --headless

-- Add the plugin directory to the runtime path
local plugin_dir = vim.fn.fnamemodify(debug.getinfo(1).source:sub(2), ":p:h:h")
vim.opt.rtp:prepend(plugin_dir)

-- Bootstrap lazy.nvim
local lazypath = vim.fn.stdpath("data") .. "/lazy/lazy.nvim"
if not vim.loop.fs_stat(lazypath) then
  vim.fn.system({
    "git",
    "clone",
    "--filter=blob:none",
    "https://github.com/folke/lazy.nvim.git",
    "--branch=stable",
    lazypath,
  })
end
vim.opt.rtp:prepend(lazypath)

-- Minimal settings
vim.g.mapleader = " "
vim.g.maplocalleader = "\\"

-- Enable colors for visual testing
vim.opt.termguicolors = true
vim.cmd("syntax on")

-- Setup lazy.nvim with minimal plugins for LSP support
require("lazy").setup({
  -- LSP config
  {
    "neovim/nvim-lspconfig",
    dependencies = {
      -- Mason for managing LSP servers (optional, we'll use lex-lsp directly)
      "williamboman/mason.nvim",
      "williamboman/mason-lspconfig.nvim",
    },
    config = function()
      -- Setup Mason (optional)
      require("mason").setup()
      require("mason-lspconfig").setup()

      -- Setup LSP capabilities with semantic tokens support
      local capabilities = vim.lsp.protocol.make_client_capabilities()
      capabilities.textDocument.semanticTokens = {
        dynamicRegistration = false,
        tokenTypes = {
          "lexSessionTitle",
          "lexDefinitionSubject",
          "lexListMarker",
          "lexAnnotationLabel",
          "lexAnnotationParameter",
          "lexInlineStrong",
          "lexInlineEmphasis",
          "lexInlineCode",
          "lexInlineMath",
          "lexReference",
          "lexReferenceCitation",
          "lexReferenceFootnote",
          "lexVerbatimSubject",
          "lexVerbatimLanguage",
          "lexVerbatimAttribute",
        },
        tokenModifiers = {},
        formats = { "relative" },
        requests = {
          full = true,
        },
      }

      -- Register the Lex LSP server
      local lspconfig = require("lspconfig")
      local configs = require("lspconfig.configs")

      -- Check if lex LSP config already exists
      if not configs.lex_lsp then
        configs.lex_lsp = {
          default_config = {
            cmd = { "lex-lsp" },
            filetypes = { "lex" },
            root_dir = function(fname)
              return lspconfig.util.find_git_ancestor(fname) or vim.fn.getcwd()
            end,
            settings = {},
          },
        }
      end

      -- Start the LSP server for .lex files
      lspconfig.lex_lsp.setup({
        capabilities = capabilities,
        on_attach = function(client, bufnr)
          -- Enable completion triggered by <c-x><c-o>
          vim.api.nvim_buf_set_option(bufnr, 'omnifunc', 'v:lua.vim.lsp.omnifunc')

          -- Buffer-local keymaps
          local bufopts = { noremap = true, silent = true, buffer = bufnr }
          vim.keymap.set('n', 'K', vim.lsp.buf.hover, bufopts)
          vim.keymap.set('n', 'gd', vim.lsp.buf.definition, bufopts)
          vim.keymap.set('n', 'gD', vim.lsp.buf.declaration, bufopts)
          vim.keymap.set('n', 'gr', vim.lsp.buf.references, bufopts)
          vim.keymap.set('n', '<leader>rn', vim.lsp.buf.rename, bufopts)
          vim.keymap.set('n', '<leader>ca', vim.lsp.buf.code_action, bufopts)

          -- Enable semantic token highlighting
          if client.server_capabilities.semanticTokensProvider then
            vim.lsp.semantic_tokens.start(bufnr, client.id)
          end

          -- Apply Lex theme when LSP attaches
          local theme_name = (vim.o.background == "light") and "lex-light" or "lex-dark"
          local ok, theme = pcall(require, "themes." .. theme_name)
          if ok and type(theme.apply) == "function" then
            theme.apply()
          end
        end,
      })
    end,
  },

  -- Treesitter for syntax highlighting (optional for now)
  {
    "nvim-treesitter/nvim-treesitter",
    build = ":TSUpdate",
    config = function()
      require("nvim-treesitter.configs").setup({
        ensure_installed = {}, -- We'll add lex parser later
        highlight = {
          enable = true,
        },
      })
    end,
  },
})

-- Filetype detection for .lex files
vim.filetype.add({
  extension = {
    lex = "lex",
  },
})
