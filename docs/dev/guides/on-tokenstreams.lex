Token Streams: Simplified Flat Transformation Architecture

	After refactoring, Token Streams are now dramatically simplified. TokenStream is just a type alias for Vec<(Token, Range<usize>)> - a flat vector of tokens with their source locations. The pipeline performs simple, composable transformations on flat streams.

1. The Evolution: From Complex to Simple

	Originally, Token Streams were a complex enum supporting both flat sequences and hierarchical trees. This added significant complexity to enable tree-building in the pipeline.

	The key insight: Only one consumer (the line-based parser) needed hierarchical structure. Making the entire pipeline handle trees to serve one consumer violated the principle of locality of complexity.

	The solution: Move tree-building into the line-based parser itself, keep the pipeline purely flat.

2. The Current TokenStream Architecture

	TokenStream is now trivially simple:
	:: file src/lex/pipeline/stream.rs ::

		pub type TokenStream = Vec<(Token, Range<usize>)>;

	That's it. A token stream is just a flat vector of (Token, Range) pairs.

	Pipeline transformations:
		Receive: Vec<(Token, Range<usize>)>
		Return: Vec<(Token, Range<usize>)>
		Simple, composable, easy to understand

	No enums, no variants, no tree-walking infrastructure needed.

3. The Immutable Log Principle

	The foundation remains unchanged: original token locations are never modified, only preserved and aggregated.

	Core principle:
		Transformations can create aggregate tokens (Indent, Dedent, BlankLine)
		These aggregates store their source tokens internally
		Location information is never lost
		AST builders extract source tokens when needed

	Example: An Indent token contains Vec<(Token, Range<usize>)> of the original Indentation tokens it represents. The Indent itself uses a placeholder range (0..0), but the real locations are preserved in source_tokens.

4. The StreamMapper Pattern (Simplified)

	With TokenStream now just a flat vector, the mapper pattern is dramatically simpler:
	:: file src/lex/pipeline/mapper.rs ::

	The StreamMapper Trait:
		map_flat(): Transform a flat token sequence → flat token sequence
		That's the entire interface. No tree hooks needed.

	The Walker Function:
		Simply calls mapper.map_flat(stream)
		No recursive traversal logic
		No tree navigation complexity

	This simplification removed over 400 lines of tree-walking code while maintaining the same transformation interface.

5. Example Transformations

	The current lexer pipeline uses three core transformations (all operate on flat streams):

	NormalizeWhitespace:
		Processes whitespace remainder tokens
		Simple token replacement logic
		Input: flat stream, Output: flat stream

	SemanticIndentation:
		Converts Indentation tokens to Indent/Dedent pairs
		Tracks indentation stack state
		Creates synthetic tokens with source_tokens embedded
		Input: flat stream, Output: flat stream

	BlankLines:
		Groups consecutive Newline tokens into BlankLine tokens
		Stateful transformation with lookahead
		Preserves original tokens in aggregates
		Input: flat stream, Output: flat stream

	All transformations share the same interface: TokenStream → TokenStream. They can be chained, reordered, or replaced independently.

6. Pipeline Orchestration

	The Pipeline chains transformations together:
	:: file src/lex/pipeline/builder.rs ::
	:: file src/lex/pipeline/executor.rs ::

	Each transformation receives the output of the previous one
	All operate on flat Vec<(Token, Range)>
	Errors propagate cleanly through the Result type
	The final TokenStream goes to parsers (with optional adapters)

	Example pipeline (both parsers use the same pipeline):
		Base tokenization → Flat stream
		NormalizeWhitespace → Flat stream
		SemanticIndentation → Flat stream (with Indent/Dedent)
		BlankLines → Flat stream
		→ Parser (reference or linebased)

7. Parser Integration

	Reference Parser:
		Consumes flat TokenStream directly
		No adapter needed (just passes the Vec through)

	LineBased Parser:
		Builds tree structure internally in tree_builder module
		:: file src/lex/parsing/linebased/tree_builder.rs ::
		Groups tokens into lines, classifies them, builds hierarchy
		Tree complexity localized to the only consumer that needs it

	The transformation pipeline itself operates purely on flat TokenStream throughout. Tree building is a parser-internal concern, not a pipeline concern.

8. Benefits of Simplified Token Streams

	Dramatic Simplification:
		TokenStream: 432 lines → 17 lines
		Mapper infrastructure: 638 lines → 235 lines
		Adapters: 629 lines → 142 lines
		Total reduction: ~1,100 lines from pipeline code

	Composability:
		All transformations share identical interface
		Easy to chain, reorder, or disable
		New transformations integrate seamlessly

	Guaranteed Location Accuracy:
		Source tokens preserved in aggregates
		Perfect location tracking for AST building
		Immutable Log principle enforced architecturally

	Clarity:
		Transformation logic is just: flat → flat
		No complex traversal patterns to understand
		Reduces cognitive load dramatically

	Locality of Complexity:
		Tree building only in line-based parser
		Pipeline stays simple and focused
		Each component has clear, single responsibility

9. Implementation Notes

	The simplification was implemented in two phases:

	Phase 1: Move tree-building into parser
		Created tree_builder module in linebased parser
		Extracted logic from ToLineTokensMapper and IndentationToTreeMapper
		Parser builds trees internally from flat tokens

	Phase 2: Simplify TokenStream to type alias
		Removed enum variants (Flat/Tree)
		Removed unroll() method (no longer needed)
		Removed tree-walking infrastructure
		Updated all code to work with plain Vec

	The result is a clean, minimal system where:
		Pipeline: flat transformations only
		Parsers: consume flat streams (build trees if needed internally)
		Original token locations: never lost (Immutable Log)

	For historical context on the original design:
	:: file docs/dev/proposals/token-streams.lex ::

	For details on how the lexer pipeline uses Token Streams:
	:: file docs/dev/guides/on-lexing.lex ::
