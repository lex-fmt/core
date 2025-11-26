-- Lex Neovim Plugin
-- ===================
--
-- Main entry point for the Lex language plugin providing:
-- - LSP integration via nvim-lspconfig
-- - Semantic token highlighting (parser-driven, not regex-based)
-- - Filetype detection for .lex files
--
-- HIGHLIGHTING MODEL
-- ------------------
-- Lex uses intentionally MONOCHROME (grayscale) highlighting to keep focus
-- on document structure rather than colorful syntax. We define our own colors
-- but adapt to dark/light mode via vim.o.background.
--
-- Four intensity levels:
--   NORMAL:   Full contrast - content readers focus on (black/white)
--   MUTED:    Medium contrast - structural elements (50% gray)
--   FAINT:    Low contrast - meta-information (70% gray)
--   FAINTEST: Minimal contrast - syntax markers (80% gray)
--
-- Users can override @lex.normal, @lex.muted, @lex.faint, @lex.faintest
-- to customize the grayscale palette while keeping Lex's typography.
--
-- See: lex-analysis/src/semantic_tokens.rs for token type definitions
-- See: editors/vscode/themes/lex-light.json for reference colors
-- See: README.lex for user documentation
--
-- USAGE
-- -----
--   require("lex").setup({
--     cmd = {"lex-lsp"},  -- command to start LSP server
--   })
--
-- KNOWN LIMITATIONS
-- -----------------
-- - Verbatim blocks don't have embedded language highlighting.
--   The block structure is highlighted but content uses generic styling.
--   Would require treesitter grammar with injection queries.

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

        -- MONOCHROME HIGHLIGHTING
        -- ========================
        -- Lex uses intentionally monochrome (grayscale) highlighting to keep
        -- focus on document structure rather than colorful syntax. We define
        -- our own colors but respect dark/light mode via vim.o.background.
        --
        -- Three intensity levels:
        --   NORMAL:   Full contrast text (content readers focus on)
        --   MUTED:    Medium contrast (structural elements, navigation)
        --   FAINT:    Low contrast (meta-info, syntax markers)
        --
        -- Users can override @lex.normal, @lex.muted, @lex.faint for custom colors.

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

        -- MUTED intensity: structural elements (markers, references)
        vim.api.nvim_set_hl(0, "@lsp.type.SessionTitle", { fg = colors.muted, bold = true })
        vim.api.nvim_set_hl(0, "@lsp.type.SessionMarker", { fg = colors.muted, italic = true })
        vim.api.nvim_set_hl(0, "@lsp.type.ListMarker", { fg = colors.muted, italic = true })
        vim.api.nvim_set_hl(0, "@lsp.type.ListItemText", { fg = colors.muted, italic = true })
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
        -- These should nearly disappear, leaving just the formatted text visible
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
      -- Comment support - Lex uses annotations for comments
      vim.bo.commentstring = ":: comment :: %s"
      vim.bo.comments = ""

      -- Document editing settings - soft wrap at window width
      vim.wo.wrap = true        -- Soft wrap long lines at window width
      vim.wo.linebreak = true   -- Break at word boundaries, not mid-word
      vim.bo.textwidth = 0      -- No hard wrapping (no auto line breaks)
    end,
  })

  -- CRITICAL: Disable built-in syntax highlighting for .lex files
  -- Neovim has a built-in lex.vim syntax file for the Unix lexer tool (flex/lex)
  -- which conflicts with our LSP semantic tokens. We rely entirely on LSP.
  --
  -- This is tricky because:
  -- 1. FileType autocmd runs, but syntax file may load after
  -- 2. Other plugins (like NvChad) may re-enable syntax
  -- 3. BufEnter may re-apply syntax settings
  --
  -- Solution: Use multiple events + vim.schedule() to run AFTER all other autocmds
  local function disable_lex_syntax()
    if vim.bo.filetype == "lex" and vim.bo.syntax ~= "" then
      -- Use both methods to ensure syntax is fully cleared
      vim.bo.syntax = ""
      vim.cmd("syntax clear")
    end
  end

  vim.api.nvim_create_autocmd({ "FileType" }, {
    group = augroup,
    pattern = "lex",
    callback = function()
      -- Schedule to run after all other FileType autocmds
      vim.schedule(disable_lex_syntax)
    end,
  })

  vim.api.nvim_create_autocmd({ "BufEnter", "BufWinEnter", "Syntax" }, {
    group = augroup,
    pattern = { "*.lex", "lex" },
    callback = function()
      -- Schedule to run after all other autocmds for these events
      vim.schedule(disable_lex_syntax)
    end,
  })
end

return M
