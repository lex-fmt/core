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

-- Filetype detection for .lex files
vim.filetype.add({
  extension = {
    lex = "lex",
  },
})
