Foreign Blocks

Introduction

	Foreign blocks embed non-lex content (source code, binary references) within lex documents. Similar to Markdown's fenced code blocks, but using indentation for delimitation.

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
		- Signals end of foreign block

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
