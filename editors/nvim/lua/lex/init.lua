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

        -- Map LSP semantic tokens (PascalCase) to Treesitter highlight groups
        local links = {
          ["@lsp.type.SessionTitle"] = "@markup.heading",
          ["@lsp.type.SessionMarker"] = "@punctuation.definition.heading",
          ["@lsp.type.SessionTitleText"] = "@markup.heading",
          ["@lsp.type.DefinitionSubject"] = "@variable.member", -- or @variable.other.definition
          ["@lsp.type.DefinitionContent"] = "@text",
          ["@lsp.type.ListMarker"] = "@punctuation.definition.list",
          ["@lsp.type.ListItemText"] = "@markup.list",
          ["@lsp.type.AnnotationLabel"] = "@comment.note",
          ["@lsp.type.AnnotationParameter"] = "@variable.parameter",
          ["@lsp.type.AnnotationContent"] = "@comment",
          ["@lsp.type.InlineStrong"] = "@markup.strong",
          ["@lsp.type.InlineEmphasis"] = "@markup.italic",
          ["@lsp.type.InlineCode"] = "@markup.raw",
          ["@lsp.type.InlineMath"] = "@constant.numeric", -- or @markup.math
          ["@lsp.type.Reference"] = "@markup.link",
          ["@lsp.type.ReferenceCitation"] = "@markup.link.label",
          ["@lsp.type.ReferenceFootnote"] = "@markup.link.label",
          ["@lsp.type.VerbatimSubject"] = "@label",
          ["@lsp.type.VerbatimLanguage"] = "@keyword",
          ["@lsp.type.VerbatimAttribute"] = "@variable.parameter",
          ["@lsp.type.VerbatimContent"] = "@markup.raw.block",
          ["@lsp.type.InlineMarker.strong.start"] = "@punctuation.delimiter",
          ["@lsp.type.InlineMarker.strong.end"] = "@punctuation.delimiter",
          ["@lsp.type.InlineMarker.emphasis.start"] = "@punctuation.delimiter",
          ["@lsp.type.InlineMarker.emphasis.end"] = "@punctuation.delimiter",
          ["@lsp.type.InlineMarker.code.start"] = "@punctuation.delimiter",
          ["@lsp.type.InlineMarker.code.end"] = "@punctuation.delimiter",
          ["@lsp.type.InlineMarker.math.start"] = "@punctuation.delimiter",
          ["@lsp.type.InlineMarker.math.end"] = "@punctuation.delimiter",
          ["@lsp.type.InlineMarker.ref.start"] = "@punctuation.delimiter",
          ["@lsp.type.InlineMarker.ref.end"] = "@punctuation.delimiter",
        }

        for lsp_hl, ts_hl in pairs(links) do
          -- Only set if not already defined (allow user overrides)
          if vim.fn.hlexists(lsp_hl) == 0 then
            vim.api.nvim_set_hl(0, lsp_hl, { link = ts_hl, default = true })
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
end

return M
