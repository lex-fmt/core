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