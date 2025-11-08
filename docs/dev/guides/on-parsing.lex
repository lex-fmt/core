Parsing Lex Strings

	This document contains a high level overview of the parsing architecture of Lex.

	Parsers receive a flat Vec<(Token, Range<usize>)> from the lexing pipeline. The linebased parser builds its own tree structure internally. Both parsers then return the AST.

AST Construction

	The AST (Abstract Syntax Tree) construction phase transforms parsed tokens into structured AST nodes with accurate location tracking. This phase is shared across all parsers through a unified three-layer architecture.

	In order to make experimenting with parsers easier while outputting consistently correct trees, the AST building code centralizes and manages most of the complexity. This ensures correct location computation and text handling.

	1. Input
		The AST building can handle input in various structures (flat list, lines, tree).
		It converts them to Vec<Vec<(Token, Range<usize>)>> in building/token/normalization.rs

	2. Data Extraction
		Extract the token data into pure structure/object format, ready to be injected into AST nodes.
		This includes span/byte range calculation and wall stripping for indented lines at building/extraction.rs

	3. AST Instantiation
		Now, with the pure and transformed data to create AST nodes, we finally create them.

Parser Integration

	Both parsers are fully integrated with the common AST construction code:

	Reference Parser:
		Receives flat Vec<(Token, Range<usize>)> directly
		Uses text-based and token-based APIs via builders.rs
		Combinator patterns extract text during parsing
		Delegates to ast_builder for final AST construction
		:: file src/lex/parsing/reference/api.rs ::

	Line-Based Parser:
		Receives flat Vec<(Token, Range<usize>)>
		Builds LineContainer tree internally via tree_builder module
		:: file src/lex/parsing/linebased/tree_builder.rs ::
		Uses unwrapper pattern in builders.rs
		Declarative grammar extracts patterns
		Unwrappers delegate to ast_builder
		:: file src/lex/parsing/linebased/engine.rs ::
		:: file src/lex/parsing/linebased/declarative_grammar.rs ::

	Flow for line-based parser:
		Flat tokens → tree_builder.build_line_container() → Pattern Match → unwrap_* → ast::build_* → AST Node

	Flow for reference parser:
		Flat tokens → Combinator → Extract Text → ast::build_*_from_text → AST Node

Notes:

 1. The core tokens: src/lex/lexing/tokens_core.rs and its grammar docs/specs/v1/grammar.lex
 2. The token stream guide: docs/dev/guides/on-tokenstreams.lex
 3. Tree builder (linebased parser only): src/lex/parsing/linebased/tree_builder.rs
 4. Normalization File: src/lex/building/token/normalization.rs
 5. Data Extraction File: src/lex/building/extraction.rs
