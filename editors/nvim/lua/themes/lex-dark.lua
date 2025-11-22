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

local markdown_links = {
  ["@lsp.type.lexSessionTitle"] = "markdownHeadingDelimiter",
  ["@lsp.type.lexDefinitionSubject"] = "markdownBold",
  ["@lsp.type.lexListMarker"] = "markdownListMarker",
  ["@lsp.type.lexAnnotationLabel"] = "markdownLinkText",
  ["@lsp.type.lexAnnotationParameter"] = "markdownUrl",
  ["@lsp.type.lexInlineStrong"] = "markdownBold",
  ["@lsp.type.lexInlineEmphasis"] = "markdownItalic",
  ["@lsp.type.lexInlineCode"] = "markdownCode",
  ["@lsp.type.lexInlineMath"] = "markdownCodeInline",
  ["@lsp.type.lexReference"] = "markdownLinkText",
  ["@lsp.type.lexReferenceCitation"] = "markdownLinkText",
  ["@lsp.type.lexReferenceFootnote"] = "markdownFootnote",
  ["@lsp.type.lexVerbatimSubject"] = "markdownCodeBlock",
  ["@lsp.type.lexVerbatimLanguage"] = "markdownCodeBlock",
  ["@lsp.type.lexVerbatimAttribute"] = "markdownUrl",
}

local fallback = {
  ["@lsp.type.lexSessionTitle"] = { fg = palette.heading, bold = true },
  ["@lsp.type.lexDefinitionSubject"] = { fg = palette.bold, bold = true },
  ["@lsp.type.lexListMarker"] = { fg = palette.list },
  ["@lsp.type.lexAnnotationLabel"] = { fg = palette.annotation, italic = true },
  ["@lsp.type.lexAnnotationParameter"] = { fg = palette.annotation_param },
  ["@lsp.type.lexInlineStrong"] = { fg = palette.bold, bold = true },
  ["@lsp.type.lexInlineEmphasis"] = { fg = palette.italic, italic = true },
  ["@lsp.type.lexInlineCode"] = { fg = palette.code },
  ["@lsp.type.lexInlineMath"] = { fg = palette.math },
  ["@lsp.type.lexReference"] = { fg = palette.link, underline = true },
  ["@lsp.type.lexReferenceCitation"] = { fg = palette.citation, underline = true },
  ["@lsp.type.lexReferenceFootnote"] = { fg = palette.footnote },
  ["@lsp.type.lexVerbatimSubject"] = { fg = palette.verbatim_fg, bg = palette.verbatim_bg },
  ["@lsp.type.lexVerbatimLanguage"] = { fg = palette.code, italic = true },
  ["@lsp.type.lexVerbatimAttribute"] = { fg = palette.annotation_param },
}

local M = {}

function M.apply()
  for group, link in pairs(markdown_links) do
    local applied = false
    if link and vim.fn.hlexists(link) == 1 then
      vim.api.nvim_set_hl(0, group, { link = link })
      applied = true
    end
    if not applied and fallback[group] then
      vim.api.nvim_set_hl(0, group, fallback[group])
    end
  end
end

return M
