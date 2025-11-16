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
        - Structural Tokens: indent, dendent, EOF.
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
					<data> = $<lex-marker> <whitespace> <label> (<whitespace> <parameters>)?
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
		For this reason, line tokens can be OR tokens at times, and at other the order of line categorization is crucial to getting the right result.[3] While there are only a few consequential marks in lines (blank, data, subject, list ) having them denormalized is required to have parsing simpler, hence we have 9 line tokes instead of 4. Mainly when data show up by itself or part of an annotation and whether sequence markers and subjcts are mixed.

		These are the line tokens: 
			- BlankLine (empty or whitespace only)
			- AnnotationEndLine: a line starting with :: marker and having no further content
			- AnnotationStartLine: a data node + lex marker 
			- DataLine: Data line: :: label params? (no closing :: marker)
			- SubjectLine:Line ending with colon (could be subject/definition/session title)
			- ListLineLine starting with list marker (-, 1., a., I., etc.)
			- SubjectOrListItemLine: Line starting with list marker and ending with colon ()
			- PargraphLine: Any other line (paragraph text)
			- DialogLine: a line that starts with a dash, but is marked not to be a list item.
            - LineContainer: a group of lines / line containers representing a single nesting level.
		And to represent a group of lines at the same live, there is a Line
	
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

	In addition the transformations over tokens, the codebase separates the semantic analysis (in lex/parsing) from the ast building (in lex/building) and finally the final document assembly step (in lex/assembly). These are done with the same intention: keeping complexity localized and shallow at every one of these layers and making the system more testable. 


5. Parsing End To End

	We will now dive into the actual stages and their steps from a string of Lex source up to the final AST.


	5.1 Lexing

		We now run transformations over the tokens. First we store the core tokens as a TokenStream for easier handling, then run transformations one by one. Each receiving a TokenStream and returning a TokenStream.

		In common, all of these processes store the source tokens in the groupped token under  `source_tokens` field, which preserves information entirely and allows for easy unrolling at the final stages.

			5.1.1 Base Tokenization

					We leverage the logos lexer to tokenize the source text's into core tokens. This is done declaratively with no custom logic, and could not be simpler.[5]

			5.1.2 Semantic Indentation

				The logos lexer will produce indentation tokens, that is groupping several spaces or tabs into a single token. However, indentation tokens per se, are not useful. We don't want to know how many spaces per line there are, but we want to know about indentation levels and what's inside each one. For this, we want to track indent and dedent events, which lets us neatly tell levels and their content.
				This transformation is a stateful machine that tracks changes in indentation levels and emits indent and dedent events. In itself, this is trivial, and how most indentation handling is done. At this point, indent/dedent could be replaced for open/close braces in more c-style languages with to the same effect.
				Like any other token transformation, the indent/dedent tokens store their constituent  source tokens for location tracking and information preservation.[6]


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
            2. The location is transformed from  byte range to a dual byte range + line:column position..

        At this stage we create the Document node, it's root session node and the ast will be attached to it. 

	5.4 Document assembly

		We do have a document ast node, but it's not yet complete. Annotations, which are metadata, are always attached to AST nodes, so they can be very targeted.  Only with the full document in place we can attach annotations to their correct target nodes.[10]
		This is harder than it seems. Keeping Lex ethos of not enforcing structure, this needs to deal with several ambiguous cases, including some complex logic for calculating "human understanding" distance between elements[12].

	5.5 Inline Parsing

		Finally, with the full and correctly annotated document, we will parse the TextContent nodes for inline elements. This parsing is much simpler, as it has formal start/end tokens as has no structural elements.

		Inline parsing is done by a declarative engine that will process each element declaration.[11] For some , this is a flat transformation (i.e. it only wraps up the text into a node, as in bold or italic). Others are more involved, as in references, in which the engine will execute a callback with the text content and return a node. 
		This solves elegantly the fact that most inlines are simple and very much the same structure, while allowing for more complex ones to handle their specific needs. 


6. AST Structure, Children Elements and Containers


Notes:

	1. docs/specs/v1/grammar-core.lex
	2. docs/specs/v1/grammar-line.lex
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