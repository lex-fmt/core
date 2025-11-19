The Lex Format, Parsing, Tokens and Grammar

	Lex is a plain text format for structured information than can scale from a quick one line note all the way up to scientific writings, while being easy for people to write and write without tooling. It can be fully understood by a human reader without prior knowledge, and require little to no traning when writing it.
	Lex's goal is to fade away, making the format transparent and letting authors focus on the ideas. This is done by skewing formal syntax elements, and piggybacking on established patterns of text formating formed in the last couple of centuries. As such, most of those are familiar to authors, hence appearing to be effortlessly. In tandem, it leverages visual appearance, in the form of indentation to convey structure.
	It is designed to be forgiving, that is, there is (outside of debugging) no such thing as a parser or syntax error. Worst case scenario it's interpreted as a pargraph, which can have any content.  This is key to make the format scale from a quick note to a full fledged structured document for publishing.
    Tooling can have linting, or helpers to suggest improvements, but the parser itself should not fail.

Design Principles: 
	- Easy to read and write without tooling.
	- Whitespace significant: has indentation, blank lines are meaningful and users spaces are preserved.
	- Arbitrarily nestable: elements (almost all) can contain children, ad-infinitum.
	- Forgiving: parsing never fails, and things degrade gracefully to paragraphs ans sessions.
	- Minimal formal syntax: only on meta or processing directives, never in text's content.


1.Structure


	1.1 Document and Sessions

		Lex documents are plain text, utf-8 encoded files with the file extension .lex. Line width is not limited, and is considered a presentation detail. Best practice dictates only limiting line length when publishing, not while authoring.
		The document node holds the document meta data and the content's root node, which is a session node. The structure of the document then is a tree of sessions, which can be nested arbitrarily. This creates powerful addressing capabilities as one can target any sub-session from an index.


	1.2 Nesting	

		The ability to make deep structures is core to Lex, and this is reflected throughout the grammar. In face the only element that does not contain children is the paragraph and the verbatim block (by definition content that is not parsed).
	    Nesting is pretty unrestricted with the following logical exceptions: 

		- Only sessions can contain other sessions: you don't want a session popping up in the middle of a list item.
        - Annotations (metadata) cannot host inner annotations, that is you can't have metadata on metadata (pretty reasonable, no?)


     1.3 Elements

		There are four type of elements: blocks, containers, inlines and components

		Components: 
			Carry a bit of information inside and element, only used in metadata: label and parameters.

      	Inlines:
			Specialization of text spans inside text lines. These are handled differently than blocks, as they are much simpler and do not affect structure nor the surrounding context.

      	Blocks: 
			These are the core elements of Lex, and the ones that users work with. Block elements are line based, that is they take at least a full line.

		Containers:
			Containers a special kind of element that can contain children, and are part of nestable block elements. 

		Lex's elements are: 
		- Sessions: have a title and it's child content.
		- Paragraphs
		- Lists: have multiple list items, each with marker and optional child content.
		- Definitions: have a subject (term) and it's content.
		- Annotations: metada , have a data tag and optional content.
		- Verbatim Blocks: has a subject, optional content and data tag (label, optional parameters)


2. Grammar

	Lex's grammar is line based, that is each element is defined by a sequence of lines. Seen this way, the grammer is actually quite simple, to the point that it can be parsed by a simple regex engine (which it indeed does).

 
	2.1 Whitespace

		Whitespace is indeed significant. Trailing whitespace (that is whitespace after the last non-whitespace character) is ignored. It's not discarded in order to keep location tracking correct, but it's not grammatical significant
		Prefixed whitespace is key, as it denotes structure via indentation. Tabs count as  tab-width spaces, which is 4 by default. That is the indentation-width, that is how many spaces form an indentation level.


	2.2 Blank Lines

		Blank lines are lines of text where only whitespace characters appears before the new line.
        They are semantically significant, but only that they exist, the exact whitespace content is not taken into account. 
        How many consecutive blank lines is not taken into account, only that there is at least one. Again , multiple blank lines are not discarded, but treated as a blank line group.


	2.3 Tokens

		Lex opts for handling more complexity in the lexing stage in order to keep the parsing stage very simple. This implies in greater token complexity, and this is the origin of several token types. See Lexing for more details.

			Even though the grammar operates mostly over lines, we have two layers of tokens: 
        - Structural Tokens: indent, dedent.
		- Core Tokens: character/word level tokens. They are produced by the logos lexer [1].
        - Line Tokens: a group of core tokens in a single line, and used in the actual parsing.[2]
        - Line Container Token: a vector of line tokens or other line container tokens. This ia a tree representation of each level's lines. This is created and used by the parser.
        - Synthetic Tokens: tokens that are not produced by the logos lexer, but are created by the lexing pipeline to capture context information from parent to children elements so that parsing can be done in a regular single pass.


3. Syntax


	3.1 Markers

		Markers are characters or small character sequences that have meaning in the grammar. There is only one syntax marker, that is a marker that is Lex introduced. All others are naturally occurring in ordinary text, and with the meaning they already convey.


		3.1 The Lex marker (Lex)

			In keeping with Lex's ethos of putting content first there is only one formal syntax element: the lex-marker, a double colon (::).
	

			3.1.1 Data Nodes

				Accordingly, it's only used in metadata, there is in Data nodes. Data nodes group a label (an identifier) and optional parameters. It's syntax is: 
				<data> = <lex-marker> <whitespace> <label> (<whitespace> <parameters>)?
				Example: 
					:: note 
					:: note severity=high ::
				:: syntax

				Data nodes always appear in the start of a line (after whitespace), so they are very easy to identify. 


		3.2 Sequence Makers (Natural)

			Serial elements in Lex like lists and sessions can be decorated by sequence markers. These vary from plain formatting (dash) to explicit sequencing as in numbers, letters and roman numerals. These can be separated by periods or paranthesis and come in short and extended forms: 
			<sequence-marker> = <plain-marker> | (<ordered-marker><separarot>)+
			Examples are -, 1., a., a), 1.b.II. and so on.


		3.3 Subject Markers (Natural)

			Some elements take the form of subject and content , as in definitions and verbatim blocks. The subject is marked by an ending colon(:).


	3.2 Lines

		Being lined based, all the grammar needs is the to have line tokens in order to parse any level of elements. Only annotations and end of verbatim blocks use data nodes, that means that pretty much all of Lex needs to be parsed from naturally occurring text lines, indentation and blank lines. 
		Since this still is happening in the lexing stage, each line must be tokenized into one category. In the real world, a line might be more than one possible category. For example a line might have a sequence marker and a subject marker (for example "1. Recap:")
			For this reason, line tokens can be OR tokens at times, and at other the order of line categorization is crucial to getting the right result.[3] While there are only a few consequential marks in lines (blank, data, subject, list ) having them denormalized is required to have parsing simpler. The definitive set is the LineType enum (blank, annotation start/end, data, subject, list, subject-or-list-item, paragraph, dialog, indent, dedent), and containers are a separate structural node, not a line token.

			These are the line tokens: 
				- BlankLine (empty or whitespace only)
				- AnnotationEndLine: a line starting with :: marker and having no further content
				- AnnotationStartLine: a data node + lex marker 
				- DataLine: :: label params? (no closing :: marker)
				- SubjectLine: Line ending with colon (could be subject/definition/session title)
				- ListLine: Line starting with list marker (-, 1., a., I., etc.)
				- SubjectOrListItemLine: Line starting with list marker and ending with colon ()
				- ParagraphLine: Any other line (paragraph text)
				- DialogLine: a line that starts with a dash, but is marked not to be a list item.
				- Indent / Dedent: structural markers passed through from indentation handling.
			And to represent a group of lines at the same live, there is a LineContainer
	
These conclude the description of the grammar and syntax. With that in mind, we will now dive into the various parsing stages.			


4. The Parser Design

	Now it's easier to understand the claim that Lex is a simple format, and yet quite hard to parse. Tactically it is stateful, recursive, line based and indentation significant. The combination of these makes it a parsing nightmare. 

	While these are all true, the format is design with enough constraints so that, if correctly implemented, it's quite easy to parse. However it does mean that using available libraries simply won't work. Libraries can handle context free, token based , non indentation significant grammars. At best, they are flexible enough to handle one of these patterns, but never all of them.

	After significant research and experimentation we settled on a design that is a bit off-the-beaten-path, but nicely breaks down complexity into very simple chunks. 

	Instead of a straight lexing -> parsing pipeline, lex-parser does the following steps: 

		1. Semantic Indentation: we convert indent tokens into semantic events as indent and dedent.
		2. We group tokens into lines.
        3. We build a tree of line groups reflecting the nesting structure.[4]
        4. We inject context information into each group allowing parsing to only read each level's lines.

    On their own, each step is fairly simple, their total sum being some 500 lines of code. Additionally they are easy to test and verify.

	They key here is that parsing only needs to read each levels line , which can can include a LineContainer (that is , there is child content there), with no tree traversal needed. Parsing is done declaratively by processing the grammar patterns (regular strings ) through rust's regex engine. Put another way, once tokens are grouped into a tree of lines, parsing can be done in a regular single pass.

	Whether passes 2-4 are indeed lexing or actual parsing is left as a bike shedding exercise. The criteria for calling these lexing has been that each tranformation is simply a groupping of tokens, there is no semantics.

			In addition the transformations over tokens, the codebase separates the semantic analysis (in lex/parsing) from the ast building (in lex/building) and finally the final document assembly step (in lex/assembly). These are done with the same intention: keeping complexity localized and shallow at every one of these layers and making the system more testable. Line grouping and tree building happen at the parsing stage, after lexing has already produced indent/dedent-aware flat tokens.


5. Parsing End To End

	We will now dive into the actual stages and their steps from a string of Lex source up to the final AST.


	5.1 Lexing

		We now run transformations over the tokens. First we store the core tokens as a TokenStream for easier handling, then run transformations one by one. Each receiving a TokenStream and returning a TokenStream.

		In common, all of these processes store the source tokens in the groupped token under  `source_tokens` field, which preserves information entirely and allows for easy unrolling at the final stages.

		Logo's tokens carry the byte range of their source text. This information will not be used in the parsing pipeline at wall, but has to be perfectly preserved for location tracking on the tooling that will use the AST. It is critical that this be left as it. The ast building stage will handle this information, but it's key that no other code changes it, and at every step it's integrity is preserved.


			5.1.1 Base Tokenization

					We leverage the logos lexer to tokenize the source text's into core tokens. This is done declaratively with no custom logic, and could not be simpler.[5]


			5.1.2 Semantic Indentation

				The logos lexer will produce indentation tokens, that is groupping several spaces or tabs into a single token. However, indentation tokens per se, are not useful. We don't want to know how many spaces per line there are, but we want to know about indentation levels and what's inside each one. For this, we want to track indent and dedent events, which lets us neatly tell levels and their content.
				This transformation is a stateful machine that tracks changes in indentation levels and emits indent and dedent events. In itself, this is trivial, and how most indentation handling is done. At this point, indent/dedent could be replaced for open/close braces in more c-style languages with to the same effect.
					Indent tokens store the original indentation token, while dedent tokens are synthetic and have no source tokens of their own.[6]


			5.1.3 Line Grouping

					Here we split tokens by line breaks into groups of tokens. Each group is a Line token and which category is determined by the tokens inside [#3.2]. This is also a fairly simple transformation. 
					Each line group is fairly simple and only contains the source tokens it uses. It does not process their information , and hence we consider this a lexing step as well.[7]

		At this point, lexing is complete. We have a TokenStream of Line tokens + indent/dedent tokens.


	5.2   Parsing (Semantic Analysis)

		At the very begging of parsing we will group line tokens into a tree of LineContainers. What this gives us is the ability to parse each level in isolation. Because we don't need to know what a LineContent has , but only that it is a line content, we can parse each level with a regular regex. We simply print token names and match the grammar patterns agains them.[8]

		When tokens are matched, we create intermediate representation node, which carry only two bits of information: the node matched and which tokens it uses. 

		This allows us to separate the semantic analysis from the ast building. This is a good thing overall, but was instrumental during development, as we ran multiple parsers in parallel and the ast building had to be unified (correct parsing would result in the same node types + tokens )


	5.3 AST Building

		From the IR nodes, we build tha actual AST nodes.[9] During this step, two important things happen: 

			1. We unroll source tokens so that ast nodes have acccess to token values .
			2. The location from tokens is used to calculate the location for the the ast node.
            3. The location is transformed from  byte range to a dual byte range + line:column position.
        At this stage we create the Document node, it's root session node and the ast will be attached to it. 


	5.4 Document assembly

		We do have a document ast node, but it's not yet complete. Annotations, which are metadata, are always attached to AST nodes, so they can be very targeted.  Only with the full document in place we can attach annotations to their correct target nodes.[10]
		This is harder than it seems. Keeping Lex ethos of not enforcing structure, this needs to deal with several ambiguous cases, including some complex logic for calculating "human understanding" distance between elements[12].

	5.5 Inline Parsing

		Finally, with the full and correctly annotated document, we will parse the TextContent nodes for inline elements. This parsing is much simpler, as it has formal start/end tokens as has no structural elements.

		Inline parsing is done by a declarative engine that will process each element declaration.[11] For some , this is a flat transformation (i.e. it only wraps up the text into a node, as in bold or italic). Others are more involved, as in references, in which the engine will execute a callback with the text content and return a node. 
		This solves elegantly the fact that most inlines are simple and very much the same structure, while allowing for more complex ones to handle their specific needs. 


6. Structure, Children, Indentation and the AST

	They design for children node and the AST has a point that is too easy to miss, and missing causes a whole lot of problems.

	The first key aspect is: indentation is the manifestation of a container node, that is, where  elements holds their children. This is a subtle point, but one worth making. 

	For example, why in sessions is the title on the same indentation as it's sibling nodes, when it's content is indented? Answer: because the title is a child of the session node, and a sessions content is a child of session.content, a container. 

	Likewise lists elements do not ident, that's why they are shown in the same indentation as their items and siblings. On nested lists, a list's items content container holds the nested list, which is why it's indented. Let's look at a complete example. 

	1. Packing
	2. Groceries
		2.1 Milk
		2.2 Eggs

	This is what the ast looks like: 

		<list>
			<item>Packing</item>
			<item> Groceries
				<content> <- this is container, this causes indentation
					<list>
						<item>Milk</item>
						<item>Eggs</item>
					</list>
				</content>
			</item>
		</list>

	That is why the outer list is not indented, while the inner list is.

    This is true for sessions (titles are outside it's children), annotations (data is note it's content), definitions (subject is not it's content) and verbatim blocks (subject is not it's content).

	One can see a patter here: most elements in Lex have a form: 

		<preceding-blank-line>?
		<head>
	    <blank-line>?
		<indent> 
			<content>
			</content>
        <dedent>
		<tail>?
	
	Seen in this way, it's now clear how one can parse a full level without peeking into the children, because the container / content is enough to know what to do.

	This is to say that save for pargraph, flat lists, and short annotation, all elements use a combination of head, presence of blank lines, and dedent and the tail to determine what it's parsing.

	Once you factor in the lack of formal syntax, that heads can be regular, list or subject lines and tails can be data lines or regular lines, and it's clear how this is a delicate balancing act. All it takes to parse is:
	1. Does the head line has list markers, colon, both or neither?
	2. Is there a blank line between the head and the content?
	3. Is there a indented content? 
	4. Does the tail ends with a lex marker? 
	In short what form is the head and tail lines, and between is there a blank line and or content? 

	Table: Nested Elements Structure and Parsing:

		| Element     | Separator Before Head | Head                | Blank (head→content) | Content  | Tail          	|
		|-------------|-----------------------|---------------------|----------------------|----------|------------------|
		| Session     | Blank or boundary     | ParagraphLine       | Yes                  | Yes      | dedent        	|
		| Definition  | Optional blank        | SubjectLine         | No                   | Yes      | dedent        	|
		| Verbatim    | Optional blank        | SubjectLine         | Optional             | Optional | dedent+ DataLine |
		| Annotation  | Optional blank        | AnnotationStartLine | Yes                  | Yes      | AnnotationEnd 	|
		| List        | Optional blank        | ListLine            | No                   | Yes      | No               | 
		|-------------------------------------------------------------------------------------------------------------|

    Table: Flat Elements Structure and Parsing:	

		| Element     | Prec. Blank | Head                |  Tail       		 					|
		|-------------|-------------|---------------------|-----------------------------------------|
		| Paragraph   | Optional    | Any Line 	          | BlankLine or Dedent                     | 
        | List        | Yes         | ListLine | No       | BlankLine or Dedent                     |
		|-------------------------------------------------------------------------------------------|

	Table Special Casing Rules: 

		| Element    | Rule             | About 													| 
		|------------|------------------|-----------------------------------------------------------|
		| Paragraph  | Dialog           | A formal way to specify that - lines are dialogs (parag.) |
        | List       | Two Item Minimum | A list must have 2+ items, otherwise it's a paragraph     |
        | Annotation | Short            | The short form of annotations are one liners              | 
        | Verbatim   | Full Width Form  | Verbatim content can break indentation rules              |
        | Verb.Group | Multiple Groups  | Multiple subject + content and only 1 closing.            |
		|-------------------------------------------------------------------------------------------|

	:: table
	

	There are a couple of interesting things to note here. The first is that all container elements, salvo for Verbatim blocks are terminated by a dedent. That it, you don't know where they ended, you just know that something else started.
	Sessions remain special because the title must be followed by a blank line before content, but the separator *before* the title can be either a blank-line in the current container or simply the boundary after a previous child. Blank lines stay with the container where they appear; they are not hoisted out of children. A boundary (dedent) therefore also counts as a separator when starting a new session sibling. Consider:

	1. I'm the outter session.

		1.1 I'm the middle session.

			I'm just a pargraph.
    :: lex

	Consider the parsing of the middle session. As it's the very first element of the session, the preceding blank line is part of it's parent session. It can see the following blank line before the pargraph just fine, as it belongs to it. But the first blank line is out of it's reach. The parser therefore treats either a visible blank line *or* the boundary after the previous child as a valid separator before a new session, keeping blank-line ownership intact while still parsing correctly across container edges.

	The obvious solultion is to imperatively walk the tree up and check if the parent session has a preceding blank line. This works but this makes the grammar context sensitive, and now things are way more complicated, good by simple regular langauge parser.

	The way this is handled is that we inject a synthetic token that represents the preceding blank line. This token is not produced by the logos lexer, but is created by the lexing pipeline to capture context information from parent to children elements so that parsing can be done in a regular single pass. As expected, this tokens is not consumed nor becomes a blank line node, but it's only used to decide on the parsing of the child elements.


7. Verbatim Elements

	Verbatim elements represent non Lex content. This can be any binary encoded data, such as images or videos or text in another formal language, most commonly programming language's code. Since the whole point of the element is to say: hands off, do not parse this, just preserve it, you'd think that it would be a simple element, but in reality this is by far the most complex element in Lex, and it warrants some explanation. 


	7.1 Parsing Verbatim Blocks

		The first point is that, since it can hold non Lex content, it's content can't be parsed. It can be lexed without prejudice, but not parsed. No only it would be gibberish, but worse, in case it would trigger indent and dedent events, it would throw off the parsing and break the document.

		This has two consequences: that verbatim parsing must come first, lest it's content create havoc on the structure and also that identifying it's end marker has to be very easy. That's the reason why it ends in a data node, which is the only form that is not common on regular text. 

			The verbatim parsing is the only stateful parsing in the pipeline. It matches a subject line, then either an indented container (in-flow) or flat lines (full-width/groups), and requires the closing annotation at the same indentation as the subject.


	7.2 Content and the Indentation Wall


		7.2.1 In-Flow Mode

			Verbatim content can be pretty much anything, and that includes any space characters, which we must not interpret as indentation, nor discard, as it's content.
			The way to think about this is through the indentation wall:

				I'm A verbatim Block Subject:
					|<- this is the indentation wall, that is the subject's + 1 level up
					I'm the first content line
					But content can be indented whoever I please
		error ->| as long as it's past the wall
				:: text 

			Verbatim content starts at the wall, until the end of line.  Whitespace characters should be preserved as content. 
				Content cannot, start, however before the wall, lest we had no way to determine the of the block.
			This logic allows for a neat trick: that verbatim blocks do not need to quote any content. Even if a line looks like a data node, the fact that it's not in the same level as the subject means it's not the block's end marker.

			In this mode, called In-flow Mode, the verbatim content is indented just like any other children content in Lex, +1 from their parent.
			

		7.2.2. Full-Width Mode

			At times, verbatim content is very wide, as in tables. In these cases, the various indentation levels in the Lex document can consume valuable space which would throw off the content making it either hard to read or truncated by some tools.
				For this cases, the full-width mode allows the content to take (almost) all columns. In this mode, the wall is at user-facing column 2 (zero-based column 1), so content can hug the left margin without looking like a closing annotation.

			This is an example: 

  Here is the content.
  |<- this is the wall

            :: lex

				The block's mode is determined by the position of the first non-whitespace character of the first content line. If it's at user-facing column 2, it's a full-width mode block; otherwise it's in-flow.
				The reason for column 2: column 1 would be indistinguishable from the subject's indentation, while a full indent would lose horizontal space. Column 2 preserves visual separation without looking like an error.


8. Testing

	Lex is a novel format, for which there is no establish body of source text nor a reference parser to compare against. Adding insult to injury, the format is still evolving, so specs change, and in some ways it looks like markdown just enough to create confusion.

	The corollary here being that getting correct Lex source text is not trivial, and if you make one up, the odds of it being slightly off are high. If one tests the parser agains an illegal source string, all goes to waste: we will have a parser tuned to the wrong thing. Worst off, as each test might produce it's slight variation, we will have an unpredictable, complex and wrong parser. If that was not enough, come a change in the spec, and now we must hunt down and review hundreds of ad-hoc strings in test files.


	8.1. The Spec Sample Files

		For this reason, all testing must use the official sample files, which are vettoed, curated and reviewed during spec changes. Of course this does not apply to the string form only, but for tokens and any intermediary processed formats. If we can't reliably come up with the string form, nevermind with that string after tokenization and processed. 	

		Here is where Lexplore comes in [13]. It includes a loader that will load from the official sample files and can return the data in various formats: string, tokens, line container, ir nodes and ast nodes.

		The sample files are organized: 

		- By elements: 
			- Isolated elements (only the element itself):
			- In Document: mixed with other elements
		- Benchmark: full documents that are used to test the parser.
		- Trifecta: a mix of sessions, paragraphs and lists, the structural elements..

		These come with handy functions to load them, get the isolated element ast node and more.


	8.2. The AST assertions

		As mentioned, the spec is in flux. This means that the lower level AST nodes are subject to change. If a tests walks through the node directly, on spec changes, it will break. 

		Additionally, low level ast tests tend to be very superficial, doing things like element counts (which is bound to be wrong) and other minor checks.

		For this reason, all AST testing is done by this powerful library [14]. It will conveniently let you verify your choice of information from any element, including children and other nested nodes. Not only it's much faster and easier to write, but on spec changes, only one change might be needed. 



Notes:

	1. specs/v1/grammar-core.lex
	2. specs/v1/grammar-line.lex
	3. lex-parser/src/lex/lexing/line_classification.rs
	4. lex-parser/src/lex/token/to_line_container.rs
	5. lex-parser/src/lex/lexing/base_tokenization.rs
	6. lex-parser/src/lex/lexing/transformations/semantic_indentation.rs
	7. lex-parser/src/lex/lexing/line_grouping.rs
	8. lex-parser/src/lex/parsing/engine.rs
	9. lex-parser/src/lex/building/ast_tree.rs
	10. lex-parser/src/lex/assembling/stages/attach_annotations.rs
	11. lex-parser/src/lex/inlines/parser.rs
	12. lex-parser/src/lex/assembling/stages/attach_annotations/distance.rs
	13. lex-parser/src/lex/testing/lexplore.rs
	14. lex-parser/src/lex/testing/ast_assertions.rs
