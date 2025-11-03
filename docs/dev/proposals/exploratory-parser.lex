Exploratory Parser Design: A Multi-Pass Approach

    This document outlines a proposal for a multi-pass parser design. The goal is to explore a simpler, potentially more maintainable parsing model for `lex` that leverages its structural regularities while sidestepping the common pitfalls of naive, single-pass implementations.

1. The Core Idea

    The central challenge in parsing `lex` is that its grammar mixes two concerns: the recursive, nested structure defined by indentation, and the specific grammatical rules for elements like Sessions, Definitions, and Lists. A single-pass approach, especially one attempting to use regular expressions, will inevitably fail because regex cannot handle the arbitrary nesting (recursion) required by the indentation rules.

    The proposed solution is to separate these concerns into three distinct, sequential passes:

    - Pass 0: The Line Categorization Pass. This pass transforms the flat stream of basic tokens from the lexer into a stream of higher-level, semantic line tokens.

    - Pass 1: The Shaping Pass. This pass handles structure only. It transforms the flat, linear stream of *semantic line tokens* into a "token tree" based purely on `INDENT` and `DEDENT` tokens.

    - Pass 2: The Pattern-Matching Pass. This pass handles grammar only. It traverses the token tree from Pass 1 and applies a series of simple, non-recursive pattern matchers (like regexes) to the direct children of each node to identify and construct the final AST elements.

2. Pass 0: The Line Categorization Pass (Basic Tokens to Semantic Line Tokens)

    This pass takes the flat stream of basic tokens (e.g., `TEXT`, `COLON`, `NEWLINE`, `SEQ_MARKER`) from the lexer and groups them into more semantically meaningful "line tokens." This pre-processing step dramatically simplifies subsequent parsing stages.

    The algorithm would iterate through the basic token stream, identifying patterns that constitute a full line, and emitting a single, higher-level token for that line type.

    Examples of semantic line tokens:

    - `ANNOTATION_LINE`: `<lex-marker> <label>? <parameters>? <lex-marker> <text-span>?`
    - `SUBJECT_LINE`: `<text-span>+ <colon> <new-line>`
    - `LIST_LINE`: `<list-item-marker> <text-span>+ <new-line>`
    - `BLANK_LINE`: `<whitespace>* <new-line>`
    - `PARAGRAPH_LINE`: (Any line not matching the above, typically just `<text-span>+ <new-line>`)

    Structural tokens like `INDENT` and `DEDENT` would pass through this stage unchanged.

3. Pass 1: The Shaping Pass (Semantic Line Tokens to Tree)

    The goal of this pass is to convert the flat stream of *semantic line tokens* (from Pass 0) into a tree that explicitly represents the document's indentation structure. The algorithm is straightforward:

    - Start with a root `Document` node.
    - Read the stream of semantic line tokens sequentially.
    - On an `INDENT` token, create a new `IndentedBlock` node as a child of the current node, and then descend, making the new block the current node.
    - On a `DEDENT` token, ascend back to the parent of the current node.
    - All other semantic line tokens (`SUBJECT_LINE`, `LIST_LINE`, `PARAGRAPH_LINE`, etc.) are added as children to whichever node is currently active.

    Example:

        Input (Flat Semantic Token Stream):
            [SUBJECT_LINE:"Outer", NEWLINE, INDENT, PARAGRAPH_LINE:"Inner", NEWLINE, DEDENT]

        Output (Token Tree):
            Document
            ├── SUBJECT_LINE:"Outer"
            ├── NEWLINE
            └── IndentedBlock
                ├── PARAGRAPH_LINE:"Inner"
                └── NEWLINE

4. Pass 2: The Pattern-Matching Pass (Tree to AST)

    With the token tree already built from semantic line tokens, the recursion problem is solved. We can now apply simple, linear pattern matching at each level of the tree. This pass traverses the token tree and builds the final, semantic AST.

    The process, likely in a function like `parse_node(node)`, is as follows:

    - For a given `node` in the token tree, get its list of direct children.
    - Convert this list of children into a string of *semantic token names* (e.g., `"SUBJECT_LINE BLANK_LINE IndentedBlock"`).
    - Apply a series of regexes to this string in order of precedence.
    - If a regex for a `Session` matches, create a `Session` AST node. The content for this session is the `IndentedBlock` child from the token tree. To parse this content, make a recursive call: `parse_node(indented_block_child)`.
    - If no specific rule matches, apply the fallback rule: interpret the sequence of tokens as a `Paragraph`.

5. Example Walkthrough

    lex Source:
        Outer Session:

            Some text.

            Inner Session:

                More text.

    Pass 0 Output (Flat Semantic Token Stream):
        [SUBJECT_LINE:"Outer Session:", BLANK_LINE, INDENT, PARAGRAPH_LINE:"Some text.", BLANK_LINE, SUBJECT_LINE:"Inner Session:", BLANK_LINE, INDENT, PARAGRAPH_LINE:"More text.", DEDENT, DEDENT]

    Pass 1 Output (Token Tree):
        Document
        ├── SUBJECT_LINE:"Outer Session:"
        ├── BLANK_LINE
        └── IndentedBlock
            ├── PARAGRAPH_LINE:"Some text."
            ├── BLANK_LINE
            ├── SUBJECT_LINE:"Inner Session:"
            ├── BLANK_LINE
            └── IndentedBlock
                └── PARAGRAPH_LINE:"More text."

    Pass 2 Process:
        1. `parse_node(Document)` is called. It examines its children: `[SUBJECT_LINE, BLANK_LINE, IndentedBlock]`. This matches the `Session` rule.
        2. It creates the "Outer Session" AST node.
        3. It makes a recursive call, `parse_node(IndentedBlock)`, to parse the session's content.
        4. Inside this call, the node's children are `[PARAGRAPH_LINE, BLANK_LINE, SUBJECT_LINE, BLANK_LINE, IndentedBlock]`. The parser would first identify and consume the "Some text." paragraph.
        5. Next, it would see the remaining children: `[SUBJECT_LINE, BLANK_LINE, IndentedBlock]`. This again matches the `Session` rule.
        6. It creates the "Inner Session" AST node and recursively calls `parse_node` on its `IndentedBlock`, which would finally parse the "More text." paragraph.

6. Benefits of this Approach

    - Enhanced Separation of Concerns: Each pass has an even more specialized and focused job, leading to extremely clear code.
    - Extreme Simplicity in Pass 2: The pattern-matching logic becomes almost trivial, as regexes operate on a highly abstract and semantic token stream.
    - Improved Maintainability: Changes to micro-syntax (e.g., how a subject line is formed) are isolated to Pass 0, without affecting Pass 1 or Pass 2.
    - Robustness: By breaking down the problem, each pass is simpler to test and verify independently.