Fullwidth Verbatim Blocks


1. Introduction
	Verbatim blocks embed non-lex content within a document, following the general indentation structure of lex, which means the block's content is +1 indented from the block's subject line.

	This is a Verbatim block:

        |<- content wall starts here
        |<- characters to the left are illegal (they are the indentation wall , not part of the content)
		def hello():
			print("hello")
    :: text ::


2. Rationale

    For some wider contents, such as tables, when the block is already deep in the document, the columns lost to the indentation wall can hinder the readability of the content. For this reason, verbatim blocks have a fullwidth mode, in which the content starts at the FULLWIDTH_INDENT, which is by default absolute 2 [1]. 

3. Conceptual Model

	From both a parsing and a code perspective we define a fullwidth block as one having the indentation wall at column absolute one, whereas the regular block (henceforth called inflow) has the wall at +1 indentation level from the block itself.

	That change aside, the internal code should behave the same. For example: 
  | Mode   | Block Start Index | Content Start Index |
  | ------ | ------------------| ------------------- |
  | Inflow 		| 4            | 8              	 |
  | Fullwidth	| 1            | 2              	 |
	:: table ::

   The example above shows a full width block with the content starting at column 2, and the block starting at column 5. Note that both the blocks start line (the subject line) and the end line (the closing annotation) are at the same indentation level as the content they are attached to.

4. Syntax 

	True to Lex's ethos, there is no explicit syntax for fullwidth blocks. The block's type is inferred from the position of the first content line character's position. Note that , for example the first line being FULLWIDTH_INDENT will make that a fullwidth block. The logic for verbatim content is that contend is indented at the wall level or greater. Full width blocks, being the same rule, behaver the same way, and as such any content starting at FULLWIDTH_INDENT +1 is valid, with the empty spaces between the wall and the first char being part of the content itself, as before.

	The opposite is not true. If the first content line is indentation +1 , no other content lines can be to the left of the wall, regardless of it being at FULLWIDTH_INDENT or not.
   
 
5. Implementation Plan

	This implementation avoids changes to the low-level tokenizer in favor of centralizing the detection logic within the more context-aware mappers of the lexing pipeline. This approach keeps the initial tokenization simple and robust.


	5.0 Prep work: 

		Before starting the code, be sure to add Lex sample files for this feature to : docs/specs/v1/elements/verbatim, using the guidelines as in docs/dev/guides/on-lexplore.lex, including: 
			- Isolated Elements: files for tests.
            - Simple ensambles (can be with only sessions, paragraphs) so that you can test the bblocks at different levels of nesting.
			- More complicated ensables with more eelements, the block being the last in a level that is about to be dedented and so on

			- When all is working add one instance of the block in the kitchensink document.

	5.1. Phase 1: No Lexer/Token Changes

		- Keep `tokens_core.rs` unchanged. Do not introduce a special `<fullwidth>` token. The initial `logos`-based tokenizer will continue to see all leading whitespace as standard `Whitespace` tokens. This maintains the simplicity and context-free nature of the foundational lexing layer.

	5.2. Phase 2: Enhance Pipeline Mappers (Core Logic)

		The primary challenge is to prevent fullwidth content lines from triggering premature `Dedent` events in the `semantic_indentation` mapper. We will achieve this by identifying potential verbatim blocks and re-classifying their content lines *before* they reach the indentation mapper.

		5.2.1. Enhance `line_type_classification.rs`:
			- Modify this mapper to become stateful. When it encounters a `SubjectLine`, it will begin buffering subsequent lines.
			- It will continue to buffer lines until it finds a matching `AnnotationLine` at the same indentation level as the initial `SubjectLine`.
			- Once this complete `Subject -> ... -> Closing Annotation` pattern is confirmed, it will re-classify all the buffered lines:
				- The first line remains a `SubjectLine`.
				- The last line remains an `AnnotationLine`.
				- All lines in between are re-tagged with a new `LineType`: `VerbatimContentLine`.
			- The mapper will then flush this sequence of re-tagged lines to the next stage of the pipeline.

		5.2.2. Enhance `semantic_indentation.rs`:
			- Modify this mapper to recognize the new `VerbatimContentLine` `LineType`.
			- When it encounters a line of this type, it will completely bypass its standard indentation logic. It will not check the line's indentation level against the stack and will not emit any `Indent` or `Dedent` events. It will simply pass the `VerbatimContentLine` through untouched.
			- This step effectively "protects" the fullwidth content from breaking the block structure, solving the core problem.

	5.3. Phase 3: Update AST and Extraction Layer

		This layer is responsible for interpreting the stream of `VerbatimContentLine`s and determining the block's mode and indentation wall.

		5.3.1. Update the AST (`verbatim.rs`):
			- Introduce a public enum to represent the block's mode:
					pub enum VerbatimBlockMode {
						Inflow,
						Fullwidth,
					}
				:: rust ::
			- Add a `mode` field to the `VerbatimBlock` struct: `pub mode: VerbatimBlockMode`.

		5.3.2. Update the Extraction Layer (`extraction.rs`):
			- The `extract_verbatim_block_data` function will be the primary site for mode detection.
			- It will receive the list of `VerbatimContentLine`s from the parser.
			- Mode Detection Logic: It will inspect the column of the first non-whitespace character of the *first content line*.
				- If `first_content_line.start_column == FULLWIDTH_INDENT` (e.g., column 2, which is index 1), the mode is set to `Fullwidth`.
				- Otherwise, the mode is set to `Inflow`.
			- Wall Calculation: Based on the detected mode, it will calculate the `indentation_wall`:
				- If `mode` is `Fullwidth`, `indentation_wall = FULLWIDTH_INDENT`.
				- If `mode` is `Inflow`, `indentation_wall = subject_line.indent_level + 1` (using the existing logic).
			- The rest of the function's logic, which strips the wall and creates `VerbatimLine`s, will remain unchanged, as it will operate on the correctly calculated `indentation_wall`. This fulfills the goal of having the internal logic behave identically for both modes.

	5.4. Phase 4: Update the Builder

		- Modify `ast_nodes.rs`: The `verbatim_block_node` function will be updated to accept the `VerbatimBlockMode` from the extraction layer. It will then use this to populate the new `mode` field when constructing the final `VerbatimBlock` AST node.

	5.5. Phase 5: Testing

		- Add new `.lex` sample files to `docs/specs/v1/elements/verbatim/` to specifically test the fullwidth feature.
		- Include test cases for content positioned exactly at the fullwidth wall, content indented beyond the wall, and mixed-indentation content.
		- Add a test case to verify that a block starting with a `+1` indent line but containing subsequent lines at the `FULLWIDTH_INDENT` is correctly parsed as an `Inflow` block and that the out-of-place lines are handled correctly (likely resulting in a parsing error or being truncated, as per the spec).
		- Add unit tests for the mode detection and wall calculation logic within the extraction layer.

6. NOTES

	1. Full Width Indent  

		The FULLWIDTH_INDENT being the second column (col 1 for 0 zero based indexing), that is absolute one. It's easier, in this spec to use base 1 indexing, but do note that the actual implementation uses zero based indexing.

		The reason for this is that: 
			- Were it to start at col 1, there would be no way to differentiate a inner content's annotation mark like line from the actual verbatim block end line without escaping , which is antethical to the goal of lex to be a simple and forgiving language.
		- Logically then anything from column two onwards are good candadiates, and the sorter the better.
		- However, tests shows that col 2 is not a good candidate, because it's too close to the indentation wall, and people often mistake it for an error. Hence position 2 makes is more obvioius to be a deliberate choice.
