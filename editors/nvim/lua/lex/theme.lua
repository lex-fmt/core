-- Lex Theme Module
-- =================
-- Handles syntax highlighting for Lex documents.
--
-- Two themes available:
--   "monochrome" - Grayscale highlighting (default)
--   "native"     - Links to standard treesitter groups

local M = {}

-- Apply native theme: links to standard treesitter/markup groups
function M.apply_native()
  -- Content text - use standard markup groups
  vim.api.nvim_set_hl(0, "@lsp.type.SessionTitleText", { link = "@markup.heading", default = true })
  vim.api.nvim_set_hl(0, "@lsp.type.DefinitionSubject", { link = "@markup.heading", default = true })
  vim.api.nvim_set_hl(0, "@lsp.type.DefinitionContent", { link = "@markup", default = true })
  vim.api.nvim_set_hl(0, "@lsp.type.InlineStrong", { link = "@markup.strong", default = true })
  vim.api.nvim_set_hl(0, "@lsp.type.InlineEmphasis", { link = "@markup.italic", default = true })
  vim.api.nvim_set_hl(0, "@lsp.type.InlineCode", { link = "@markup.raw", default = true })
  vim.api.nvim_set_hl(0, "@lsp.type.InlineMath", { link = "@markup.math", default = true })
  vim.api.nvim_set_hl(0, "@lsp.type.VerbatimContent", { link = "@markup.raw.block", default = true })
  vim.api.nvim_set_hl(0, "@lsp.type.ListItemText", { link = "@markup", default = true })

  -- Structural elements - use punctuation/delimiter groups
  vim.api.nvim_set_hl(0, "@lsp.type.DocumentTitle", { link = "@markup.heading", default = true })
  vim.api.nvim_set_hl(0, "@lsp.type.SessionMarker", { link = "@punctuation.special", default = true })
  vim.api.nvim_set_hl(0, "@lsp.type.ListMarker", { link = "@markup.list", default = true })

  -- References - use link groups
  vim.api.nvim_set_hl(0, "@lsp.type.Reference", { link = "@markup.link", default = true })
  vim.api.nvim_set_hl(0, "@lsp.type.ReferenceCitation", { link = "@markup.link", default = true })
  vim.api.nvim_set_hl(0, "@lsp.type.ReferenceFootnote", { link = "@markup.link", default = true })

  -- Meta-information - use comment group
  vim.api.nvim_set_hl(0, "@lsp.type.AnnotationLabel", { link = "@comment", default = true })
  vim.api.nvim_set_hl(0, "@lsp.type.AnnotationParameter", { link = "@comment", default = true })
  vim.api.nvim_set_hl(0, "@lsp.type.AnnotationContent", { link = "@comment", default = true })
  vim.api.nvim_set_hl(0, "@lsp.type.VerbatimSubject", { link = "@label", default = true })
  vim.api.nvim_set_hl(0, "@lsp.type.VerbatimLanguage", { link = "@label", default = true })
  vim.api.nvim_set_hl(0, "@lsp.type.VerbatimAttribute", { link = "@attribute", default = true })

  -- Inline markers - use punctuation delimiter
  local marker_groups = {
    "InlineMarker_strong_start", "InlineMarker_strong_end",
    "InlineMarker_emphasis_start", "InlineMarker_emphasis_end",
    "InlineMarker_code_start", "InlineMarker_code_end",
    "InlineMarker_math_start", "InlineMarker_math_end",
    "InlineMarker_ref_start", "InlineMarker_ref_end",
  }
  for _, name in ipairs(marker_groups) do
    vim.api.nvim_set_hl(0, "@lsp.type." .. name, { link = "@punctuation.delimiter", default = true })
  end
end

-- Apply monochrome theme: grayscale highlighting that adapts to dark/light mode
function M.apply_monochrome()
  local is_dark = vim.o.background == "dark"
  local colors = is_dark and {
    normal = "#e0e0e0",   -- light gray on dark bg
    muted = "#888888",    -- medium gray
    faint = "#666666",    -- darker gray
    faintest = "#555555", -- darkest gray for markers
    code_bg = "#2a2a2a",  -- subtle dark bg for code
  } or {
    normal = "#000000",   -- black on light bg
    muted = "#808080",    -- medium gray
    faint = "#b3b3b3",    -- light gray
    faintest = "#cacaca", -- lightest gray for markers
    code_bg = "#f5f5f5",  -- subtle light bg for code
  }

  -- Define base intensity groups (user-overridable)
  vim.api.nvim_set_hl(0, "@lex.normal", { fg = colors.normal, default = true })
  vim.api.nvim_set_hl(0, "@lex.muted", { fg = colors.muted, default = true })
  vim.api.nvim_set_hl(0, "@lex.faint", { fg = colors.faint, default = true })
  vim.api.nvim_set_hl(0, "@lex.faintest", { fg = colors.faintest, default = true })

  -- NORMAL intensity: content text readers focus on
  vim.api.nvim_set_hl(0, "@lsp.type.SessionTitleText", { fg = colors.normal, bold = true })
  vim.api.nvim_set_hl(0, "@lsp.type.DefinitionSubject", { fg = colors.normal, italic = true })
  vim.api.nvim_set_hl(0, "@lsp.type.DefinitionContent", { fg = colors.normal })
  vim.api.nvim_set_hl(0, "@lsp.type.InlineStrong", { fg = colors.normal, bold = true })
  vim.api.nvim_set_hl(0, "@lsp.type.InlineEmphasis", { fg = colors.normal, italic = true })
  vim.api.nvim_set_hl(0, "@lsp.type.InlineCode", { fg = colors.normal })
  vim.api.nvim_set_hl(0, "@lsp.type.InlineMath", { fg = colors.normal, italic = true })
  vim.api.nvim_set_hl(0, "@lsp.type.VerbatimContent", { fg = colors.normal, bg = colors.code_bg })
  vim.api.nvim_set_hl(0, "@lsp.type.ListItemText", { fg = colors.normal })

  -- MUTED intensity: structural elements (markers, references)
  vim.api.nvim_set_hl(0, "@lsp.type.DocumentTitle", { fg = colors.muted, bold = true })
  vim.api.nvim_set_hl(0, "@lsp.type.SessionMarker", { fg = colors.muted, italic = true })
  vim.api.nvim_set_hl(0, "@lsp.type.ListMarker", { fg = colors.muted, italic = true })
  vim.api.nvim_set_hl(0, "@lsp.type.Reference", { fg = colors.muted, underline = true })
  vim.api.nvim_set_hl(0, "@lsp.type.ReferenceCitation", { fg = colors.muted, underline = true })
  vim.api.nvim_set_hl(0, "@lsp.type.ReferenceFootnote", { fg = colors.muted, underline = true })

  -- FAINT intensity: meta-information (annotations, verbatim metadata)
  vim.api.nvim_set_hl(0, "@lsp.type.AnnotationLabel", { fg = colors.faint })
  vim.api.nvim_set_hl(0, "@lsp.type.AnnotationParameter", { fg = colors.faint })
  vim.api.nvim_set_hl(0, "@lsp.type.AnnotationContent", { fg = colors.faint })
  vim.api.nvim_set_hl(0, "@lsp.type.VerbatimSubject", { fg = colors.faint })
  vim.api.nvim_set_hl(0, "@lsp.type.VerbatimLanguage", { fg = colors.faint })
  vim.api.nvim_set_hl(0, "@lsp.type.VerbatimAttribute", { fg = colors.faint })

  -- FAINTEST intensity: inline syntax markers (*, _, `, #, [])
  vim.api.nvim_set_hl(0, "@lsp.type.InlineMarker_strong_start", { fg = colors.faintest, italic = true })
  vim.api.nvim_set_hl(0, "@lsp.type.InlineMarker_strong_end", { fg = colors.faintest, italic = true })
  vim.api.nvim_set_hl(0, "@lsp.type.InlineMarker_emphasis_start", { fg = colors.faintest, italic = true })
  vim.api.nvim_set_hl(0, "@lsp.type.InlineMarker_emphasis_end", { fg = colors.faintest, italic = true })
  vim.api.nvim_set_hl(0, "@lsp.type.InlineMarker_code_start", { fg = colors.faintest, italic = true })
  vim.api.nvim_set_hl(0, "@lsp.type.InlineMarker_code_end", { fg = colors.faintest, italic = true })
  vim.api.nvim_set_hl(0, "@lsp.type.InlineMarker_math_start", { fg = colors.faintest, italic = true })
  vim.api.nvim_set_hl(0, "@lsp.type.InlineMarker_math_end", { fg = colors.faintest, italic = true })
  vim.api.nvim_set_hl(0, "@lsp.type.InlineMarker_ref_start", { fg = colors.faintest, italic = true })
  vim.api.nvim_set_hl(0, "@lsp.type.InlineMarker_ref_end", { fg = colors.faintest, italic = true })
end

-- Apply the specified theme
-- @param theme_name string: "monochrome" (default) or "native"
function M.apply(theme_name)
  if theme_name == "native" then
    M.apply_native()
  else
    M.apply_monochrome()
  end
end

return M
