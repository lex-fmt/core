Lexpore: Testing Parsing


	The code base has a multi-layer test harness that solves a few key challenges when testing the parser. Lex is a novel format with no established references or external parsers to compare against. It's also a changing format, with updates happening frequently.

	TLDR: 
	- Use Lexplore to get the source string, parsed, and the element ready.
	- Use assert_ast to verify the AST data, not counts.

    - 
    Hence proper care must be taken to: 	

	- Ensure the tests uses correct lex strings.
    - These strings are centrally managed and vetoed.
    - On lang spec changes, it's easy to update needed strings.

	Also, from it's indentation based natures, it's often the case where element counts are a poor assertion. Not only can the same count be wrong, but often the elements will be in the wrong session or container. Hence never use element counts as many parsing results can have the same count, and often an element will be parsed to the wrong container, another point you can to verify.   

	It ensures the central repository of pre-approved Lex source strings is used, and provides a simple and powerful API to access the parsed AST. The harness used to run multiple parser implementations in parallel; today it always targets the linebased parser, but the API surface remains ready for future experiments.

    It is powered by the test library, which contains the corpora of language samples to be be used with the api on top.


Trifecta:
	The core elements: sessions, paragraphs, lists.

    Not only these are the most common and most used, but they encapsulate all central rules and trickier cases.

1. The Lex Test Corpora

	The test library resides in the docs/specs/<version> directory, currently v1:
		docs/specs/v1
			├── elements # per element tests
			│   ├── annotation # Each element has one sample per file. No progression in the files.
			│   │   ├── annotation-01-flat-marker-simple.lex # The number is how the test is accessed.
			|   |   ├── annotation-09-nested-definition-inside.lex #  Explicit flat and nested forms.
			|   |   ├── annotation-document-simple.lex # Element in larger document context
			|   |   ├── annotation-document-tricky.lex # Element in larger document context
 
			│   ├── <element>...
			└── trifecta # The core structural form tests
			│ 	├── 000-paragraphs.lex # Tests are ordered in increasing complexity.
			│   ├── 060-trifecta-nesting.lex # Files are 10 numbers apart,for easy in between additions.
			└── benchmark # The golden standard for a working parser
			│ 	├── 010-kitchensink.lex # Contains all elements in their variations.

	:: files :: 
			
			
The harness has utilities tailored for different document types. They allow you to load the document by type and number (they support opening by file name as well).

	1.1 Element Documents

		1.1.1 Isolated Elements:

			For testing parsing an isolated element, each test string should be on a stand alone file.
			The file should be in elements/<element>/<element>-<number>-<flat|nested>-<hint>.lex format.
			The number is the test number, used by the api to load the test. Everything after it is used to describe the test, with the convention of <flat|nested> to indicate the form of the test.

			The content should be the "isolated" form of the element, and the element should be the only content in the file. This avoids parsing problems with the structure and other elements confounding the test.
			The isolation is in quotes, because many elements do contain other elements. What we mean is that no siblings to the tested element are present, but if the element requires childres, they will be present. For example, there is no way to define a session without any other content.


			For nested forms, it's usually recommended to have a succession of : self recursion, one level then multiple levels then multiple elements with nesting.

		1.1.1 In Document 

			While isolated elements will allow for ast extractions and parsing, they do not test the actual semantic analysis / disambiguation. To keep a somewhat still manageable suite, we have two document types:
			- Simple: docs/specs/v1/elements/XXX-document-simple.lex
			- Tricky: docs/specs/v1/elements/XXX-document-tricky.lex

			These are template files with <insert element here> tags with suggestions of good spots where the elements could be inserted, but you're free to insert them wherever you think is best.

			Keep in mind that this is about one element, that is to say full documents with all elements is critical but it's done on other documents and tests.


	1.2 Trifecta Documents

		All the tricky structural and indentation parsing tests is concentrated between sessions, lists and paragraphs as those are the ambiguous cases that require more context to parse. Hence, if trifecta passes, all other issues are bound to be simpler cases related to a particular element's grammar.

		The trifecta corpora is available at docs/specs/v1/trifecta directory. They *must* be tested in order, that is , each file increases the complexity of the test. If you have not mastered the previous files, the next one won't work until you do.

	1.3 Benchmark Documents

		The benchmark documents are used to test the integration of the elements. They are available at docs/specs/v1/benchmarks directory. They are the golden standardt for we consider a "working-parser"  by including all elements and providing a large surface area of the spec.

            - The "kitchensink" benchmark, which ia a document that contains all elements in their main variations. This is only to be used as a blunt smoke test for regressions and full parser compliance.  The size and complexity of this document makes it a bad candidate to use when testing elements per se.
            

2. The Test Harness API

	Most parsing tests are about feeding a source string to the parser and checking the resulting AST.  The test harness API is designed to achieve this with the minimal amount of code.
	 Additionally, by encapsulating much of the low level details, it makes for less brittle suite, where changes to the Lex grammar and parser instead of fixing hundreds of tests will only require the inner library changes.

	The API has two forms depending on your needs:
	- Direct element access: get_paragraph(), get_list(), etc. return the element directly
	- Fluent pipeline: paragraph().parse(), list().tokenize() for full control over parsing/tokenization


	1. Isolated Elements

	For testing a single element, use the direct access API:
        // Gets the element directly
	 	paragraph = Lexplore.get_paragraph(1)
    :: rust ::

	This one liner will:
		- Find the element source string 1 for paragraphs in the test library.
	        - Parse it with the linebased parser.
        - Return the element directly.

    Note that this requires the document to follow the one relevant element rule to be most useful.
	This , combined with the deep AST assertion library, allows for consice, robust and deep tests:
		verbatim = Lexplore.get_verbatim(1);
         verbatim.assert_verbatim_block()
             .subject("This is the hello world example")
             .label("python");
	:: rust ::

    2. Elements in Document

		When you need the full document (for iteration, tokenization, or source access), use the fluent API:

			document = Lexplore.verbatim(8).parse()
			for verbatim in document.iter_verbatim_blocks() {
				verbatim.assert_verbatim_block();
			}
		:: rust ::

		The fluent API also provides access to source and tokens:
			source = Lexplore.paragraph(1).source()
			tokens = Lexplore.paragraph(1).tokenize()
		:: rust ::


	3. Benchmark Documents:

	    Full documents use the fluent pipeline API:
		document = Lexplore.benchmark(10).parse();
