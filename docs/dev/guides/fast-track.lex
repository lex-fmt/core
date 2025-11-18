Lex Format Fasttrack


This is a high level overview of how the Lex parser is designed.

1. Processing Management

	The lex/transforms module provides a composable transform system for processing stages.

	Key components:
	- Transform<I, O>: Type-safe transformations that chain via .then()
	- Runnable trait: Interface for individual processing stages
	- Static lazy transforms: Pre-built pipelines (LEXING, TO_IR, STRING_TO_AST, etc.)
	- DocumentLoader: Universal API for file/string loading and transform execution

	This system enables:
	- Type-safe stage composition with compiler verification
	- Reusable transforms across CLI, tests, and library code
	- Easy access to specific stages or full pipelines
	- Custom transform chains for specialized needs


2. Processing Pipeline Stages

	Processing flows through five main stages via the transform system:


	1. Lexing (LEXING)
		String → Vec<(Token, Range<usize>)>
	- Core Tokenization: Uses logos crate for lexical analysis, produces flat token stream
	- Semantic Indentation: Adds Indent/Dedent tokens based on whitespace levels
	- Line Grouping: Groups tokens into lines and classifies line types
	- Result: TokenStream of Line tokens + indent/dedent tokens


	2. Parsing - Semantic Analysis (TO_IR)
		String → ParseNode (IR tree)
	- Groups line tokens into hierarchical LineContainer tree
	- Uses declarative regex patterns to match grammar sequences
	- Emits IR nodes (ParseNode) specifying which AST nodes to create and which tokens to use
	- Separates semantic analysis from AST building for flexibility


	3. AST Building (part of STRING_TO_AST)
		ParseNode → Document
	- Walks IR tree creating final AST nodes
	- Unrolls source tokens so AST nodes have access to token values
	- Translates byte ranges to AST Location objects (line:column positions)
	- Creates Document node with root Session


	4. Document Assembly (part of STRING_TO_AST)
		Document → Document
	- Attaches annotations from content to AST nodes as metadata
	- Calculates "human understanding" distance for ambiguous cases
	- Post-parsing transformations on the complete AST


	5. Inline Parsing (ParseInlines stage)
		Document → Document
	- Parses TextContent nodes for inline elements (bold, italic, references, etc.)
	- Uses declarative engine with formal start/end tokens
	- Much simpler than block parsing (no structural elements)


	Standard pipelines:
	- LEXING: Core tokenization + semantic indentation
	- TO_IR: Full pipeline through IR (lexing + parsing)
	- STRING_TO_AST: Complete pipeline (lexing → parsing → building → assembling)
	- Inline parsing is typically done as a separate stage after STRING_TO_AST


3. Parser Design


	Line-based declarative grammar engine:

	- Receives TokenStream from lexing stage (Vec<(Token, Range<usize>)>)
	- Groups tokens into lines and builds hierarchical LineContainer tree
	- Classifies lines (SubjectLine, ListLine, ParagraphLine, etc.) as LineType
	- Uses declarative regex patterns to match container sequences
	- Pattern matching order is crucial (verbatim first, then annotations, lists, sessions, etc.)
	- Emits IR nodes (ParseNode) for AST construction

	Key insight: By grouping tokens into a tree of LineContainers, parsing can be done
	level-by-level in isolation. Each level only needs to know it has a LineContainer,
	not what's inside it. This enables regex-based pattern matching.

	The grammar is simple enough for ordered regex expressions, with one exception:
	verbatim blocks require stateful imperative matching (they must be parsed first
	to prevent their non-lex content from breaking structure).

	Key files:
	- src/lex/lexing/ - Tokenization and line grouping
	- src/lex/parsing/engine.rs - Main parsing orchestrator
	- src/lex/parsing/parser/grammar.rs - Declarative grammar patterns
	- src/lex/parsing/ir.rs - IR node definitions (ParseNode)
	- src/lex/building/ - AST node construction from IR
	- src/lex/assembling/ - Annotation attachment and post-processing


4. Token System

	Multiple layers of tokens for simplicity at parsing stage:

	- Core Tokens: Character/word level from logos lexer (Text, Whitespace, Dash, etc.)
	- Structural Tokens: Indent, Dedent (semantic indentation events)
	- Line Tokens: Groups of core tokens classified by type (SubjectLine, ListLine, etc.)
	- LineContainer: Hierarchical tree of lines reflecting indentation structure
	- Synthetic Tokens: Context-capturing tokens (e.g., preceding blank line for sessions)

	All tokens preserve byte ranges for location tracking. Source tokens are stored
	in grouped tokens under `source_tokens` field for easy unrolling at AST building stage.


5. AST Structure

	Document is the root node, containing:
	- Root Session: The content tree (sessions can nest arbitrarily)
	- Annotations: Document-level metadata

	Element types:
	- Blocks: Sessions, Paragraphs, Lists, Definitions, Annotations, Verbatim
	- Containers: Type-safe containers enforcing nesting rules at compile time
	- Inlines: Text spans (bold, italic, references, citations)

	Key design: Indentation is the manifestation of a container node. Titles/subjects
	are siblings of their content, not parents. Content is indented under container nodes.
