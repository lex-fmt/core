-- Minimal init.lua for testing the Lex plugin
-- This config bootstraps dependencies and loads the Lex plugin as a lazy.nvim local plugin

-- Add the plugin directory to the runtime path early
local script_path = debug.getinfo(1).source:sub(2)
local test_dir = vim.fn.fnamemodify(script_path, ":p:h")
local plugin_dir = vim.fn.fnamemodify(test_dir, ":h")
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

-- Prevent plugin/lex.lua from auto-running since we'll load via lazy.nvim
vim.g.lex_plugin_loaded = 1

-- Enable syntax highlighting and colors BEFORE loading plugins
-- IMPORTANT: These must be set early so syntax files load correctly when filetype is detected
vim.opt.termguicolors = true  -- Required for modern color themes to display properly
vim.cmd("syntax on")           -- Required to enable syntax highlighting for standard filetypes (markdown, etc.)

-- Setup lazy.nvim with just lspconfig (no auto-setup)
require("lazy").setup({
  {
    "neovim/nvim-lspconfig",
  },
})

-- Wait for lazy to finish installing plugins (critical for CI)
local start_time = vim.loop.hrtime()
local timeout_seconds = 120 -- Wait up to 2 minutes in CI
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

-- CRITICAL: Re-add plugin directory to rtp AFTER lazy.nvim setup
-- lazy.nvim modifies the runtimepath during setup, so we must re-add our plugin dir
-- after lazy finishes or the 'lex' module won't be found by require()
vim.opt.rtp:prepend(plugin_dir)

-- Enable a basic colorscheme for visual testing
vim.cmd("colorscheme default")

-- Filetype detection for .lex files
vim.filetype.add({
  extension = {
    lex = "lex",
  },
})

-- Load and setup the Lex plugin
-- This minimal_init.lua bootstraps dependencies (lazy.nvim, lspconfig) and loads our plugin
-- The actual plugin logic (LSP config, semantic tokens) is in lua/lex/init.lua
local project_root = vim.fn.fnamemodify(plugin_dir, ":h:h")
local lex_lsp_path = project_root .. "/target/debug/lex-lsp"

-- Debug: verify paths (set DEBUG_LEX_INIT=1 to enable)
if vim.env.DEBUG_LEX_INIT then
  print("Plugin dir: " .. plugin_dir)
  print("Project root: " .. project_root)
  print("Expected lex module at: " .. plugin_dir .. "/lua/lex/init.lua")
  print("File exists: " .. tostring(vim.fn.filereadable(plugin_dir .. "/lua/lex/init.lua") == 1))
end

-- Setup the lex plugin which will configure LSP and semantic tokens
local ok, lex = pcall(require, "lex")
if ok then
  lex.setup({
    cmd = { lex_lsp_path },  -- Path to lex-lsp binary
    -- debug_theme = true,   -- Uncomment to use lex-light.json colors for visual testing
  })
elseif vim.env.DEBUG_LEX_INIT then
  print("Failed to load lex plugin - semantic tokens won't work for .lex files")
  print("This is expected if lex-lsp binary doesn't exist yet")
end
