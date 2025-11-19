# Fullwidth Verbatim Blocks

## 1. Introduction

Verbatim blocks embed non-lex content within a document. Typically, the block's content must be indented one level deeper than its subject line.

Inflow (Standard) Verbatim block:
```lex
    Subject:
        |<- content wall starts here (+1 indent)
        def hello():
            print("hello")
:: lex ::
```

## 2. Rationale

For wide content like tables, especially when nested deep in a document, the columns lost to indentation can harm readability. To address this, a "fullwidth" mode is proposed, allowing content to start at a fixed, absolute column close to the left margin, independent of the block's own indentation level.

## 3. Conceptual Model & Syntax

-   Inflow Mode (Default): The content's indentation "wall" is at `+1` level from the subject line.
-   Fullwidth Mode: The content's indentation "wall" is at a fixed absolute column: `FULLWIDTH_INDENT` (column 2, index 1).

True to Lex's ethos, there is no explicit syntax. The mode is inferred by the position of the first non-whitespace character of the *first content line*.

-   If it starts at `FULLWIDTH_INDENT`, the block is Fullwidth.
-   Otherwise, the block is Inflow.

Once the mode is set for a block, the wall is fixed. All subsequent content lines must start at or after the wall.

## 4. Revised Implementation Plan

This revised plan is simpler and more robust than the original. It moves the core logic from the lexer to the parser and builder, which are the layers responsible for understanding block structure and context.

### Phase 1: Keep the Lexer Simple and Stateless

-   No Lexer Changes: The lexing pipeline (`line_classification.rs`, `semantic_indentation.rs`, etc.) will not be modified.
-   It will have no special knowledge of verbatim blocks. Content lines within a verbatim block will be treated as standard `ParagraphLine`s or `BlankLine`s.
-   The `semantic_indentation` mapper will correctly emit `Dedent` tokens for fullwidth lines, but this is expected and will be handled by the parser.

### Phase 2: Enhance the Parser (Core Logic)

The `linebased` parser is context-aware and is the correct place to identify the full boundary of a verbatim block.

-   Modify `declarative_grammar.rs`:
    -   The pattern for a verbatim block will be updated. Instead of looking for a `SubjectLine` followed by an indented `<container>`, it will be modified to find a `SubjectLine` followed by a sequence of *any* line tokens, ending when it finds a closing `AnnotationLine` at the same indentation level as the subject.
    -   This change makes the parser responsible for finding the block's boundaries, without relying on the `tree_builder` to have pre-nested the content in a container (which it can't do for fullwidth content).
    -   The parser will then pass the raw, flat list of content `LineToken`s to the building stage.

### Phase 3: Centralize Logic in the Builder

The `building` stage is the ideal place for the feature-specific logic of mode detection and wall calculation.

-   Update the AST (`verbatim.rs`):
    -   Introduce a public enum: `VerbatimBlockMode { Inflow, Fullwidth }`.
    -   Add a `mode: VerbatimBlockMode` field to the `VerbatimBlock` struct.

-   Update the Extraction Layer (`extraction/verbatim.rs`):
    -   The `extract_verbatim_block_data` function will be the primary site for the new logic. It will now receive the flat list of content `LineToken`s from the parser.
    -   Mode Detection: It will inspect the *first content line token*. It will calculate the column of its first non-whitespace character.
        -   If `column == FULLWIDTH_INDENT` (index 1), the mode is set to `Fullwidth`.
        -   Otherwise, the mode is set to `Inflow`.
    -   Wall Calculation: Based on the detected mode, it will calculate the `indentation_wall`.
        -   If `mode` is `Fullwidth`, `indentation_wall = FULLWIDTH_INDENT`.
        -   If `mode` is `Inflow`, `indentation_wall = subject_line.indent_level + 1`.
    -   The rest of the function's logic (stripping the wall from each line) will operate on the correctly calculated `indentation_wall`.

-   Update the Builder (`ast_nodes.rs`):
    -   The `verbatim_block_node` function will be updated to accept the `VerbatimBlockMode` from the extraction layer and populate the new `mode` field in the final AST node.

### Phase 4: Testing

-   Add new `.lex` sample files to `specs/v1/elements/verbatim.docs/` to test the fullwidth feature at various nesting levels.
-   Add unit tests for the mode detection and wall calculation logic in the extraction layer to test it in isolation.
-   Ensure existing verbatim block tests continue to pass.

## 5. NOTES

1.  Full Width Indent: `FULLWIDTH_INDENT` is column 2 (0-based index 1). This provides a clear visual offset from the margin and avoids ambiguity with closing annotation markers (`::`) which could appear at column 1.
