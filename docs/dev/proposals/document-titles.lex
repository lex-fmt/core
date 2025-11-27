Document Titles

1. Introduction

    Currently, the Lex format lacks a formal definition of a document's title. While sessions have titles, the document itself—the root container—does not. This leads to ambiguity and inconsistency, as the root session's title is technically empty, and there is no standard way to determine what the "human-readable" title of a document is.

    This proposal aims to formalize the concept of a Document Title, defining how it is parsed, stored, and accessed.

2. Problem Statement

    In the current implementation, a document is a tree of sessions. The root node is a `Session` object, which has a `title` field. However, this root session is created implicitly to hold the document content, and its title is initialized to an empty string.

    Consider a simple document:

    ----
    1. Introduction

      Welcome.
    -----

    The AST structure is effectively:
    `Document -> RootSession(title="") -> [Session(title="Introduction"), ...]`

    There is no place to store a title for the document itself. Users might expect the first line "1. Introduction" to be the document title, but structurally it is just the title of the first child session.

    This lack of a formal document title makes it difficult for tooling (like table of contents generators, indexers, or viewers) to display a meaningful title for the file.

3. Proposed Design

    The design introduces a formal Document Title concept with specific parsing rules to disambiguate it from regular content.

    3.1. Storage

        We will leverage the existing `Session` structure of the document root. The document title will be stored in the `title` field of the root `Session` node.

        - **No new AST fields**: The `Document` struct remains a wrapper around the root `Session`.
        - **API Access**: Helper methods `document.title()` and `document.set_title()` will be added to the `Document` struct, which delegate to `self.root.title`.

    3.2. Parsing Rules

        The core challenge is distinguishing a Document Title from a standard first paragraph or a child session title. To resolve this, we define strict rules for what constitutes a Document Title.

        A Document Title is identified if the **first element** of the document meets the following criteria:

        1.  It is a **single line** of text.
        2.  It is followed by at least one **blank line**.
        3.  It is **not indented** (indented text would imply it belongs to a parent container, but at the document root, indentation is not allowed for the first element anyway).

        If these conditions are met, that line is promoted to be the **Document Title** (i.e., set as the root session's title).

        Example A: Explicit Document Title
            My Document Title

            This is the content.
        :: lex ::

        Here, "My Document Title" is a single line followed by a blank line. It becomes the Document Title. The content "This is the content." becomes a child of the root session.

        Example B: Not a Title (Multi-line)
            This is a paragraph
            that spans multiple lines.

            Content.
        :: lex ::

        Here, the first element is a multi-line paragraph. It is **not** a title. The document title remains empty.

        Example C: Not a Title (No blank line)
            Title?
            Content immediately following.
        :: lex ::

        Here, there is no blank line separator. This is parsed as a single paragraph with two lines. It is **not** a title.

    3.3. Fallback: Session Hoisting

        If the document does **not** have an explicit Document Title (as defined above), but starts with a `Session`, we adopt a "Session Hoisting" strategy.

        If the first element is a `Session`, its title is effectively the title of the content. In this case, we can consider the document's title to be the same as this first session's title.

        Example D: Session Hoisting
            1. Introduction

                Content.
        :: lex ::

        Here, the parsing logic sees a Session "1. Introduction". The Document Title is implicitly "Introduction" (or "1. Introduction").
        *Note: The implementation may choose to copy this string to the root title or simply have the `document.title()` accessor fall back to the first child session's title if the root title is empty.*

4. Implementation Strategy

    4.1. Spec Update
        Create `specs/v1/elements/document.lex` to formally define these rules.

    4.2. AST Changes
        - No structural changes to `Document` or `Session`.
        - Add `title()` and `set_title()` methods to `Document` in `lex-parser/src/lex/ast/elements/document.rs`.

    4.3. Parser Changes
        - Modify `lex-parser/src/lex/parsing/engine.rs` (or the appropriate parsing stage).
        - Implement a check at the start of parsing:
            - If the first parsed node is a `Paragraph` with 1 line and is followed by a `BlankLineGroup`, extract the text and set it as the root session's title. Remove that paragraph node from the children.
            - Ensure this only happens at the very beginning of the document.

5. Comprehensive Examples

    Example 1: Explicit Title
        The Lex Manual

        Lex is a format...
    :: lex ::
    Result: `doc.title` = "The Lex Manual". First child = Paragraph("Lex is a format...").

    Example 2: Session Start (Hoisting)
        1. Introduction

            Content...
    :: lex ::
    Result: `doc.title` = "Introduction". First child = Session("1. Introduction").

    Example 3: Paragraph Start (No Title)
        This is just a note.
        It has no title.
    :: lex ::
    Result: `doc.title` = "" (Empty). First child = Paragraph("This is just a note...").

6. Revised Implementation: Grammar-Driven Parsing

    The initial implementation (Section 4) used imperative code in the AST builder to detect and extract document titles. While functional, this approach has drawbacks:

    - Special-case logic buried in `ast_tree.rs:build()`
    - Title detection bypasses the grammar system entirely
    - Difficult to test in isolation
    - Fragile to changes in parsing order

    This section proposes a grammar-driven approach that integrates title parsing into the existing declarative grammar system.

    6.1. Core Insight: Document Content Boundary

        The key insight is that "document title" is really about marking where document content begins. A document has two regions:

        1. Document metadata (annotations at the top)
        2. Document content (everything after)

        The first line of document content is special - it might be a title. By explicitly marking this boundary, we enable grammar rules to reason about document structure.

    6.2. DocumentStart Synthetic Token

        We introduce a new synthetic line type: `LineType::DocumentStart`.

        This token marks the boundary between document-level annotations and document content. It is injected by a transformation after line token grouping.

        Placement rules:
        - If no document-level annotations exist: position 0
        - If document-level annotations exist: immediately after the last annotation

        The token is synthetic (like `Indent`/`Dedent`) - it has no source text but carries structural meaning.

    6.3. Grammar Rule for Document Title

        With `DocumentStart` in place, the grammar can express document title as a declarative pattern:

            ("document_title", r"^<document-start-line>(?P<title><paragraph-line>)(?P<blank><blank-line>+)")

        This reads: "A document title is a paragraph line immediately after DocumentStart, followed by blank lines."

        The grammar matcher will:
        1. See `<document-start-line>` and know we're at content start
        2. Match a single paragraph line as the title
        3. Require trailing blank line(s) as separator
        4. Produce `NodeType::DocumentTitle` in the IR

    6.4. AST Builder Integration

        The AST builder handles `NodeType::DocumentTitle` like any other node type:

            NodeType::DocumentTitle => {
                // Extract title text, set on root session
                // No special-case detection logic needed
            }

        This removes the imperative title detection from `build()` entirely.

    6.5. Benefits

        - Declarative: Title pattern is data, not code
        - Testable: Grammar rules can be unit tested
        - Consistent: Uses same machinery as all other elements
        - Extensible: Easy to add document subtitle, author, etc.
        - Clear: DocumentStart explicitly marks the metadata/content boundary

7. Phased Delivery

    The implementation is split into two phases to reduce risk and enable incremental testing.

    7.1. Phase 1: DocumentStart Token

        Deliverables:
        - Add `LineType::DocumentStart` to `line.rs`
        - Create `DocumentStartMarker` transformation
        - Integrate into lexing pipeline (after `LineTokenGroupingMapper`)
        - Update grammar to recognize `<document-start-line>`
        - Ensure document-level annotations work correctly with the new boundary
        - Add tests for DocumentStart placement

        This phase delivers value by:
        - Establishing the metadata/content boundary formally
        - Enabling future grammar rules that depend on document position
        - Validating the transformation pipeline integration

        No changes to title parsing in this phase - existing imperative code continues to work.

    7.2. Phase 2: Document Title Grammar Rule

        Deliverables:
        - Add `document_title` grammar pattern
        - Add `NodeType::DocumentTitle` to IR
        - Add builder for DocumentTitle nodes
        - Remove imperative title detection from `ast_tree.rs`
        - Update tests to use grammar-based title parsing

        This phase completes the migration to grammar-driven title parsing.

8. File Reference

    Phase 1 changes:
    - `lex-parser/src/lex/token/line.rs` - Add `LineType::DocumentStart`
    - `lex-parser/src/lex/lexing/transformations/` - New `document_start.rs`
    - `lex-parser/src/lex/lexing/transformations/mod.rs` - Export new transformation
    - `lex-parser/src/lex/parsing/parser/grammar.rs` - Add `<document-start-line>` symbol

    Phase 2 changes:
    - `lex-parser/src/lex/parsing/parser/grammar.rs` - Add `document_title` pattern
    - `lex-parser/src/lex/parsing/ir.rs` - Add `NodeType::DocumentTitle`
    - `lex-parser/src/lex/parsing/parser/builder.rs` - Add DocumentTitle builder
    - `lex-parser/src/lex/building/ast_tree.rs` - Remove imperative title detection