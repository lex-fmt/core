Source String Location / Range Tracking in lex

	This document goes over the need to accurately track source string locations and ranges throughout the entire pipeline up to the final AST construction.

1. Guiding Principles

	1. The Need for Complete Accuracy

	The parser *must* be able to extract the exact source string text from the AST nodes, with complete accuracy. This parser is made, primarily for language server protocol support, which requires this level of accuracy. Hence let's get this out of the way: this parser serves no purpose if it cannot extract the source string text from the AST nodes.

	In scenarios such as IDEs, a one-off error is not an incremental issue, it's a phase one - it can render the experience unusable.

	2. The Work is Done

	All the hard work is done by the logos lexer, which provides automatic span tracking. All that lex has to do is not throw away or corrupt the spans. The codebase never has to change a token's span, all it does is aggregate and at one point convert to line:column positions.

	That is to say, it requires more work to damage location tracking than to preserve it. All it requires is: be careful the one time you're aggregating and doing the conversion. It's a couple lines of code. Otherwise, just don't touch it.

	3. The Representation: The Truth

	The native representation - the one that captures the data in the most direct way - is the byte offset range. It requires no actual conversion or significant processing when collecting nor aggregating. It's machine efficient and unambiguous.

	Logos, like most parsers, uses byte offset ranges, a Range<usize>: This is the byte-offset span. It represents the exact slice of the original source string that constitutes this token. While a byte range, it's Rust byte-level slicing, guaranteeing we don't get an invalid UTF-8 sequence.

	This byte-offset span is the atomic ground truth. All subsequent location information is derived from it. It is precise and unambiguous. This means we never alter any byte offset. Getting this right is hard, and we start off with it being correct. Any way we meddle with it we are bound to break something; there is no upside to doing so.

	4. The Representation: People

	The one thing to consider, though, is that humans are not good at understanding byte offsets. The line:column representation is the natural one for us, so at some point, the machine form is to be converted so that a human can interact with it.

	The key is to delay this as much as possible. At the first tokenization stage, the number of tokens is very high. As transformations and parsing take place, we coalesce tokens into larger units and the number of tokens decreases drastically. There is no point in knowing where the second "e" of the word you just read is. You will only interact with paragraph levels, for example.

	Of course, there are no humans at the early stages, which makes this even a moot point. The goal is to only convert after all the pipeline has run, when building the AST.

	From that point on, it *is* useful to keep the human form as LSP and most tools are built around it, and avoiding mental model friction is wonderful.

2. The Specifics of Lex

	With the principles established, we can turn our attention to the specifics of lex. While logos generates the initial token stream with precise span information, our pipeline runs several transformations, coalescing tokens into larger units. The rationale for this is to make parsing easier and more efficient. Lex, in particular, being a line-based language, makes writing its grammar as lines infinitely easier than as tokens. These transformations are valuable and should not be removed.

	What we don't want is for them to be doing any sort of manipulation with the spans. Like we've said, nothing but pain will come from doing so.

	The good news is: they don't have to do anything. In fact the best design is not to touch it.

	2.1 The Immutable Log Approach

		Our transformations don't inject new information nor (at least they shouldn't) erase any. They only aggregate for ergonomics.

		During the lex pipeline none of this manipulation is needed at all. And at its very end, when building the AST, these builders operate on the expanded token sources (they have to). Our aggregates are just a temporary facilitator that is to be replaced by the original tokens it represented.

		Hence the solution couldn't be simpler: each "synthetic" token we create (Indent, Dedent, BlankLine) has a field "source_tokens" that is the original token vector it was created from. This never has to be touched until the very end where we expand them to feed them to the builders.

		2.1.1 Ground Truth: The logos lexer output—the flat stream of (Token, Range<usize>)—is the one and only source of truth for location information. It is treated like an immutable log of events.

		2.1.2 Aggregation for Structure: Pipeline transformations (NormalizeWhitespace, SemanticIndentation, BlankLines) exist to provide more convenient structure for parsing. They create aggregate tokens (Indent, Dedent, BlankLine) that store original source_tokens.

		2.1.3. Preservation, Not Transformation: Crucially, these aggregate tokens store the original source_tokens that comprise them. They should not calculate or store their own aggregate spans (they use placeholder 0..0 ranges). The Indent token for "    " would contain the (Token::Indentation, 5..9) source token, and that's it.

		2.1.4. Parser's Role: The linebased parser consumes the flat token stream, builds its own LineContainer tree, and when it's time to build AST nodes it expands every aggregate back into the original source_tokens to return to the immutable log.

	3. AST Building: When creating an AST node, the builder gathers all tokens involved. It extracts source_tokens from any aggregates, creating a flat list of all original tokens from the immutable log. This list is then used to:

		- Compute a single, final bounding-box Range<usize>
		- Convert that Range<usize> to a Location struct
		- Pass the final Location and extracted text to the common AST builder

		This code is trivial, and that is all the location tracking we need to do:
		:: file src/lex/building/token/processing.rs ::

			fn compute_bounding_box(
				flat_list: &[(Token, Range<usize>)]) -> Option<Range<usize>> {
				if flat_list.is_empty() {
					return None;
				}
				let min_start = flat_list.iter().map(|(_, range)| range.start).min().unwrap_or(0);
				let max_end = flat_list.iter().map(|(_, range)| range.end).max().unwrap_or(0);
				Some(min_start..max_end)
			}
		:: rust ::

		That is it - the above 8 lines of code is all we need to get to the AST.

		Recap:
			- Every aggregate token we create stores its raw, original form as source_tokens
			- At AST build time, the builder expands these, processing them through the function above
			- The AST builders use the raw tokens and store the location from them

4. After Simplification

	The recent TokenStream simplification reinforced these principles:

	Before:
		- Pipeline built trees (ToLineTokensMapper, IndentationToTreeMapper)
		- Complex tree-walking infrastructure
		- Locations preserved but infrastructure was complex

	After:
		- Pipeline stays flat (just Vec<(Token, Range<usize>)>)
		- Tree building moved into linebased parser only
		- Locations still preserved via Immutable Log
		- Much simpler infrastructure

	The Immutable Log principle remains unchanged and central. The simplification just removed unnecessary complexity while maintaining perfect location tracking.

5. Verification In Practice

	With location data acting as the ground truth for LSP and the viewer, we now treat validation as part of the contract.

	5.1 Aggregation Boundaries

		`compute_location_from_locations` no longer guesses start/end columns independently. It computes lexicographic minima/maxima and carries byte spans so document/session ranges can be mapped back to source slices. Regression tests cover overlapping and disjoint child spans.

	5.2 SourceLocation Cache

		`AstTreeBuilder` constructs a `SourceLocation` once per document and reuses it across all AST constructors. Every normalized token API now receives a shared reference so byte→line conversions are O(1) per node and cannot drift due to multiple reconstructions.

	5.3 Fixture Sweeps

		`tests/location_integrity.rs` parses representative fixtures (nested trifecta doc + paragraph with special characters) and walks the entire AST. Each node must satisfy:
			:: bullet ::
				* `range.span` strictly increases (start < end)
				* slicing `source[span]` matches the stored text content when available
				* no node retains the default `(0,0)..(0,0)` range

	5.4 Cursor Hit Guarantees

		The same test exercises `Session::element_at` end-to-end. For every child range we pick a position inside it and assert the lookup returns an ancestor whose range contains that cursor. This ensures IDE navigation (click in editor → highlight tree node) remains reliable.

	5.5 Builder-Specific Tests

		Targeted unit tests cover the tricky builders:
			:: bullet ::
				* Parameter/data nodes verify label and parameter byte spans become `Range` objects.
				* Verbatim blocks with multiple subject/content groups assert we aggregate subject lines, content, and closing data into one enclosing range while preserving per-line spans.

	The combination of immutable byte spans, a single conversion point, and automated fixture sweeps keeps us confident that any future regressions in location tracking are caught immediately.
