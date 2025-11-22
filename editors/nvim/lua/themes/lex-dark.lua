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

-- Map Lex semantic tokens to standard markdown highlight groups
-- Based on element mapping: Session→Heading, Definition→Bold+Colon, etc.
local markdown_links = {
  -- Session titles map to markdown headings
  ["@lsp.type.lexSessionTitle"] = "markdownH1",

  -- Definition subjects map to bold term (as in **Term**: description pattern)
  ["@lsp.type.lexDefinitionSubject"] = "markdownBold",

  -- List markers map directly
  ["@lsp.type.lexListMarker"] = "markdownListMarker",

  -- Annotations map to HTML comments (not visible in rendered markdown)
  ["@lsp.type.lexAnnotationLabel"] = "Comment",
  ["@lsp.type.lexAnnotationParameter"] = "SpecialComment",

  -- Inline formatting maps directly to markdown inlines
  ["@lsp.type.lexInlineStrong"] = "markdownBold",
  ["@lsp.type.lexInlineEmphasis"] = "markdownItalic",
  ["@lsp.type.lexInlineCode"] = "markdownCode",
  ["@lsp.type.lexInlineMath"] = "markdownMath",

  -- References - Lex uses citations not URLs, so map to link text
  ["@lsp.type.lexReference"] = "markdownLinkText",
  ["@lsp.type.lexReferenceCitation"] = "markdownLinkText",
  ["@lsp.type.lexReferenceFootnote"] = "markdownFootnote",

  -- Verbatim blocks map to code blocks
  ["@lsp.type.lexVerbatimSubject"] = "markdownCodeBlock",
  ["@lsp.type.lexVerbatimLanguage"] = "markdownCodeDelimiter",
  ["@lsp.type.lexVerbatimAttribute"] = "SpecialComment",
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
