Parsing Lex Strings

	This document contains a high level overview of the parsing architecture of Lex.

	Parsers will receive a TokenStream[1] with the core tokens[2], and use adapters if it needs the token structured to be different as in the linebased parser does[3].  And after parsing return the AST tree.

	The final parsing step, building the AST has token stream support (and the adapters for it)

	
AST Construction

	The AST (Abstract Syntax Tree) construction phase transforms parsed tokens into structured AST nodes with accurate location tracking. This phase is shared across all parsers through a unified three-layer architecture.

	In order to make experimenting with parsers easier and still outputing consistently correct trees, the ast building code centralizes and manages most of the complexity. This ensures correct location computation and text handling.

	1. Input
		The ast building can handle input in the common structures (flat list, lines , tree).
		It then converts them to Vec<Vec<(Token, Range<usize>)>> in ast/token/normalization.rs[4]
	2. Data Extraction
		Extract the token data into a pure structu / object format, ready to be injected in the ast node.
		This includes de span / byte range calculation and wall stripping for foreing lines at ast/extraction.rs
	3. AST Instantiation	
		Now , with the pure and transformed data to create the ast nodes, we finally create them.
5. Parser Integration

	Both parsers are fully integrated with the common AST construction code:

	Reference Parser:
		Uses text-based and token-based APIs via builders.rs
		Combinator patterns extract text during parsing
		Delegates to ast_builder for final AST construction
		:: file src/lex/parsers/reference/builders.rs ::

	Line-Based Parser:
		Uses unwrapper pattern in builders.rs
		Declarative grammar extracts patterns
		Unwrappers delegate to ast_builder
		:: file src/lex/parsers/linebased/builders.rs ::
		:: file src/lex/parsers/linebased/declarative_grammar.rs ::

	Flow for line-based parser:
		Pattern Match → unwrap_* → ast::build_* → AST Node

	Flow for reference parser:
		Combinator → Extract Text → ast::build_*_from_text → AST Node



Notes: 

 2. The token stream guide:  docs/dev/guides/on-tokenstreams.lex
 1. The core tokens: src/lex/lexers/tokens_core.rs and it's grammar docs/specs/v1/grammar.lex
 3. Adapters: src/lex/pipeline/adapters_linebased.rs and src/lex/pipeline/adapters.rs
 4. Normalization File: src/lex/parsers/ast/token/normalization.rs ::
 5. Data Extraction File: src/lex/parsers/ast/extraction.rs