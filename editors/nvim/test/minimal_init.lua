-- Minimal init.lua for testing the Lex plugin
-- This config bootstraps dependencies and loads the Lex plugin as a lazy.nvim local plugin

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

-- Prevent plugin/lex.lua from auto-running since we'll load via lazy.nvim
vim.g.lex_plugin_loaded = 1

-- Enable syntax highlighting and colors BEFORE loading plugins
-- IMPORTANT: These must be set early so syntax files load correctly when filetype is detected
vim.opt.termguicolors = true
vim.cmd("syntax on")

-- Get paths
local script_path = debug.getinfo(1).source:sub(2)
local test_dir = vim.fn.fnamemodify(script_path, ":p:h")
local plugin_dir = vim.fn.fnamemodify(test_dir, ":h")
local project_root = vim.fn.fnamemodify(plugin_dir, ":h:h")
local lex_lsp_path = project_root .. "/target/debug/lex-lsp"

-- Debug: print paths if DEBUG_LEX_INIT is set
if vim.env.DEBUG_LEX_INIT then
  print("script_path: " .. script_path)
  print("test_dir: " .. test_dir)
  print("plugin_dir: " .. plugin_dir)
  print("project_root: " .. project_root)
  print("lex_lsp_path: " .. lex_lsp_path)
  print("LSP binary exists: " .. tostring(vim.fn.filereadable(lex_lsp_path) == 1))
end

-- Setup lazy.nvim with dependencies and the lex plugin
require("lazy").setup({
  -- LSP configuration dependency
  {
    "neovim/nvim-lspconfig",
  },

  -- Load the Lex plugin as a local plugin
  {
    name = "lex",
    dir = plugin_dir,
    config = function()
      require("lex").setup({
        cmd = { lex_lsp_path },  -- Path to lex-lsp binary
        theme = "lex-dark",      -- Apply lex-dark theme (or false to disable)
      })
    end,
  },
})

-- Wait for lazy to finish installing plugins (critical for CI)
local start_time = vim.loop.hrtime()
local timeout_seconds = 120
local timeout_ns = timeout_seconds * 1e9

while true do
  -- Check if lspconfig can be required
  local status, _ = pcall(require, "lspconfig")
  if status then
    break
  end

  -- Check for timeout
  if (vim.loop.hrtime() - start_time) > timeout_ns then
    print("ERROR: Timed out waiting for lspconfig to install")
    vim.cmd("cquit 1")
  end

  -- Wait a bit to let lazy.nvim background tasks run
  vim.wait(100)
end

-- Enable a basic colorscheme for visual testing
vim.cmd("colorscheme default")
