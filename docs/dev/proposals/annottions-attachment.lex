Annotations And Metadata in Lex

1. Introduction

	Annotations are a core Lex element providing a way to attach metadata to a document. As such, they are the only element that is not part of the document's content. Annotations are helpful not only for authors and collaborators but also as structured hooks for tooling to build on top of Lex.

	For annotations to be useful, especially for tooling, they should be attached to any AST node as metadata, rather than being treated as content nodes themselves. This proposal outlines a design to achieve this.

2. Problem Statement

	The current model treats annotations as content items, creating an architectural inconsistency. A robust system requires a clear, deterministic, and predictable mechanism for associating annotations with the elements they describe.

	The primary challenge is to define an attachment rule that is both intuitive for authors and capable of gracefully handling mistakes, in line with Lex's design philosophy of avoiding hard parsing errors.

3. Proposed Design

	Under this design, annotations will no longer be part of the content tree. Instead, they will be stored in an `annotations` field on the AST nodes they apply to. The following rules will govern their attachment.

	3.1. Primary Rule: Closest Element Attachment

		An annotation attaches to the closest content element to it, measured by the distance (including blank lines) to both the previous and next elements. If there is a tie (same distance to both previous and next elements), the next element wins.

		This rule applies uniformly at all levels of the document hierarchy: document, sessions, list items, and other containers.

		Example: Annotation between paragraphs
			Some paragraph ends here.

			:: note status="review" ::

			Another paragraph here.
		:: lex ::

		In this case, the annotation is equidistant from both paragraphs, so it attaches to the next paragraph (the one that follows it).

		Example: Annotation closer to following element
			Some paragraph ends here.

			:: note status="review" ::
			Another paragraph here.
		:: lex ::

		Here, the annotation is closer to the following paragraph (no blank line), so it attaches to that paragraph.

	3.2. Document-Level Annotations

		When an annotation appears at the beginning of the document and is followed by a blank line, it attaches to the root `Document` node itself. This provides a mechanism for document-level metadata.

		Example: Document-level annotation
			:: foo ::

			Any element here, the annotation attaches to the document itself.
		:: lex ::

		If the annotation at the document start is not followed by a blank line, the normal closest-element rule applies, and it will attach to the following content element.

	3.3. Container-End Annotations

		When an annotation is the last element in a container, the same closest-element rule applies, except that the "next" element is considered to be the container itself (the annotation's parent element). This ensures that annotations at the end of containers have a predictable attachment target.

		Example: Annotation at document end
			Some paragraph here.

			:: foo ::
		:: lex ::

		In this case, the annotation is closest to the document end (the container), so it attaches to the `Document` node.

	3.4. Special Cases

		Verbatim Block Exception
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

6. Comprehensive Examples

	The following examples illustrate the attachment rules in various scenarios. Each example shows the annotation's target element and explains why it attaches there.

	Example A: Annotation between paragraphs, closest wins
		Some paragraph ends here.

		:: foo ::
		Another paragraph here.
	:: lex ::

		The annotation attaches to "Another paragraph here." because it is closer to the following paragraph (no blank line) than to the previous one (blank line).

	Example B: Annotation between paragraphs, tie goes to next
		Some paragraph ends here.

		:: foo ::

		Another paragraph here.
	:: lex ::

		The annotation is equidistant from both paragraphs (one blank line each). The tie-breaker rule applies: the next element wins, so it attaches to "Another paragraph here."

	Example C: Annotation between paragraphs, same distance, next wins
		Some paragraph ends here.

		:: foo ::

		Another paragraph here.
	:: lex ::

		Same as Example B: equidistant paragraphs, annotation attaches to the following paragraph.

	Example D: Annotation closer to previous element
		Some paragraph ends here.
		:: foo ::

		Another paragraph here.
	:: lex ::

		The annotation is closer to "Some paragraph ends here." (no blank line before annotation), so it attaches to that paragraph.

	Example E: Document start, annotation attaches to document
		:: foo ::

		Some paragraph here.
	:: lex ::

		The annotation is at the document start and followed by a blank line. It attaches to the `Document` node itself.

	Example F: Document start, no blank line, normal rule applies
		:: foo ::
		This is some text.
	:: lex ::

		The annotation is at the document start but not followed by a blank line. The normal closest-element rule applies: it attaches to "This is some text." (closest element).

	Example G: Document start with blank line, next wins
		:: foo ::

		This is some text.
	:: lex ::

		The annotation is equidistant from document start and the following paragraph. The tie-breaker applies: next element wins, so it attaches to "This is some text."

	Example H: Document end, non-continuous
		Some paragraph here.

		:: foo ::
	:: lex ::

		The annotation is closest to the document end (the container). It attaches to the `Document` node.

	Example I: Document end, same distance
		Some paragraph here.

		:: foo ::

	:: lex ::

		The annotation is equidistant from the previous paragraph and the document end. The tie-breaker applies: next element (the container) wins, so it attaches to the `Document` node.

	Example J: Session level, closest wins
		1. This is a Session Title

		    This is the first paragraph of the inner session.
		    :: foo ::

		    This is the first paragraph of the outer session.
		:: lex ::

		The annotation is closer to "This is the first paragraph of the inner session." (no blank line before annotation), so it attaches to that paragraph.

	Example K: Session level, annotation attaches to session
		1. This is a Session Title

		    This is the first paragraph of the inner session.

		    :: foo ::

		This is the first paragraph of the outer session.
	:: lex ::

		The annotation is closest to the session container (the end of the inner session), so it attaches to the `Session` node.

	Example L: Multiple annotations, all attach to document
		:: foo ::

		:: bar ::

		:: baz param=value ::

		:: long form ::
		    This is a long form annotation.

		Some text here.

		- Bread

		- Milk

		:: note :: This is not good.

		There is something in the way she moves.
	:: lex ::

		All annotations at the document start (followed by blank lines) attach to the `Document` node. The annotation between list items attaches to the following paragraph "There is something in the way she moves." (closest element).
