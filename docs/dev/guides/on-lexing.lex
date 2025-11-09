Lexing

	Tokenization is handled by logos, the lexer generator library. The base_tokenization.rs module uses it to tokenize Lex source text into the core tokens defined in tokens_core.rs.

	Base Tokenization:
		Converting source text string into a sequence of tokens
		:: file src/lex/lexing/base_tokenization.rs ::

1. Pipeline & Transformations

	The pipeline is a declarative sequence of transformations applied to a flat TokenStream of core tokens. All pipelines start with core tokenization, producing a Vec<(Token, Range<usize>)>, then apply transformations in order, feeding one's output as input to the next.

	Transformations are designed to be chained. Here is the full flow:

	Lexing Pipeline:
		Source Text → Base Tokenization → Vec<(Token, Range)> → [Transformations] → Vec<(Token, Range)>

	Parsing Flow:
		Vec<(Token, Range)> → Parser → AST

	Transformations receive a flat token vector and return one as output. This is the contract that enables decoupled but composable transformations.

2. The Unified Lexer Pipeline

	After simplification, there is now one core pipeline used by both parsers:
	:: file src/lex/lexing.rs ::

	Three transformations (all operate on flat streams):
		NormalizeWhitespace → SemanticIndentation → BlankLines

	Output: Flat Vec<(Token, Range<usize>)>

	Both the reference parser and linebased parser consume this same flat output:
		Reference parser: Uses it directly
		LineBased parser: Builds tree structure internally via tree_builder module

3. The Three Core Transformations

	NormalizeWhitespace:
		Handles whitespace remainder tokens from logos
		Converts WhitespaceRemainder to Whitespace
		Simple token replacement
		:: file src/lex/lexing/transformations/normalize_whitespace.rs ::

	SemanticIndentation:
		Converts raw Indentation tokens into semantic Indent/Dedent pairs
		Tracks indentation stack to detect level changes
		Creates synthetic tokens with source_tokens preserved
		:: file src/lex/lexing/transformations/semantic_indentation.rs ::

	BlankLines:
		Groups consecutive Newline tokens into BlankLine aggregates
		Preserves original tokens in source_tokens field
		Enables parsers to handle blank lines as single units
		:: file src/lex/lexing/transformations/blank_lines.rs ::

4. Token Streams

	TokenStream is an enum that can represent both flat and tree structures:
	:: file src/lex/pipeline/stream.rs ::

		pub enum TokenStream {
			Flat(Vec<(Token, Range<usize>)>),
			Tree(Vec<TokenStreamNode>),
		}

	Most pipeline transformations output Flat. Tree building happens inside the linebased parser when needed. All TokenStreams can be unrolled back to flat Vec<(Token, Range<usize>)> for AST building.

	See the full documentation for details:
	:: file docs/dev/guides/on-tokenstreams.lex ::

5. Parser Integration

	The flat token stream goes directly to parsers:

	Reference Parser:
		Receives: Vec<(Token, Range<usize>)>
		Uses directly for combinator parsing
		:: file src/lex/parsing/reference/api.rs ::

	LineBased Parser:
		Receives: Vec<(Token, Range<usize>)>
		Builds LineContainer tree internally via tree_builder
		:: file src/lex/parsing/linebased/tree_builder.rs ::
		Groups tokens into lines, classifies them, builds hierarchy
		Tree complexity localized to the parser

	The transformation pipeline is simple and focused: flat token transformations only.
