-- Lex plugin entry point
-- This file is auto-loaded by Neovim when the plugin is in the runtimepath
-- For users who load plugins with lazy.nvim, this file won't be used
-- (lazy.nvim will call require("lex").setup() instead)

-- Only auto-setup if the user hasn't already called setup manually
if vim.g.lex_plugin_loaded then
  return
end
vim.g.lex_plugin_loaded = 1

-- Auto-setup with defaults if not using a plugin manager
-- Users with lazy.nvim or other plugin managers should call require("lex").setup() themselves
local ok, lex = pcall(require, "lex")
if ok then
  lex.setup()
end
