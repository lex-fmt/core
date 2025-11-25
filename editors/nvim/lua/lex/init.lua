-- Lex Neovim plugin
-- Main entry point for the Lex language plugin
--
-- This plugin provides LSP integration for Lex documents, including:
-- - Automatic LSP server registration and connection
-- - Semantic token highlighting (parser-driven, not regex-based)
-- - Filetype detection for .lex files
--
-- Usage:
--   require("lex").setup({
--     cmd = {"lex-lsp"},   -- command to start LSP server
--   })
--
-- KNOWN LIMITATIONS:
-- - Verbatim blocks (code blocks) do not have embedded language syntax highlighting.
--   The block structure (:: python label) is highlighted, but the content inside
--   gets generic "string" highlighting instead of Python syntax highlighting.
--   This limitation exists because embedded language support requires either:
--   1. A Lex treesitter grammar with injection queries (like markdown's injections.scm)
--   2. LSP protocol extensions for embedded content (not yet standardized)
--   Traditional vim syntax files with 'syntax include' are not used as Lex relies
--   entirely on LSP semantic tokens for highlighting.
--
-- See docs/dev/nvim-fasttrack.lex for architecture overview
-- See docs/dev/guides/lsp-plugins.lex for detailed design documentation

local binary_manager = require("lex.binary")

local M = {}

-- Plugin version + bundled lex-lsp version (used by binary manager).
M.version = "0.1.0"
M.lex_lsp_version = "v0.1.14"

-- Resolve which lex-lsp binary to execute. When opts.lex_lsp_version (or the
-- default M.lex_lsp_version) is set, we lazily download the correct GitHub
-- release artifact into ${PLUGIN_ROOT}/bin/lex-lsp-vX.Y.Z and reuse it across
-- sessions. Setting lex_lsp_version=nil (or "") keeps the prior behaviour and
-- defers to whatever binary is on PATH – handy for local development.
local function resolve_lsp_cmd(opts)
  if opts.cmd then
    return opts.cmd
  end

  local desired = opts.lex_lsp_version
  if desired == nil then
    desired = M.lex_lsp_version
  end

  if desired and desired ~= "" then
    -- Binaries are stored under ${PLUGIN_ROOT}/bin/lex-lsp-vX.Y.Z(.exe).
    -- They are downloaded lazily on demand and reused across plugin upgrades.
    local path = binary_manager.ensure_binary(desired)
    if path then
      return { path }
    end
  end

  return { "lex-lsp" }
end

-- Setup function called by lazy.nvim or manual setup
function M.setup(opts)
  opts = opts or {}
  local resolved_cmd = resolve_lsp_cmd(opts)

  -- Register .lex filetype
  vim.filetype.add({
    extension = {
      lex = "lex",
    },
  })

  -- Setup LSP if lspconfig is available
  local ok, lspconfig = pcall(require, "lspconfig")
  if ok then
    local configs = require("lspconfig.configs")

    -- Register lex-lsp server config if not already registered
    if not configs.lex_lsp then
      configs.lex_lsp = {
        default_config = {
          cmd = resolved_cmd,
          filetypes = { "lex" },
          root_dir = function(fname)
            return lspconfig.util.find_git_ancestor(fname) or vim.fn.getcwd()
          end,
          settings = opts.settings or {},
        },
      }
    end

    -- Auto-start LSP for .lex files with semantic token support
    local lsp_config = opts.lsp_config or {}
    local user_on_attach = lsp_config.on_attach

    -- Ensure cmd is passed to the LSP config if provided
    if not lsp_config.cmd then
      lsp_config.cmd = resolved_cmd
    end

    lsp_config.on_attach = function(client, bufnr)
      -- CRITICAL: Enable semantic token highlighting for .lex files
      -- .lex files don't use traditional Vim syntax files - they rely on LSP semantic tokens
      -- Without this call, .lex files will fall back to generic C syntax (wrong highlighting)
      -- or show no highlighting at all. This must be called in on_attach after LSP connects.
      if client.server_capabilities.semanticTokensProvider then
        vim.lsp.semantic_tokens.start(bufnr, client.id)

        if opts.debug_theme then
          -- Debug theme: exact colors from editors/vscode/themes/lex-light.json
          -- Useful for testing/debugging semantic token highlighting
          local debug_highlights = {
            ["@lsp.type.SessionTitle"] = { fg = "#000000", bold = true },
            ["@lsp.type.SessionMarker"] = { fg = "#808080", italic = true },
            ["@lsp.type.SessionTitleText"] = { fg = "#000000", bold = true },
            ["@lsp.type.DefinitionSubject"] = { fg = "#000000", italic = true },
            ["@lsp.type.DefinitionContent"] = { fg = "#000000" },
            ["@lsp.type.ListMarker"] = { fg = "#666666", italic = true },
            ["@lsp.type.ListItemText"] = { fg = "#808080", italic = true },
            ["@lsp.type.AnnotationLabel"] = { fg = "#b3b3b3" },
            ["@lsp.type.AnnotationParameter"] = { fg = "#b3b3b3" },
            ["@lsp.type.AnnotationContent"] = { fg = "#b3b3b3" },
            ["@lsp.type.InlineStrong"] = { fg = "#000000", bold = true },
            ["@lsp.type.InlineEmphasis"] = { fg = "#000000", italic = true },
            ["@lsp.type.InlineCode"] = { fg = "#000000" },
            ["@lsp.type.InlineMath"] = { fg = "#000000", italic = true },
            ["@lsp.type.Reference"] = { fg = "#808080", underline = true },
            ["@lsp.type.ReferenceCitation"] = { fg = "#808080", underline = true },
            ["@lsp.type.ReferenceFootnote"] = { fg = "#808080", underline = true },
            ["@lsp.type.VerbatimSubject"] = { fg = "#b3b3b3" },
            ["@lsp.type.VerbatimLanguage"] = { fg = "#b3b3b3" },
            ["@lsp.type.VerbatimAttribute"] = { fg = "#b3b3b3" },
            ["@lsp.type.VerbatimContent"] = { fg = "#000000", bg = "#f5f5f5" },
            ["@lsp.type.InlineMarker_strong_start"] = { fg = "#cacaca", italic = true },
            ["@lsp.type.InlineMarker_strong_end"] = { fg = "#cacaca", italic = true },
            ["@lsp.type.InlineMarker_emphasis_start"] = { fg = "#cacaca", italic = true },
            ["@lsp.type.InlineMarker_emphasis_end"] = { fg = "#cacaca", italic = true },
            ["@lsp.type.InlineMarker_code_start"] = { fg = "#cacaca", italic = true },
            ["@lsp.type.InlineMarker_code_end"] = { fg = "#cacaca", italic = true },
            ["@lsp.type.InlineMarker_math_start"] = { fg = "#cacaca", italic = true },
            ["@lsp.type.InlineMarker_math_end"] = { fg = "#cacaca", italic = true },
            ["@lsp.type.InlineMarker_ref_start"] = { fg = "#cacaca", italic = true },
            ["@lsp.type.InlineMarker_ref_end"] = { fg = "#cacaca", italic = true },
          }
          for lsp_hl, hl_opts in pairs(debug_highlights) do
            vim.api.nvim_set_hl(0, lsp_hl, hl_opts)
          end
        else
          -- Production: link to Treesitter highlight groups for theme compatibility
          local links = {
            ["@lsp.type.SessionTitle"] = "@markup.heading",
            ["@lsp.type.SessionMarker"] = "@punctuation.definition.heading",
            ["@lsp.type.SessionTitleText"] = "@markup.heading",
            ["@lsp.type.DefinitionSubject"] = "@variable.member",
            ["@lsp.type.DefinitionContent"] = "@text",
            ["@lsp.type.ListMarker"] = "@punctuation.definition.list",
            ["@lsp.type.ListItemText"] = "@markup.list",
            ["@lsp.type.AnnotationLabel"] = "@comment.note",
            ["@lsp.type.AnnotationParameter"] = "@variable.parameter",
            ["@lsp.type.AnnotationContent"] = "@comment",
            ["@lsp.type.InlineStrong"] = "@markup.strong",
            ["@lsp.type.InlineEmphasis"] = "@markup.italic",
            ["@lsp.type.InlineCode"] = "@markup.raw",
            ["@lsp.type.InlineMath"] = "@constant.numeric",
            ["@lsp.type.Reference"] = "@markup.link",
            ["@lsp.type.ReferenceCitation"] = "@markup.link.label",
            ["@lsp.type.ReferenceFootnote"] = "@markup.link.label",
            ["@lsp.type.VerbatimSubject"] = "@label",
            ["@lsp.type.VerbatimLanguage"] = "@keyword",
            ["@lsp.type.VerbatimAttribute"] = "@variable.parameter",
            ["@lsp.type.VerbatimContent"] = "@markup.raw.block",
            ["@lsp.type.InlineMarker_strong_start"] = "@punctuation.delimiter",
            ["@lsp.type.InlineMarker_strong_end"] = "@punctuation.delimiter",
            ["@lsp.type.InlineMarker_emphasis_start"] = "@punctuation.delimiter",
            ["@lsp.type.InlineMarker_emphasis_end"] = "@punctuation.delimiter",
            ["@lsp.type.InlineMarker_code_start"] = "@punctuation.delimiter",
            ["@lsp.type.InlineMarker_code_end"] = "@punctuation.delimiter",
            ["@lsp.type.InlineMarker_math_start"] = "@punctuation.delimiter",
            ["@lsp.type.InlineMarker_math_end"] = "@punctuation.delimiter",
            ["@lsp.type.InlineMarker_ref_start"] = "@punctuation.delimiter",
            ["@lsp.type.InlineMarker_ref_end"] = "@punctuation.delimiter",
          }
          for lsp_hl, ts_hl in pairs(links) do
            if vim.fn.hlexists(lsp_hl) == 0 then
              vim.api.nvim_set_hl(0, lsp_hl, { link = ts_hl, default = true })
            end
          end
        end
      end

      -- Preserve user's on_attach callback if they provided one
      if user_on_attach then
        user_on_attach(client, bufnr)
      end
    end

    lspconfig.lex_lsp.setup(lsp_config)
  end

  -- Debug command: inspect semantic token under cursor
  vim.api.nvim_create_user_command("LexDebugToken", function()
    local bufnr = vim.api.nvim_get_current_buf()
    local cursor = vim.api.nvim_win_get_cursor(0)
    local row, col = cursor[1] - 1, cursor[2] -- 0-indexed

    local lines = {}
    table.insert(lines, "=== LexDebugToken ===")
    table.insert(lines, string.format("Cursor: L%d:C%d", row + 1, col + 1))
    table.insert(lines, string.format("Filetype: %s | Syntax: '%s'", vim.bo.filetype, vim.bo.syntax))

    -- LSP client info
    local clients = vim.lsp.get_clients({ bufnr = bufnr })
    table.insert(lines, "")
    table.insert(lines, "-- LSP Clients --")
    if #clients == 0 then
      table.insert(lines, "  (none attached)")
    else
      for _, client in ipairs(clients) do
        local has_st = client.server_capabilities.semanticTokensProvider and "yes" or "no"
        table.insert(lines, string.format("  %s (id=%d) semantic_tokens=%s", client.name, client.id, has_st))
      end
    end

    -- Get semantic tokens at cursor using Neovim's API
    table.insert(lines, "")
    table.insert(lines, "-- Semantic Tokens at Cursor --")
    local found_token = false

    -- Use vim.inspect_pos for comprehensive highlight info
    local inspect = vim.inspect_pos(bufnr, row, col)

    if inspect.semantic_tokens and #inspect.semantic_tokens > 0 then
      for _, token in ipairs(inspect.semantic_tokens) do
        found_token = true
        local hl_group = "@lsp.type." .. token.type
        if token.modifiers and #token.modifiers > 0 then
          hl_group = hl_group .. " (modifiers: " .. table.concat(token.modifiers, ", ") .. ")"
        end
        table.insert(lines, string.format("  Type: %s", token.type))
        table.insert(lines, string.format("  HL Group: @lsp.type.%s", token.type))

        -- Get the highlight definition
        local hl_info = vim.api.nvim_get_hl(0, { name = "@lsp.type." .. token.type })
        if hl_info and next(hl_info) then
          local def_parts = {}
          if hl_info.link then table.insert(def_parts, "link=" .. hl_info.link) end
          if hl_info.fg then table.insert(def_parts, string.format("fg=#%06x", hl_info.fg)) end
          if hl_info.bg then table.insert(def_parts, string.format("bg=#%06x", hl_info.bg)) end
          if hl_info.bold then table.insert(def_parts, "bold") end
          if hl_info.italic then table.insert(def_parts, "italic") end
          if hl_info.underline then table.insert(def_parts, "underline") end
          table.insert(lines, string.format("  HL Def: %s", #def_parts > 0 and table.concat(def_parts, " ") or "(empty!)"))
        else
          table.insert(lines, "  HL Def: (NOT DEFINED)")
        end
      end
    end

    if not found_token then
      table.insert(lines, "  (no semantic token at cursor)")
    end

    -- All highlights at position (treesitter, syntax, etc.)
    table.insert(lines, "")
    table.insert(lines, "-- All Highlights at Cursor --")
    if inspect.treesitter and #inspect.treesitter > 0 then
      for _, ts in ipairs(inspect.treesitter) do
        table.insert(lines, string.format("  treesitter: %s", ts.hl_group))
      end
    end
    if inspect.syntax and #inspect.syntax > 0 then
      for _, syn in ipairs(inspect.syntax) do
        table.insert(lines, string.format("  syntax: %s", syn.hl_group))
      end
    end
    if inspect.extmarks and #inspect.extmarks > 0 then
      for _, ext in ipairs(inspect.extmarks) do
        if ext.opts and ext.opts.hl_group then
          table.insert(lines, string.format("  extmark: %s", ext.opts.hl_group))
        end
      end
    end

    -- Output
    local output = table.concat(lines, "\n")
    print(output)

    -- Also copy to clipboard
    vim.fn.setreg("+", output)
    vim.notify("Debug info copied to clipboard", vim.log.levels.INFO)
  end, { desc = "Debug semantic token under cursor" })

  -- Setup autocommands for .lex files
  local augroup = vim.api.nvim_create_augroup("LexPlugin", { clear = true })

  vim.api.nvim_create_autocmd("FileType", {
    group = augroup,
    pattern = "lex",
    callback = function()
      -- CRITICAL: Disable built-in syntax highlighting for .lex files
      -- Neovim has a built-in lex.vim syntax file for the Unix lexer tool (flex/lex)
      -- which conflicts with our LSP semantic tokens. We rely entirely on LSP for highlighting.
      vim.bo.syntax = ""

      -- Comment support - Lex uses annotations for comments
      vim.bo.commentstring = ":: comment :: %s"
      vim.bo.comments = ""

      -- Document editing settings - soft wrap at window width
      vim.wo.wrap = true        -- Soft wrap long lines at window width
      vim.wo.linebreak = true   -- Break at word boundaries, not mid-word
      vim.bo.textwidth = 0      -- No hard wrapping (no auto line breaks)
    end,
  })
end

return M
