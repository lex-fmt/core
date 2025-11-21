-- Filetype detection for .lex files
vim.api.nvim_create_autocmd({ "BufRead", "BufNewFile" }, {
  pattern = "*.lex",
  callback = function()
    vim.bo.filetype = "lex"
  end,
})
