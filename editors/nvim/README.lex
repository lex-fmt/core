Lex Neovim Plugin

Neovim plugin providing LSP integration, semantic highlighting, and editor support for Lex documents.

Quick Start:

	Install the lex-lsp binary and add this plugin to your Neovim config:
	require("lex").setup()
	:: lua

	For detailed setup instructions, see lua/lex/init.lua

Interactive Testing:

	Test all Phase 1 features:
	./editors/nvim/test_phase1_interactive.sh
	:: shell

	Run headless test suite:
	./editors/nvim/test/run_suite.sh
	:: shell

Documentation:

	- docs/dev/nvim-fasttrack.lex: Quick architecture overview
	- docs/dev/guides/lsp-plugins.lex: Detailed design documentation
	- lua/lex/init.lua: Plugin source with inline documentation

Packaging:

	Build a release-ready plugin archive (sets lex_lsp_version to the provided tag):
	./editors/nvim/scripts/build_plugin.sh --version v0.1.10
	:: shell
