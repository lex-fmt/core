-- Minimal init.lua for LSP testing (without auto-configuring LSP)
-- This provides just the basic infrastructure without starting any LSP servers

-- Add the plugin directory to the runtime path
local script_path = debug.getinfo(1).source:sub(2)
local config_dir = vim.fn.fnamemodify(script_path, ":p:h")
local plugin_dir = vim.fn.fnamemodify(config_dir, ":h")
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

-- Setup lazy.nvim with just lspconfig (no auto-setup)
require("lazy").setup({
  {
    "neovim/nvim-lspconfig",
  },
})

-- Filetype detection for .lex files
vim.filetype.add({
  extension = {
    lex = "lex",
  },
})
