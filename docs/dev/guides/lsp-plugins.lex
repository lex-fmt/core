LSP and Editor Plugins

	Lex provides rich editor support through the Language Server Protocol (LSP), enabling features like hover documentation, outline navigation, and semantic highlighting in any LSP-compatible editor. This guide covers the design, architecture, and implementation of the LSP server and editor plugins.


Design Principles:
	- LSP-first: All editor features come from lex-lsp, ensuring consistency across editors
	- Semantic tokens: Parser-driven highlighting, not regex-based syntax files
	- Zero config: Plugins work out of the box with sensible defaults
	- Standards-based: Follow LSP spec and editor conventions


1. Architecture

	This section overviews the LSP server, editor plugins, and how they interact.


	1.1 LSP Server (lex-lsp)

		The lex-lsp server is a standalone Rust binary that implements the Language Server Protocol.

		Key design decisions:
		- Built with tower-lsp framework for async LSP handling
		- Thin server layer that delegates to feature modules
		- Stateless features that operate on Lex AST
		- No editor-specific code in the server

		Architecture layers:
		- LSP Layer (tower-lsp): JSON-RPC, protocol handshaking, request routing
		- Server Layer: Document state management, coordinate feature calls
		- Feature Layer: Individual LSP features (hover, symbols, etc.)
		- Parser Layer: lex-parser crate provides AST

		Why this design:
		- Separation of concerns: LSP protocol vs feature logic vs parsing
		- Testability: Each layer tested independently
		- Maintainability: Features are isolated, easy to add/modify
		- Performance: Stateless features enable parallel processing


	1.2 Editor Plugins

		Editor plugins are thin integration layers that connect lex-lsp to the editor.

		Responsibilities:
		- Register lex-lsp server with editor's LSP client
		- Configure filetype detection for .lex files
		- Apply themes/color schemes for semantic tokens
		- Provide editor-specific conveniences (keybindings, commands)

		What plugins do NOT do:
		- Parse Lex documents (that's lex-lsp's job)
		- Implement LSP features (that's lex-lsp's job)
		- Maintain editor-specific feature implementations

		Why this division:
		- Consistency: All editors get same features from lex-lsp
		- Maintainability: One LSP implementation, not N editor-specific ones
		- Quality: Deep testing of lex-lsp benefits all editors


	1.3 Communication Flow

		How an editor feature works end-to-end:

		1. User opens document.lex in editor
		2. Editor detects .lex filetype (via plugin's ftdetect)
		3. Editor's LSP client spawns lex-lsp process
		4. Client sends initialize request with capabilities
		5. Server responds with its capabilities (hover, symbols, etc.)
		6. Client sends textDocument/didOpen with file content
		7. Server parses content into AST and caches it
		8. User triggers feature (e.g., presses K for hover)
		9. Client sends textDocument/hover request with cursor position
		10. Server's hover feature finds element at position, returns content
		11. Client displays hover popup to user

		Key insight: Server maintains document state, client maintains UI state.


2. LSP Features

	This section details each LSP feature, its purpose, and implementation approach.


	2.1 Phase 1 Features (Complete)

		These features are implemented and working in lex-lsp.


		2.1.1 Semantic Tokens (textDocument/semanticTokens/full)

			Purpose: Provide syntax highlighting based on semantic understanding of code structure.

			How it works:
			- Server walks Lex AST emitting semantic tokens
			- Each AST node type maps to a semantic token type
			- Token types: lexSessionTitle, lexSessionNumber, lexAnnotation, etc.
			- Client applies highlight groups based on token types

			Why semantic tokens:
			- Parser-driven: Highlighting matches actual structure, not regex guesses
			- Accurate: Parser knows if "1." is session number or paragraph text
			- Extensible: New Lex features automatically get correct highlighting
			- Consistent: Same highlighting in VSCode, Neovim, Emacs, etc.

			Implementation (lex-lsp/src/features/semantic_tokens.rs):
			- Walk AST depth-first collecting tokens
			- For each node, emit token with type, modifiers, range
			- Use standard LSP token types where possible (namespace for sessions)
			- Custom types for Lex-specific elements (prefixed with "lex")

			Editor side:
			- Plugin defines highlight groups for each token type
			- Theme applies colors to highlight groups
			- Client automatically requests tokens on file open/change


		2.1.2 Hover Information (textDocument/hover)

			Purpose: Show documentation when cursor is over an element.

			What it shows:
			- Footnote references [42]: Show footnote content
			- Citations [@spec2025]: Show citation details
			- Internal references [TK-foo]: Show definition content
			- Annotations @ note: Show annotation metadata

			Implementation (lex-lsp/src/features/hover.rs):
			- Find AST node at cursor position
			- For references, look up target (annotation, definition, etc.)
			- Extract preview content (first 3 lines of target)
			- Return Markdown-formatted hover content

			Editor side:
			- User presses K (or hover trigger)
			- Client sends textDocument/hover with position
			- Client displays returned Markdown in popup


		2.1.3 Document Symbols (textDocument/documentSymbol)

			Purpose: Provide hierarchical outline of document structure.

			What it shows:
			- Sessions with nesting (1. → 1.1. → 1.1.1.)
			- Definitions (term: content pairs)
			- Annotations (@ note, @ warning, etc.)
			- Lists (root list items)

			Implementation (lex-lsp/src/features/document_symbols.rs):
			- Walk AST collecting DocumentSymbol entries
			- Each session/definition/annotation becomes a symbol
			- Children field maintains hierarchy
			- Symbol kinds: Module (session), Property (definition), Function (annotation)

			Editor side:
			- User opens outline view (Telescope, symbols-outline.nvim, etc.)
			- Client requests symbols on file open
			- Clicking symbol jumps to that location


		2.1.4 Folding Ranges (textDocument/foldingRange)

			Purpose: Allow collapsing/expanding sections of document.

			What can be folded:
			- Sessions (entire section and children)
			- Lists with children
			- Multi-line annotations
			- Verbatim blocks

			Implementation (lex-lsp/src/features/folding.rs):
			- Walk AST collecting foldable ranges
			- Each container node with children becomes a fold
			- Return start/end line numbers for each fold

			Editor side:
			- Client requests folding ranges on file open
			- User presses zc/zo/za to fold/unfold
			- Editor uses LSP ranges for folding, not syntax-based


	2.2 Phase 2 Features (Planned)

		These features add navigation capabilities.


		2.2.1 Go to Definition (textDocument/definition)

			Purpose: Jump from reference to its definition.

			Navigation targets:
			- Footnote [42] → @ note label="42"
			- Citation [@spec2025] → bibliography entry
			- Internal reference [TK-foo] → definition "foo:"

			Implementation approach (lex-lsp/src/features/definition.rs):
			- Parse reference at cursor position (extract label/key)
			- Search AST for matching annotation/definition
			- Return Location of target element
			- Handle multiple matches (e.g., duplicate labels)

			Challenges:
			- Footnote references can be numeric [42] or labeled [^source]
			- Citations can have multiple keys [@foo; @bar]
			- Definitions are case-insensitive matches
			- Need to search across entire document, not just local scope

			Editor side:
			- User presses gd (or equivalent)
			- Client sends textDocument/definition
			- Editor jumps to returned location


		2.2.2 Find References (textDocument/references)

			Purpose: Find all usages of an annotation or definition.

			What it finds:
			- All [42] references to a footnote
			- All [@cite] uses of a bibliography entry
			- All [TK-foo] uses of a definition

			Implementation approach (lex-lsp/src/features/references.rs):
			- Identify target at cursor (annotation label, definition subject)
			- Walk entire AST finding all reference nodes
			- Match reference text against target
			- Return Vec<Location> for all matches

			Challenges:
			- References can be in any text content (paragraphs, lists, etc.)
			- Need inline parsing to extract references from TextContent
			- Handle reference variants ([42] vs [^source])
			- Exclude false positives (random numbers in text)

			Editor side:
			- User presses gr (or equivalent)
			- Client sends textDocument/references
			- Editor populates quickfix list with all references


		2.2.3 Document Links (textDocument/documentLink)

			Purpose: Make URLs and file paths clickable.

			Link types:
			- URLs in text: [url:https://example.com]
			- File references: [file:./image.png]
			- Verbatim src parameters: :: src="./data.csv"

			Implementation approach (lex-lsp/src/features/document_links.rs):
			- Walk AST looking for URL/file references
			- Parse verbatim parameters for src/href attributes
			- Return DocumentLink with target URI
			- Resolve relative paths to absolute

			Challenges:
			- URL detection in free text (avoid false positives)
			- Path resolution relative to document
			- Handle different reference syntaxes

			Editor side:
			- Links become underlined/styled
			- User presses gx (or clicks)
			- Editor opens link in browser/viewer

		Implementation Notes (Phase 2 delivery):

			- Go to definition lives in lex-lsp/src/features/go_to_definition.rs and resolves footnotes, definitions, sessions, and citation keys directly from the AST. Unit tests cover each reference type.
			- Find references is implemented in lex-lsp/src/features/references.rs by scanning inline spans, so references discovered while editing surface immediately. Tests assert both request-side filtering and includeDeclaration semantics.
			- Document links reuse the parser’s DocumentLink extractor. The language server resolves relative file paths based on the buffer URI so gx opens the right file.
			- Neovim’s headless test suite now exercises all three features via test_lsp_definition.lua, test_lsp_references.lua, and test_lsp_document_links.lua. These run as part of editors/nvim/test/lex_nvim_plugin.bats ensuring lua↔︎LSP wiring stays healthy.


	2.3 Phase 3 Features (Future)

		These features add document manipulation.


		2.3.1 Document Formatting (textDocument/formatting)

			Purpose: Automatically fix formatting issues.

			What it fixes:
			- Indentation errors (breaking the indentation wall)
			- Normalize blank lines (collapse multiple, ensure separators)
			- Align list markers to consistent column
			- Fix session number spacing

			Implementation approach (lex-lsp/src/features/formatting.rs):
			- Parse document into AST
			- Walk AST identifying formatting issues
			- Generate TextEdit operations to fix each issue
			- Return Vec<TextEdit> for client to apply

			Challenges:
			- Preserve user's indentation style (tabs vs spaces)
			- Don't break verbatim content (it must not be reformatted)
			- Handle edge cases (nested lists, deep indentation)
			- Provide incremental formatting (range formatting)

			Editor side:
			- User runs :Format or saves with format-on-save
			- Client sends textDocument/formatting
			- Editor applies returned TextEdit operations


		2.3.2 Range Formatting (textDocument/rangeFormatting)

			Purpose: Format only selected region of document.

			Same as document formatting, but constrained to a range.
			Useful for fixing one section without touching rest of file.


	2.4 Phase 4 Features (Future)

		These features add error detection.


		2.4.1 Diagnostics (textDocument/publishDiagnostics)

			Purpose: Show errors and warnings in document.

			What it detects:
			- Indentation errors (content before indentation wall)
			- Malformed structures (single-item lists, unclosed verbatim)
			- Broken references (footnote/citation not found)
			- Invalid annotation syntax

			Implementation approach (lex-lsp/src/features/diagnostics.rs):
			- Parse document, collect errors/warnings
			- Check AST invariants (list has 2+ items)
			- Validate references (all [42] have matching annotations)
			- Return Vec<Diagnostic> with severity and range

			Editor side:
			- Client automatically requests diagnostics on file change
			- Editor shows squiggly underlines, gutter marks
			- User sees error list in problems panel


3. Editor Plugin Implementation

	This section covers how to implement a plugin for a specific editor.


	3.1 Neovim Plugin

		The Neovim plugin (editors/nvim) demonstrates the standard pattern.


		3.1.1 Plugin Structure

			Standard Neovim plugin layout:
			- lua/lex/init.lua: Main plugin, setup() function
			- plugin/lex.lua: Auto-loader for traditional users
			- ftdetect/lex.lua: Filetype detection
			- lua/themes/: Color schemes for semantic tokens

			Setup function (lua/lex/init.lua):
			1. Register .lex filetype
			2. Configure lspconfig with lex-lsp
			3. Set on_attach callback to enable semantic tokens
			4. Apply theme for highlight groups

			Critical: The on_attach must call vim.lsp.semantic_tokens.start().
			Without this, Neovim defaults to generic syntax highlighting.


		3.1.2 Theme System

			Themes map semantic token types to colors.

			Example (lua/themes/lex-dark.lua):
			- Define @lsp.type.lexSessionTitle highlighting
			- Define @lsp.type.lexBold, @lsp.type.lexItalic
			- Define @lsp.mod.* for modifiers

			Auto-selection:
			- Check vim.o.background
			- Load lex-dark for dark, lex-light for light
			- User can override with setup({ theme = "lex-dark" })


		3.1.3 Testing

			Three testing strategies:

			Headless tests (test/*.lua):
			- Run with Bats (Bash test framework)
			- Test LSP attachment, semantic tokens, symbols
			- Fast, CI-friendly

			Manual tests:
			- Interactive walkthrough (test_phase1_interactive.sh)
			- Quick check guide (PHASE1_QUICK_CHECK.lex)
			- Visual confirmation

			LSP debugger (test/run_lsp_command.sh):
			- Execute any LSP request at any position
			- Inspect raw LSP responses
			- Critical for debugging hover, definition, etc.


	3.2 VSCode Plugin (Future)

		VSCode plugin would follow similar pattern:

		Structure:
		- package.json: Extension manifest
		- src/extension.ts: Activation, LSP client setup
		- syntaxes/: TextMate grammar (optional, semantic tokens preferred)

		Key differences from Neovim:
		- Use vscode-languageclient library
		- Theme via contributes.semanticTokenTypes in package.json
		- Testing with VSCode extension test framework


	3.3 Emacs Plugin (Future)

		Emacs plugin with lsp-mode:

		Structure:
		- lex-mode.el: Major mode definition
		- lex-lsp.el: LSP client configuration

		Key points:
		- Use lsp-mode or eglot for LSP client
		- Define faces for semantic token types
		- Register lex-lsp server


4. Semantic Token Design

	Semantic tokens are central to Lex editor support. This section details the design.


	4.1 Why Semantic Tokens

		Traditional syntax highlighting uses regex patterns to guess structure.
		This works for simple languages but fails for Lex:

		Problem: Is "1." a session number or paragraph text?
		- Regex: Match any "digit followed by period" → wrong (catches paragraphs)
		- Semantic: Check if AST node is Session → correct (parser knows)

		Semantic tokens solve this by using parser output, not regex patterns.


	4.2 Token Types

		Standard LSP token types (reused):
		- namespace: Sessions (borrowing module/namespace semantics)
		- property: Definitions (key-value pairs)
		- function: Annotations (callable/metadata semantics)
		- string: Verbatim blocks, inline code
		- comment: Comments (Lex doesn't have comments, reserved for future)

		Custom token types (Lex-specific):
		- lexSessionTitle: Session titles after number
		- lexSessionNumber: Session numbers (1., 1.1., etc.)
		- lexAnnotation: Annotation labels (@ note)
		- lexBold, lexItalic, lexCode, lexMath: Inline formatting
		- lexFootnote, lexCitation, lexReference: Reference types
		- lexListMarker: List item markers (-, 1., a.)
		- lexDefinitionSubject: Definition terms


	4.3 Token Modifiers

		Modifiers provide additional styling hints:
		- declaration: First occurrence (e.g., definition term)
		- definition: Target of reference
		- readonly: Immutable content (verbatim blocks)


	4.4 Theme Guidelines

		When creating themes for Lex:

		Borrow from Markdown:
		- Sessions → Markdown headings (bold, larger)
		- Lists → Markdown lists (marker styled)
		- Code → Markdown code (monospace, gray background)

		Lex-specific:
		- Session numbers: Muted color (less important than title)
		- Annotations: Warning/info colors based on label
		- References: Link color (blue, underlined)
		- Inline formatting: Bold=bold, italic=italic (obvious!)

		Avoid:
		- Over-styling (let content be primary focus)
		- High contrast (Lex should feel calm, readable)
		- Syntax-language colors (Lex is not code)


5. LSP Implementation Patterns

	This section covers common patterns when implementing LSP features.


	5.1 Feature Module Structure

		Each feature is a separate module (lex-lsp/src/features/*.rs):

		Standard structure:
		- Public function: feature_name(document: &Document, params: Params) -> Result
		- Walk AST to collect relevant information
		- Return LSP response type
		- Unit tests using sample files

		Example (hover.rs):

		pub fn hover(document: &Document, position: Position) -> Option<HoverResult> {
		    // Find element at position
		    // Look up target if it's a reference
		    // Return hover content
		}
		:: rust

		Why this pattern:
		- Stateless: Easy to test, no shared state
		- Pure function: Same input → same output
		- Thin: Server just routes request to feature function


	5.2 AST Walking

		Most features need to walk the AST looking for nodes.

		Common patterns:

		Visitor pattern:
		- Define visitor trait with visit_session, visit_paragraph, etc.
		- Walk AST calling appropriate visitor method
		- Collect results in visitor state

		Recursive descent:
		- Function for each node type (walk_session, walk_paragraph)
		- Recursively descend into children
		- Return early when target found

		Iterator:
		- Flatten AST into iterator of nodes
		- Filter/map/collect to find target


	5.3 Position Mapping

		LSP uses line:column positions, AST uses byte ranges.

		Conversion:
		- AST stores both: Range (bytes) and Location (line:column)
		- Use Location.contains(position) to find node at cursor
		- Return Range for LSP responses

		Key insight: Parser tracks both during AST building.


	5.4 Testing

		Feature testing strategy:

		Unit tests (features/*.rs):
		- Use sample files from specs/v1/
		- Test individual features in isolation
		- Assert LSP responses match expected

		Integration tests (tests/*.rs):
		- Test full LSP request/response cycle
		- Use lexplore to load sample documents
		- Verify server behavior end-to-end


6. Future Directions

	Potential enhancements to LSP and plugins:


	6.1 Code Actions

		LSP code actions provide quick fixes:
		- Convert paragraph to list (when lines start with -)
		- Convert flat list to nested (when indentation suggests nesting)
		- Add missing blank line before session
		- Fix indentation to match parent


	6.2 Code Lens

		Code lens shows inline information:
		- Reference count for footnotes (how many times [42] is used)
		- Section length for sessions (word count, reading time)


	6.3 Rename

		Rename refactoring:
		- Rename definition subject, update all references
		- Rename annotation label, update all usages


	6.4 Workspace Symbols

		Cross-file symbol search:
		- Find all definitions across project
		- Find all annotations by label
		- Jump to any session by title


Key Files:
	- lex-lsp/src/lib.rs: LSP server overview and feature roadmap
	- lex-lsp/src/server.rs: LSP server implementation
	- lex-lsp/src/features/: Individual feature implementations
	- editors/nvim/lua/lex/init.lua: Neovim plugin setup
	- editors/nvim/lua/themes/: Semantic token themes
	- docs/dev/nvim-fasttrack.lex: Quick overview for Neovim plugin
