Lexing

	The initial lexing is done by logos, the lexer generator library. The base_tokenization.rs  will use it to generate the core Lex tokens defined in tokens.rs.

	Base Tokenization:
		Generates all Lex tokens as definied in tokens.rs.
	
	It's a key part of the design to leverage the logos lib as much as possible. 
	Once the core tokens are available we run a series of transformations, that is functions that receive the token stream , do their thing and return the modified token stream. 

	Transoformations are designed to be chained into pipelines. Here is what the full lexing looks like: 

		base tokens -> pipeline -> transformation A -> transformation B

	If a token can be deteced by very simple rules it should be on the logos base tokeniztion, as leveraging a battle tested, performant and capable solution and being execused of having to write more code. 

1. The Architecture
 
	Transformations

		There are however situations in which we want to further process the token stream and choose not to do it in the base tokenization. Typically these are either about a more complex transform, for example a stateful one with a state machine, or ones that create highe level semantics on top of previous tokens. 

		This design keeps the risker parts (statefull transformation) isolated , where it's easy to test and to reason about. And it makes the semantic transforms much easier as they operate not on characters , but on tokens. 

	Pipelines

		Pipelines combine the initial tokenization then a sequence of transformations

	Indentation/Whitespace

		Lex is an indentation significant language, and there is no standar tokenization support for these. Sure, the lexer can parse indentations per se, and logos does just that it will replace either tab-widths spaces or a tab with an indentation token.

		This is a very straight forward token. For example , on a line with 8 space: 
			........println("hello"):
		This becomes: 
			<indentation><indentation>println("hello")
		:: text

		But an indentation token alone is too low level for parsing, we want to know when levels change, not how many spaces there are. The standard solution is to have a statefull run over the raw tokens that replaces them with the actual start/end of blocks we're interested, 'indent' and 'dedent'. In this representation

	Line

		In this transformation we group tokens by line, hinting on what kind of element that line looks like. This is used in parser linebased. Since Lex is fundamentatyy a lined base grammar, having tokens groupped as such allows us to parse it really easier (in said parser the grammar is a list of regexes using the line based tokens)


	Line Container

		As described, the linebased parser is a simple regex. Hence it can't do recursion correctly, as regexed do not count. Since parsing an element does not require matching against it's children tokens , with a token tree we can parse each level (the level's children, the LineContainer ) is one expression that the regex uses.). The parser will match a linecontainer (a level) , then descent , unrolling a line container into it's consituent Line parts.

2. Tokens, Aggregation , Transformations and Parsing.

	Some transformations are flat, namely the oned in the indentation pipeline. 
	
	1. Aggregation
	 
		The linebased pipeline however composes tokens. In the first step it groups tokens by lines, in Line enumbs (essenctially a vector of tokens). Lex is basically a line defined language: you can write the grammar only describeing a small number of line forms. In fact this is the easiest way to parse Lex.

		This is what the linebased parser does. It uses a list of regular regexes as the grammaer and matches them against the lines. For each level, it will only match line tokens names, and from there it perfectly parse a level, simply.

		Since lex is recursive, the next transformation, LineContainer will generate a tree of Line tokens that reprepresnt the document, Since the grammar does not look inside the container, it can parse a level by only that levels lines , it does so, parsing a level, then unrolling it's containers and recursively parsing that.


	2. Parsing

		This means that parser will operate on flat tokens or LineContainers , that is, a tree of Line tokens , which, in turn , are a vector of regular tokens.	

		It would be very wasteful to have ast builders for each token type. Instead, we have common utitiles that can unroll lines to tokens. As mentioned, it's pretty simple as lines store the actual tokens in a vector. See oken_processing.rs . 