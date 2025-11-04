Token Streams: Unified Transformation Architecture

	Token Streams are the unified data structure that enables composable, robust transformations in the Lex lexer pipeline. This architecture solves the problem of irregular transformation interfaces and provides guaranteed location tracking through the "Immutable Log" principle.

1. The Problem: Why Token Streams?

	Before Token Streams, the lexer pipeline had transformations with inconsistent interfaces. Some operated on flat token vectors, others on line-grouped structures, and others on hierarchical trees. This irregularity created brittleness when chaining transformations and made the pipeline difficult to maintain or extend.

	Token Streams unify these representations into a single data structure that can express both flat sequences and hierarchical nesting, enabling any transformation to work with any other.

2. The TokenStream Architecture

	At the core is the `TokenStream` enum, which can represent tokens in two forms:
	:: file src=src/lex/pipeline/stream.rs ::

	TokenStream::Flat:
		A linear sequence of (Token, Range) pairs
		The initial output from base tokenization
		Used by transformations that don't require structural nesting

	TokenStream::Tree:
		A hierarchical representation using TokenStreamNode
		Each node contains tokens and optionally nested children
		Enables indentation-based structures and line grouping

	TokenStreamNode Structure:
		tokens: The flat list of tokens at this level
		children: Optional nested TokenStream for indented content
		line_type: Optional LineType classification (SubjectLine, ListLine, etc.)

	This dual representation allows transformations to express their output naturally - flat when appropriate, nested when needed - while maintaining a common interface.

3. The Immutable Log Principle

	The foundation of Token Streams is the Immutable Log principle: original token locations are never modified, only preserved and aggregated. This guarantees accurate location tracking throughout all transformations.

	The Universal Unroll Method:
		Every TokenStream, regardless of complexity, can be unrolled
		Returns the flat list of original (Token, Range) pairs
		Provides the "ground truth" for AST building
		Recursively extracts tokens from any nesting depth

	This architecture means:
		Transformations can restructure tokens freely
		Location information is never lost
		AST builders receive accurate source positions
		Complex nested structures decompose reliably to their origins

4. The StreamMapper Pattern

	To prevent each transformation from implementing its own tree traversal logic, Token Streams use the StreamMapper pattern (a Visitor design pattern). The pipeline provides a generic walker that handles traversal, while transformations focus purely on their logic.
	:: file src=src/lex/pipeline/mapper.rs ::

	The StreamMapper Trait:
		map_flat(): Transform a flat token sequence
		enter_node(): Pre-order hook (before visiting children)
		exit_node(): Post-order hook (after visiting children)

	The Walker Function:
		Handles all recursive traversal automatically
		Calls mapper methods at appropriate points
		Manages the complexity of tree navigation
		Written once, tested thoroughly, used by all transformations

	This separation means transformations are simple, focused functions that declare their logic without implementing traversal mechanics.

5. Example Transformations

	The current lexer pipeline uses several TokenStream-based transformations:

	NormalizeWhitespace:
		Processes whitespace remainder tokens
		Operates on flat streams
		Simple token replacement logic

	SemanticIndentation:
		Converts Indentation tokens to Indent/Dedent pairs
		Tracks indentation stack state
		Outputs synthetic tokens with source_tokens embedded

	BlankLines:
		Groups consecutive Newline tokens into BlankLine tokens
		Stateful transformation with lookahead
		Preserves original tokens in aggregates

	ToLineTokens:
		Groups tokens into lines by Newline delimiters
		Classifies each line (SubjectLine, ListLine, ParagraphLine)
		First transformation to produce Tree variant

	IndentationToTree:
		Restructures shallow tree into nested hierarchy
		Processes Indent/Dedent markers to build nesting
		Creates proper parent-child relationships

	All these transformations share the same interface: TokenStream ’ TokenStream. They can be chained, reordered, or replaced independently.

6. Pipeline Orchestration

	The Pipeline builder chains transformations together:
	:: file src=src/lex/pipeline/builder.rs ::

	Each transformation receives the output of the previous one
	The walker handles traversal for Tree variants automatically
	Errors propagate cleanly through the Result type
	The final TokenStream goes to adapters for parser consumption

	Example pipeline (linebased lexer):
		Base tokenization ’ Flat stream
		NormalizeWhitespace ’ Flat stream
		SemanticIndentation ’ Flat stream
		BlankLines ’ Flat stream
		ToLineTokens ’ Tree stream (shallow)
		IndentationToTree ’ Tree stream (nested)

7. Architectural Boundaries

	Token Streams are the internal transformation currency. At architectural boundaries, adapters convert to domain-specific formats:
	:: file src=src/lex/pipeline/adapters_linebased.rs ::

	Lexer ’ Parser Boundary:
		TokenStream ’ LineContainer (for linebased parser)
		Preserves LineType classifications
		Maintains all original token information

	These adapters are the only places where TokenStream converts to external formats. The transformation pipeline itself is adapter-free and operates purely on TokenStream.

8. Benefits of Token Streams

	Composability:
		All transformations share identical interface
		Easy to chain, reorder, or disable in pipelines
		New transformations integrate seamlessly

	Guaranteed Location Accuracy:
		The unroll() method ensures original tokens are accessible
		Perfect location tracking for AST building
		Immutable Log principle enforced architecturally

	Simplicity:
		Transformation logic focuses on "what" not "how"
		Complex traversal written once in the walker
		Reduces duplication and cognitive load

	Robustness:
		Centralized, tested traversal logic
		Reduced chance of bugs in individual transformations
		Type-safe transformation interfaces

	Flexibility:
		Pre-order and post-order hooks provide full control
		Supports simple token replacement to complex restructuring
		Enables both flat and hierarchical transformations

9. Implementation Notes

	The Token Stream architecture was implemented progressively, migrating transformations one at a time while maintaining a working pipeline throughout. The final result is a clean, unified system where:

		All transformations use StreamMapper trait
		Walker function handles all traversal
		Original token locations are never lost
		Adapters exist only at architectural boundaries

	For the original design proposal, see:
	:: file src=docs/dev/proposals/token-streams.lex ::

	For details on how the lexer pipeline uses Token Streams:
	:: file src=docs/dev/guides/on-lexing.lex ::
