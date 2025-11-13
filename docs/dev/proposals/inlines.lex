:: title :: Proposal: Inline Elements

1. Introduction

	Lex is primarily a line-based language composed of block elements that define a document's structure. This proposal introduces Inline Elements: span-based markers that provide rich text formatting and semantic meaning within these blocks.

	Unlike block elements, inlines can start and end at arbitrary positions within a line of text. Their self-contained nature makes them highly suitable for parallelized parsing and simplifies testing.

2. Problem Statement

	Currently, Lex lacks a mechanism for formatting text within a block element, such as making a single word bold in a paragraph or marking a term as `code`. This is a fundamental feature for any rich text format.

	This proposal defines a robust, extensible, and coherent system for inline elements that aligns with Lex's core principles of readability and graceful degradation.

3. Proposed Design

	The proposed system is built on a unified foundation that can be extended to support various types of inline content, from simple formatting to complex references.

	3.1. General Token Form

		All inline elements follow a consistent pattern: `<token>content<token>`.

		- The `token` is one or more non-alphanumeric characters that mark the boundaries.
		- The `content` is the text to be affected.
		- There must be no whitespace between the tokens and the content.

		Example:
			*strong text*
			`code text`
			[a reference]
		:: lex ::

	3.2. Delimiter Recognition Rule

		To distinguish inline delimiters from literal punctuation (e.g., `7 * 8`), a precise recognition rule is required. A token is only treated as a delimiter if it is adjacent to a "word character" (alphanumeric) on the inside and "non-word" context on the outside.

		- A start token is valid only if it is not immediately preceded by a word character and is immediately followed by a word character.
		- An end token is valid only if it is immediately preceded by a word character and is not immediately followed by a word character.

		Example:
			*word*            :: Valid ::
			a *word* in text  :: Valid ::
			7 * 8             :: Invalid - tokens treated as literal asterisks ::
			word*s*           :: Invalid - start token is preceded by a word char ::
		:: lex ::

	3.3. Element Categories & Nesting

		Inline elements are grouped into categories that also define their nesting behavior.

		1.  Formatting: For visual and semantic emphasis (e.g., `*strong*`, `_emphasis_`). These elements can contain other inline elements, enabling multi-level formatting.
		2.  Literal: For content that should not be parsed further (e.g., `` `code` ``, `#math#`). These elements cannot contain other inlines.
		3.  References: For links, citations, and footnotes (e.g., `[target]`, `[@key]`). Their content has a specialized, non-recursive grammar.

		The ability for formatting elements to contain others is the foundation of multi-level inlines. The parser will recursively process the content of an inline, allowing for rich combinations.

		Example of valid nesting:
			*strong and _emphasized_ text*
		:: lex ::

	3.4. Universal Rules

		All inline elements adhere to the following rules:
		- No Empty Content: `` is invalid.
		- No Crossing Lines: An inline element cannot start on one line and end on another.
		- No Crossing Inlines: `*a _b* c_` is invalid.
		- No Same-Type Nesting: `*outer *inner* text*` is invalid.

	3.5. Parsing Priority

		To resolve ambiguity, inline elements are parsed in a specific order of precedence:
		1.  `Literal` elements (`Code`, `Math`)
		2.  `References`
		3.  `Formatting` elements (`Strong`, `Emphasis`, etc.)
		4.  `Plain Text` (the fallback)

	3.6. Graceful Degradation

		In keeping with Lex's philosophy, malformed inline syntax does not produce an error. If a start token is found but a valid end token is not, the start token is treated as a literal character.

4. Implementation Strategy

	The implementation will cleanly separate inline parsing from the existing block-level parsing.

	4.1. Decoupled Parsing

		The process will occur in two main phases:
		1.  Block Parsing: The `linebased` parser runs first, identifying the document's structure (`Paragraph`, `List`, etc.) and the raw text content within them.
		2.  Inline Parsing: A dedicated inline parser then recursively processes the raw text content of each block, transforming it into a rich, structured representation.

	4.2. AST Homogeneity: The `TextContent` Node

		To ensure a clean and uniform AST, the text content of elements like `Paragraph` will not be a simple `String`. Instead, it will be a `TextContent` node.

		This `TextContent` node acts as a container for a sequence of `InlineItem`s (e.g., `Text`, `Strong`, `Code`). For multi-level inlines, the content of a formatting element like `Strong` is itself another `TextContent` node, enabling the recursive structure.

		Example AST Structure for `*a _b_*`:
			├── Strong
			│   └── TextContent
			│       ├── Text("a ")
			│       ├── Emphasis
			│       │   └── TextContent
			│       │       └── Text("b")
		:: tree ::

	4.3. Parameterized Parser Declaration

		Many simple formatting elements are structurally identical, differing only by their delimiter token. To avoid repetitive code, the implementation will be driven by a declarative, parameterized list of inline specifications.

		This can be represented as a collection of structs or an enum that defines each inline type's properties.

		Example Declaration:
			struct InlineSpec {
			    name: &'static str,
			    start_token: &'static str,
			    end_token: &'static str,
			    node_type: NodeType,
			    // ... other properties like nesting allowance
			}

			const INLINE_SPECS: &[InlineSpec] = &[
			    InlineSpec { name: "Strong", start_token: "*", end_token: "*", ... },
			    InlineSpec { name: "Emphasis", start_token: "_", end_token: "_", ... },
			];
		:: rust ::

		The main inline parsing loop will iterate through this list, checking for matching tokens. This makes the parser data-driven, highly extensible, and easy to maintain. Adding a new simple formatting element becomes a one-line change to this list.

5. Initial Scope

	The initial implementation will focus on the foundational formatting elements:
	- Strong (`*content*`)
	- Emphasis (`_content_`)
	- Code (`` `content` ``)