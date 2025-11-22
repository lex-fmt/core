-- Lex Dark Theme for Neovim
--
-- This theme provides semantic token highlighting for .lex files via LSP.
-- The LSP server emits standard semantic token types (markup.heading, markup.bold, etc.)
-- which we link to markdown highlight groups for maximum theme compatibility.
--
-- See lex-lsp/src/features/semantic_tokens.rs for the LSP token mapping.

local palette = {
  text = "#c3c6cb",
  dim = "#555f6d",
  heading = "#11B7D4",
  list = "#c7910c",
  bold = "#dfa30d",
  italic = "#d46ec0",
  code = "#00a884",
  math = "#38c7bd",
  link = "#11B7D4",
  citation = "#d46ec0",
  footnote = "#d4770c",
  annotation = "#d46ec0",
  annotation_param = "#00b8a9",
  verbatim_bg = "#1c2027",
  verbatim_fg = "#ecedef",
}

-- Map LSP semantic tokens to markdown highlight groups
-- LSP emits standard types like "markup.heading", "markup.bold", etc.
-- These become @lsp.type.markup.heading, @lsp.type.markup.bold in Neovim
local markdown_links = {
  -- Headings (Session titles)
  ["@lsp.type.markup.heading"] = "markdownH1",

  -- Bold text (Definition subjects, inline strong)
  ["@lsp.type.markup.bold"] = "markdownBold",

  -- Italic text (inline emphasis)
  ["@lsp.type.markup.italic"] = "markdownItalic",

  -- Underline (References, citations, footnotes)
  ["@lsp.type.markup.underline"] = "markdownLinkText",

  -- String (inline code, verbatim blocks)
  ["@lsp.type.string"] = "markdownCode",

  -- Number (math)
  ["@lsp.type.number"] = "markdownMath",

  -- Comment (annotations)
  ["@lsp.type.comment"] = "Comment",

  -- Parameter (annotation parameters, verbatim attributes)
  ["@lsp.type.parameter"] = "SpecialComment",

  -- Operator (list markers)
  ["@lsp.type.operator"] = "markdownListMarker",

  -- Type (verbatim language)
  ["@lsp.type.type"] = "markdownCodeDelimiter",
}

-- Fallback colors if markdown groups don't exist
local fallback = {
  ["@lsp.type.markup.heading"] = { fg = palette.heading, bold = true },
  ["@lsp.type.markup.bold"] = { fg = palette.bold, bold = true },
  ["@lsp.type.markup.italic"] = { fg = palette.italic, italic = true },
  ["@lsp.type.markup.underline"] = { fg = palette.link, underline = true },
  ["@lsp.type.string"] = { fg = palette.code },
  ["@lsp.type.number"] = { fg = palette.math },
  ["@lsp.type.comment"] = { fg = palette.annotation, italic = true },
  ["@lsp.type.parameter"] = { fg = palette.annotation_param },
  ["@lsp.type.operator"] = { fg = palette.list },
  ["@lsp.type.type"] = { fg = palette.code, italic = true },
}

local M = {}

function M.apply()
  for group, link in pairs(markdown_links) do
    local applied = false

    -- Check if the target markdown group actually has a definition
    if link and vim.fn.hlexists(link) == 1 then
      local hl = vim.api.nvim_get_hl(0, { name = link, link = false })
      -- Only link if the group has actual color/style attributes
      if next(hl) ~= nil then
        vim.api.nvim_set_hl(0, group, { link = link })
        applied = true
      end
    end

    -- Fall back to custom colors if markdown group doesn't exist or is empty
    if not applied and fallback[group] then
      vim.api.nvim_set_hl(0, group, fallback[group])
    end
  end
end

return M
