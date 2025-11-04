Lexing

	Tokenization is handled by logos, the lexer generator library. The base_tokenization.rs module uses it to tokenize Lex source text into the core tokens defined in tokens_core.rs.

	Base Tokenization:
		Converting source text string into a sequence of tokens
		:: file src/lex/lexers/base_tokenization.rs ::

1. Parsing Leverage: Higher Level Tokens

	Lex is an unconventional language which, from a general perspective, makes it a parsing challenge: recursive, stateful (in parts) and indentation significant. While this is a fact, the format has been designed with specific constraints that, when leveraged, will simplify parsing enormously.

	The linebased parser specifically requires tokens grouped into higher order structures: first into lines, then into levels (lines nested in hierarchical indentation). Other parsers may work directly with the flat token stream.

2. Pipeline & Transformations

	In order to foster easy experimentation between shifting complexity between the parser and the lexer, Lex has an extendable and declarative pipeline manager that can chain transformations.

	The pipeline is a declarative sequence of transformations to be applied to a TokenStream of core tokens. All pipelines will start with the core tokenization, convert them to a TokenStream and then apply its transformations in order, feeding one's TokenStream output as the input to the next until all transformations have been applied and generated the final TokenStream.

	Transformations are designed to be chained into pipelines. Here is what the full lexing and parsing flow looks like:

	Lexing Pipeline:
		Source Text → Base Tokenization → TokenStream::Flat → [Transformations] → TokenStream

	Parsing Flow:
		TokenStream → Adapter → Parser Input Format → Parser → AST

	Transformations receive a TokenStream and return one as output. This is the contract that enables us to keep decoupled but composable transformations.

3. The Two Lexer Pipelines

	There are currently two lexer pipelines serving different parsers:

	Simple Pipeline (lex):
		Used by the reference parser
		:: file src/lex/lexers.rs ::
		Three transformations:
			NormalizeWhitespace → SemanticIndentation → BlankLines
		Output: Flat TokenStream (converted to Vec<Token> for parser)

	LineBased Pipeline (_lex):
		Used by the linebased parser
		:: file src/lex/lexers/linebased/pipeline.rs ::
		Five transformations:
			NormalizeWhitespace → SemanticIndentation → BlankLines → ToLineTokens → IndentationToTree
		Output: Nested TokenStream (converted to LineContainer for parser)

	Both pipelines share the first three transformations, then the linebased pipeline adds two more to group tokens into lines and build hierarchical structure.

4. Token Streams

	The common data structure that allows different transformations to take place is the Token Stream.
	:: file src/lex/pipeline/stream.rs ::

	From a single vector to a full tree of tokens, the format offers standard interfaces for various transformations. See the full documentation for details:
	:: file docs/dev/guides/on-tokenstreams.lex ::

5. Adapters at Architectural Boundaries

	Token Streams are the internal transformation currency. At the lexer→parser boundary, adapters convert TokenStream to parser-specific formats:

	For Reference Parser:
		TokenStream::Flat → Vec<(Token, Range<usize>)>
		:: file src/lex/pipeline/adapters.rs ::

	For LineBased Parser:
		TokenStream::Tree → LineContainer
		:: file src/lex/pipeline/adapters_linebased.rs ::

	The transformation pipeline itself is adapter-free and operates purely on TokenStream throughout.
