Source String Location / Range Tracking in lex

	This document goes over the need to accurately track source string locations and ranges throughout the entire pipeline up to the final ast construction.

1. Guiding Principles

	1. The Needs for Complete Accuracy

	The parser *must* be able to extract the exact source string text from the ast nodes, with complete accuracy. This parser is made , primarely for language server protocol support, which requires this level of accuracy. Hence let's get this out of the way: this parser serves no purpose if it cannot extract the source string text from the ast nodes.

	 In scenarios such as IDEs, a one off error is not an incremental issue, it's a phase one, it can render the experience unusable.

     2. The Work is Done

	 All the hard work is done by the logos lexer, which provides automatic span tracking. All that lex has to do is not to throw away or corrupt the spans. The code base never has to change a token's span, all it does is to aggregate and at one point convert to line:column positions.

	 That is to say, it requires more work to damage location tracking then to preserve it. All it requires is: be careful the one time you're aggregating and doing the coversion. It's a couple lines of code. Otherwise, just don't touch it.


	3. The Representation: The Truth

	The native, as in the one where that captures the date in a most direct way is the byte offset range. It requires no actual conversion or significant processing when collecting nor aggregating. It's machine efficient and unambiguous.

	Logos, as most parsers uses byte offset ranges, a  Range<usize>: This is the byte-offset span. It represents the exact slice of the original source string that constitutes this token.  While a byte range, it's a Rust byte-level slicing, guaranteeing we don't get an invalid UTF-8 sequence.

	This byte-offset span is the atomic ground truth. All subsequent location information is derived from it. It is precise and unambiguous. This means, we never alter any byte offset. Getting this right is hard, and we start off with it being correct. Any way we meddle with it we are bound to break something, there is no upside to doing so.
	
	4. The Representation: People

	The one thing to consider, though, is that humans are not good at understanding byte offsets. The line:column representation is the natural one for us, so at some point, the machine form is to be converted so that a human can interact with it. 

	The key is to delay this as much as possible. At the first tokenization stage, the number of tokens is very high. As transformations and parsing take places, we coalesce tokens into larger units and the number of tokes decreases drastically. There is no point in knowing where the second "e" of the work where you just read is. You will only interact with paragraph levels, for example.

	Of course, there are no humans at the early stages, which makes this even a moot point. The goal is to only convert after all the pipeline has run , when building the ast. 

	From that point on, it *is* useful to keep the human form as LSP and most tools are built around it, and avoiding mental model friction is wonderful.


2. The Specifics of Lex

	With the principles established , we can turn our attention to the specifics of lex. While logos generates the initial token stream with precise span information, our pipeline will run several transformations, coalescing tokens into larger units. The rationale for this is to make the parsing easier and more efficient. lex , in particular, being a line based language, makes writing it's grammar as lines infinitely easier then as tokens. These transformations are valuable and should not be removed.

	What we don't want is for them to be doing any sort of fuckery with the spans. Like we've said, nothing but pain will come from doing so.
 The good news is: they don't have to anything. If fact the best design is not to touch it. 


	2.1 The Immutable Log Approach

		Our transformations don't inject new information nor (at least they shouldn't) erase any. They only aggregate for ergonomics. 
		During the lex pipeline none of this is needed at all.  And at it's very end, when building the ast, these builders operate on the expanded tokens sources (they have too). Our aggregates are just a temporary facilitator that is to be replaced by the original tokens it represented. 

		Hence the solution couldn't be simpler: each "synthetic" token we create is to have a field "source_tokens" that is the original token vector it was created from. This never has to be touched until the very end where we expand them to feed them to the builders. 

		2.1.1 Ground Truth: The logos lexer output—the flat stream of (Token, Range<usize>)—is the one and only source of truth for location information. It is treated like an immutable log of events.
		2.1.2 Rollup for Structure: The line-based lexer transforms (to_line_tokens, indentation_to_token_tree) exist only to provide a more convenient structure for the parser. They group the raw tokens into LineTokens and LineContainerTokens.
	2.1.3. Preservation, Not Transformation: Crucially, these container tokens should simply store the original `source_tokens` that comprise them. They should not calculate or store their own aggregate spans. The LineToken for "Title:" would contain the (Token::Text("Title"), 0..5) and (Token::Colon, 5..6) tokens, and that's it.
	2.1.4. Parser's Role: The line-based parser uses the simplified tree structure (LineToken, LineContainerToken) to easily match grammar rules (e.g., "a SubjectLine followed by a Container is a Definition").
	5. AST Building: When a rule is matched, the parser gathers all the LineTokens involved in that match. It then "unrolls" them, creating a flat 
		list of all the original source_tokens from the immutable log. This list is then used to:

		- Compute a single, final bounding-box Range<usize>.
		- Convert that Range<usize> to a Location struct.
		- Pass the final Location and extracted text to the common AST builder

			This code is trivial, and that is all the location tracking we need to do: 

				fn compute_bounding_box(
					flat_list: &[(Token, Range<usize>)]) -> Option<Range<usize>> {
					if flat_list.is_empty() {
						return None;
					}
					let min_start = flat_list.iter() .map(|(_, range)| range.start) .min() .unwrap_or(0);
					let max_end = flat_list.iter() .map(|(_, range)| range.end) .max() .unwrap_or(0);
					Some(min_start..max_end)
					}
			:: rust ::

		That is it , the above 8 lines of code is all we need to get to the ast. 

		Recap: 
			- Everything token we create stores it's raw , original form as source_tokens.
			- At ast build time, the parser unrolls these, processing them through the function above, 
			- The ast builders will use the raw tokens and store the location from them.

3. The Actual Scenario


	Now that we've seen how simple this is , we can cry by looking at what we are currently doing: 

	1. Transformations will create their own span ranges, with techniques raging from reasonable to downright insane.
    2. At other times we will simply discard all of it. 
    3. We will convert to locations early on, carrying the both representations. This conversion is buggy at times, and even when it is not, we consistently switch representations , resulting in a code that is hard to understand and maintain, and easy to err upon.
    
	3.1 The Way Forward
    

		The silver linging here is that fixing this is simple. We'll keep a more detailed version of this on github, but the core idea is: 
		- Every token transformation is to store the original tokens it was created from.
		- No transformation is to aggregate or create spans in any way. 
		- At the ast border the code above gets called before passing the tokens to the builders.

	3.2 Safeguarding

		There is an additional problem. Many places that takes tokens have two signatures, one being the correct one, where the source is passed, and the other without it. There is NO ACTUAL USE CASE for the second one. This is a parser, the only thing it does is translate a string into an ast, there has to be a source string.

		The reason this happened and perpetuates is testing: this wild and free version saves one line of code per test or something like that. But the cost it creates is way more: as different developers will mix these. Since any location use happens at the very end, the developer rarely is aware of the problem. The code at that point, however, is constantly being broken by this.
		The solution is trivial: type the extra line OR use the ready made factories (which you should any way).

		Hence we need to remove all versions that don't take a source string, while updateing tests that rely on it. There is no getting around it, as this is a parser, and the only thing it does is translate a string into an ast, there has to be a source string.	

 .      The work here looks like this: 

		- Assess how many places use the sourceless signature for creating tokens or ast nodes.
        - Updating them to use the factory functions. 
        - Running the test suite to ensure no regressions.
        - Removing the sourceless signature from the codebase.