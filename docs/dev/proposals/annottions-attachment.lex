Annotations And Metadata in Lex

1. Introduction

	Annotations are a core Lex element providing a way to attach metadata to a document. As such, they are the only element that is not part of the document's content. Annotations are helpful not only for authors and collaborators but also as structured hooks for tooling to build on top of Lex.

	For annotations to be useful, especially for tooling, they should be attached to any AST node as metadata, rather than being treated as content nodes themselves. This proposal outlines a design to achieve this.

2. Problem Statement

	The current model treats annotations as content items, creating an architectural inconsistency. A robust system requires a clear, deterministic, and predictable mechanism for associating annotations with the elements they describe.

	The primary challenge is to define an attachment rule that is both intuitive for authors and capable of gracefully handling mistakes, in line with Lex's design philosophy of avoiding hard parsing errors.

3. Proposed Design

	Under this design, annotations will no longer be part of the content tree. Instead, they will be stored in an `annotations` field on the AST nodes they apply to. The following rules will govern their attachment.

	3.1. Primary Rule: Prefix Attachment

		An annotation attaches to the content element it immediately precedes, with no blank lines between them. This "prefix" model makes the author's intent explicit and removes ambiguity.

		Example: Attaching to a Paragraph
			:: note author="John Doe" ::
			This is the paragraph being annotated.
		:: lex ::

		Example: Attaching to a Session
			:: session review_status="draft" ::
			1. My Session Title

			    Content of the session.
		:: lex ::

	3.2. Graceful Degradation: Orphaned Annotations

		Lex is designed to be forgiving. If an annotation does not immediately precede a content element (e.g., it is followed by a blank line or is the last item in a container), it is not discarded. Instead, it becomes an "orphaned" annotation.

		Orphaned annotations are attached to the `annotations` field of their immediate parent container (e.g., a Session, List Item, or the root Document). This preserves the metadata and allows tooling to identify and potentially flag misplaced annotations without causing a parse failure.

		Example: Orphaned annotation in a Session
			1. A Session

			    A paragraph inside the session.

			    :: note status="misplaced" ::

			    Another paragraph.
		:: lex ::

		In this case, the `:: note status="misplaced" ::` annotation is separated by a blank line from the following paragraph. It will be attached as an "orphaned" annotation to the `Session` node itself.

	3.3. Special Cases

		Two special cases are handled explicitly:

		1. Document-Level Annotations
			Any annotation at the beginning of the file that does not immediately precede a content element is attached to the root `Document` node.

		2. Verbatim Block Exception
			The closing annotation of a `Verbatim` block is a required, integral part of its grammar. It is not treated as attachable metadata. The parser will continue to consume it as part of the `VerbatimBlock` element itself.

4. Implementation Strategy

	The attachment logic will be integrated into the existing parsing and building pipeline, requiring no extra tree-walking passes.

	1. Parsing Stage
		The `linebased` parser will continue to identify annotations and emit them as distinct `NodeType::Annotation` nodes in the intermediate representation (IR) stream, alongside content nodes like paragraphs and sessions.

	2. Building Stage
		The `AstBuilder` will be responsible for the attachment logic. This can be encapsulated in a new module, such as `lex/building/metadata.rs`.

		As the `AstBuilder` iterates through the `ParseNode` stream:
		- It will maintain a temporary buffer for `NodeType::Annotation` nodes.
		- When it encounters a content node (`Paragraph`, `Session`, etc.), it will build the corresponding AST node, attach all annotations from the buffer to its `annotations` field, and then clear the buffer.
		- Any annotations remaining in the buffer at the end of a container will be treated as orphaned and attached to the parent container's AST node.

5. Storage and API

	5.1. Storage
		All AST nodes that can be annotated will have a field: `annotations: Vec<Annotation>`. Storing them in a `Vec` is essential as neither labels nor parameters have uniqueness constraints.

	5.2. API Naming
		The field will be named `annotations` rather than `metadata`. While `metadata` is a more generic term, `annotations` is consistent with the language's syntax (`:: annotation ::`), which will provide a clearer and more consistent experience for developers using the library.

	5.3. API Access
		To simplify access, two API methods should be provided:
		- An iterator over the raw `Annotation` blocks.
		- A flattened iterator that yields all content items within all attached annotations, simplifying access for common use cases.
