Neovim Plugin Fasttrack

This is a high-level overview of how the Lex Neovim plugin is designed.

1. Plugin Architecture

	The Neovim plugin is a thin integration layer that connects the lex-lsp server to Neovim's LSP client.

	Key components:
	- lua/lex/init.lua: Main plugin entry point, handles setup and LSP registration
	- plugin/lex.lua: Auto-loader for users not using plugin managers
	- ftdetect/lex.lua: Filetype detection for .lex files
	- lua/themes/: Color schemes for semantic token highlighting

	This design enables:
	- Zero-config LSP integration (just install and it works)
	- Automatic semantic token highlighting without traditional syntax files
	- Theme customization while maintaining familiar Markdown-like colors
	- Standard Neovim plugin conventions for easy distribution


2. LSP Integration Flow

	The plugin follows Neovim's standard LSP client pattern:

	1. Filetype Detection
		When a .lex file is opened, ftdetect/lex.lua sets filetype=lex

	2. Plugin Setup
		plugin/lex.lua or user's lazy.nvim calls require("lex").setup()

	3. LSP Registration
		The setup function registers lex-lsp with lspconfig

	4. Client Connection
		Neovim's LSP client spawns lex-lsp process and connects via stdio

	5. Semantic Token Activation
		In on_attach callback, vim.lsp.semantic_tokens.start() is called
		This is CRITICAL - without it, .lex files get C syntax highlighting

	6. Feature Availability
		All Phase 1 features (hover, symbols, folding) work automatically
		User presses K for hover, uses native vim.lsp.buf functions, etc.


3. Semantic Token Highlighting

	Lex uses LSP semantic tokens instead of traditional Vim syntax files.

	Why semantic tokens:
	- Parser-driven: Highlighting matches actual AST structure, not regex patterns
	- Consistent: Same highlighting logic as other editors using lex-lsp
	- Accurate: Parser knows the difference between a session title and paragraph
	- Extensible: New Lex features automatically get correct highlighting

	How it works:
	1. lex-lsp parses document and emits semantic token data
	2. Neovim's LSP client receives tokens and applies highlight groups
	3. Theme defines colors for token types (@lsp.type.lexSessionTitle, etc.)
	4. Visual result: sessions bold, inline formatting styled, references colored

	Key insight: The plugin MUST call vim.lsp.semantic_tokens.start() in on_attach.
	Without this, Neovim defaults to filetype-based syntax, which doesn't exist for .lex.


4. Theme System

	Themes map semantic token types to highlight groups with colors.

	Available themes:
	- lex-dark: Dark background theme, borrowing Markdown color palette
	- lex-light: Light background theme

	Theme structure (see lua/themes/lex-dark.lua):
	- Defines vim.api.nvim_set_hl() for each @lsp.type.* token
	- Groups: lexSessionTitle, lexSessionNumber, lexAnnotation, etc.
	- Inline: lexBold, lexItalic, lexCode, lexMath
	- References: lexFootnote, lexCitation, lexReference

	Auto-selection:
	- Default: lex-dark if vim.o.background == "dark", else lex-light
	- Override: require("lex").setup({ theme = "lex-dark" })
	- Disable: require("lex").setup({ theme = false })


5. Testing Strategy

	The plugin has three testing approaches:

	Headless Tests (test/*.lua):
	- Run via Bats (test suite runner)
	- Test LSP attachment, semantic tokens, document symbols, etc.
	- CI-friendly with JUnit output
	- Fast: No GUI, no manual interaction

	Interactive Manual Tests:
	- test_phase1_interactive.sh: Guided walkthrough of all Phase 1 features
	- PHASE1_QUICK_CHECK.lex: Quick reference for manual verification
	- Useful for visual confirmation and debugging

	LSP Command Runner (test/run_lsp_command.sh):
	- Execute any LSP request at a specific file position
	- Returns raw LSP response for inspection
	- Example: ./test/run_lsp_command.sh file.lex 10,5 'vim.lsp.buf.hover()'
	- Critical for debugging hover, symbols, etc.


6. Phase 2 Implementation Guide

	Phase 2 adds navigation features (see lex-lsp/src/lib.rs for details).

	Go to Definition (textDocument/definition):

		LSP side:
		- Implement in lex-lsp/src/features/definition.rs
		- For references [TK-foo], find definition "foo:"
		- For footnotes [42], find annotation with label="42"
		- Return Location with range of target element

		Neovim side:
		- Zero changes needed - vim.lsp.buf.definition() works automatically
		- User presses gd (or custom mapping) to jump to definition
		- Test: Cursor on [TK-foo], press gd, jumps to "foo:" definition

	Find References (textDocument/references):

		LSP side:
		- Implement in lex-lsp/src/features/references.rs
		- Find all [TK-foo], [42], [@cite] that reference the target
		- Walk AST collecting references to given annotation/definition
		- Return Vec<Location> for all reference sites

		Neovim side:
		- Zero changes - vim.lsp.buf.references() populates quickfix
		- User presses gr, sees list of all references
		- Test: Cursor on definition, press gr, sees all usages

	Document Links (textDocument/documentLink):

		LSP side:
		- Implement in lex-lsp/src/features/document_links.rs
		- Extract URLs from text: [url:https://example.com]
		- Extract file references: [file:./image.png]
		- Return DocumentLink with target URI

		Neovim side:
		- Links become clickable in supporting plugins
		- gx opens link in browser (standard Neovim)
			- Useful for images, includes in verbatim blocks

	Phase 2 verification:

		- The editors/nvim/test/fixtures/example.lex fixture now contains definitions, references, and external links so the Lua tests exercise real-world data.
		- New headless scripts test_lsp_definition.lua, test_lsp_references.lua, and test_lsp_document_links.lua call vim.lsp.buf_request_sync() and assert the returned locations/targets. The Bats suite runs them automatically, catching regressions before CI.


7. Phase 3 Implementation Guide

	Phase 3 adds document formatting (see lex-lsp/src/lib.rs for details).

	Document Formatting (textDocument/formatting):

		LSP side:
		- Implement in lex-lsp/src/features/formatting.rs
		- Fix indentation to match Lex indentation rules
		- Normalize blank lines (collapse multiple, ensure separators)
		- Align list markers to consistent column
		- Return Vec<TextEdit> with all changes

		Neovim side:
		- vim.lsp.buf.format() works automatically once the LSP attaches
		- The plugin now exposes <leader>lf as a discoverable shortcut that calls vim.lsp.buf.format({ async = true }) when the server advertises documentFormattingProvider
		- The headless test_lsp_formatting.lua script rewrites fixtures, calls vim.lsp.buf.format(), and verifies the buffer matches lex CLI output so regressions are caught in CI

	Range Formatting (textDocument/rangeFormatting):

		LSP side:
		- Same as formatting but only for selected range
		- Useful for fixing one section without touching rest of document

		Neovim side:
		- Visual select lines, run :lua vim.lsp.buf.range_formatting() or rely on any mappings the user adds
		- Only the selection changes; the Lua test confirms the untouched prefix/suffix remain byte-for-byte identical to the original fixture


Key files:
- editors/nvim/lua/lex/init.lua - Plugin setup and LSP registration
- editors/nvim/lua/themes/lex-dark.lua - Dark theme with semantic token colors
- editors/nvim/plugin/lex.lua - Auto-loader for traditional plugin users
- editors/nvim/test/run_lsp_command.sh - LSP debugging utility
- docs/dev/guides/lsp-plugins.lex - Detailed design documentation
