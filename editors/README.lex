Editor Tooling

	We will fully develop the initial version of plugins for Neovim and VSCode.
	While in the future these will be best served by dedicated repos, for now, as we are iterating over various layers in binaries, libs and the plugin themselves, they'll be colocated in the master lex repo.  

	The design's goal is to have all logic-heavy lifting done in common rust code, and the plugins themselves being thin wrappers for each editors ui / entry points and interaction models.

1. Minimal Featureset : Syntax and Language Support

	These are the initial launch features for both editors: 

	1. Syntax Highlighting
	2. Document Symbols
	3. Hover Information
	4. Folding Ranges
    5. Formatting
    6. Comment / Uncomment
	7. Symbol Navigation (mostly references in Lex's context)

	While diagnostics surely would make sense, given's Lex modus operandi of "never fails" and worst case scenario "parse as paragraphs", we don't have a useful , working implementation that would offer any value. This will be tackled later, but it is to be left out for now.

	These are currently working in the Neovim plugin. All of these are built over the LSP protocol, with the lex-lsp binary being the server.


2. Document Handling

	Being a markup document format, there is a set of features that are table stakes: 
	1. Interop:
		- Export to Markdown
		- Import to Markdown
		- Export to HTML
		- HTML Preview
    	- Others to follow (soon will pandoc, pdf, etc)
	2. Content Management: 
		- Generate / update TOC.
        - Generate  footnotes.
        - Citation linking (integration with Zotero, bibtex, etc)

3. Editing

	While the minimal featureset covered some of these, we have left out a few useful features related to editing:

	1. Completion for paths, urls, citations.
	2. Inserting Images / Files.
	3. Inserting Verbatim Blocks from files.
	4. Annotation management: 
		4.1 Iterating through annotations
		4.2 Resolving, unresolving.
        4.3 Show Hide Annotations
    5. Indenting on paste.
	6. Tab shifting.


4. Publishing

	While the interop features will generate various formats, the publishing workflow can be more detailed, alowing template selection, image sizing, previews, and so on. 

	This will be a core part of Lex, as it's value proposition is a single format from note to publication. 


6. Help / Documentation

	Being a novel format, it would be very welcome to be able to offer help and documetation for the format itself. I'm not very sure how to best achieve this, but it's worth carving out the mental model for this.

7. Shared Architecture

	To avoid duplicating logic across plugins, we use the LSP `workspace/executeCommand` capability. This allows plugins to delegate complex tasks to the `lex-lsp` server.

	Mechanism:
	The server exposes a set of commands (e.g., `lex.echo`). Plugins invoke these commands using their editor's LSP client API.

	Usage:

	VSCode:
	Use `vscode.commands.executeCommand('lex.commandName', args)`.

	Neovim:
	Use `client:exec_cmd({ command = 'lex.commandName', arguments = args })` (or `vim.lsp.buf.execute_command` for older versions).