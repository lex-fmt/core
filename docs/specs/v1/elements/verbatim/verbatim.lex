Verbatim Blocks

Introduction

	Verbatim blocks embed non-lex content (source code, binary references) within lex documents. Similar to Markdown's fenced code blocks, but using indentation for delimitation.

Syntax

	Two forms exist:

	Block form (embedded text content):
		Subject:
		    raw content here
		    preserves all formatting
		:: label params ::

	Marker form (no content, typically for binary references):
		Subject:
		:: label params :: optional caption text

	Note: Optional blank line after subject is allowed in both forms.

Verbatim Groups

	Multiple subject/content pairs can share a single closing annotation. This is handy for
	step-by-step shell transcripts or grouped code samples that use the same language.

	Syntax:
		(<subject-line>:
		    <content lines>)+
		:: label params ::

		- Each subject anchors to the indentation wall established by the first subject.
		- Content for every pair must be indented past the wall and preserves blank lines.
		- Content remains optional for parity with marker blocks, but textual payloads are preferred.
		- Blank lines between groups are preserved and do not break the group structure.

	Examples:
		- docs/specs/v1/elements/verbatim/verbatim-11-group-shell.lex - Multiple groups with mixed content
		- docs/specs/v1/elements/verbatim/verbatim-13-group-spades.lex - Groups with blank lines between pairs
		- docs/specs/v1/elements/verbatim/verbatim-12-document-simple.lex - Groups within document context

The Indentation Wall

	Critical rule: The subject line establishes the base indentation level (the "wall").

	Valid:
		Subject:
		    content (indented past wall)
		        more content (further indented - preserved)
		:: label ::

	Invalid:
		Subject:
		  content (not enough indent - breaks the wall)

	The wall ensures:
		- Unambiguous content boundaries without escaping
		- Content can contain :: markers (they're ignored if indented)
		- Clean detection of closing annotation

Fullwidth Mode

	When indentation steals too much horizontal space, content can drop to a
	fixed, absolute wall at column 2 (zero-based index 1). The parser detects
	this automatically when the first non-blank content line starts at that
	column.

		- The closing annotation stays aligned with the subject, so existing
		  readers still see the same structure.
		- All content lines share the same wall regardless of how deeply the block
		  is nested.
		- Blank lines and any indentation beyond the wall remain untouched after
		  extraction.

	Example:
		- docs/specs/v1/elements/verbatim/verbatim-14-fullwidth.lex - Flat table
		  whose rows start near the left margin

Content Preservation

	Everything between subject and closing annotation is preserved exactly:
		- All whitespace (spaces, blank lines)
		- Special characters (no escaping needed)
		- Indentation beyond the wall (part of content)

	Example:
		Code:
		    // spaces    preserved
		    
		    function() { return "::"; }  // :: not treated as marker
		:: javascript ::

Closing Annotation

	The closing annotation:
		- Must be at same indentation level as subject (at the wall)
		- Is a full annotation (can have label, params, text content)
		- Signals end of verbatim block

	Examples:
		:: javascript caption="Hello World" ::
		:: python version=3.11 :: Example code
		:: image src=photo.jpg :: Beautiful sunset

Examples

	Block form with code:
		JavaScript Example:
		    function hello() {
		        return "world";
		    }
		:: javascript ::

	Marker form for images:
Sunset Photo:
		:: image type=jpg, src=sunset.jpg :: As the sun sets over the ocean.

	With parameters and caption:
		API Response:
		    {"status": "ok", "data": [...]}
		:: json format=pretty :: Example API response from /users endpoint

Use Cases

	- Source code examples (any language)
	- Configuration files (JSON, YAML, TOML)
	- Binary data references (images, videos, PDFs)
	- Command output
	- Any non-lex text that needs exact preservation

Implementation Notes

	The AST exposes the first subject/content pair directly on the Verbatim node for backwards
	compatibility. Additional pairs are available through the Verbatim::group() iterator, which
	yields immutable subject/content views. Agents adding formatting logic should iterate over this
	group API so multi-pair verbatim sequences stay cohesive.
