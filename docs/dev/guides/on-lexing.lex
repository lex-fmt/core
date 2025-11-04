Lexing

	Tokenization is handled by logos, the lexer generator library. The base_tokenization.rs  will use it to generate the core Lex tokens defined in tokens_core.rs.

	Base Tokenization:
		Tokenizing a string source of Lex into tokens.
	:: file src=src/lex/lexers/tokens_core.rs :: 
	
1. Parsing Leverage: Higher Level Tokens

	Lex is an unconventional language which, from a general perspecitve, makes it parsing challenge: revcursive, statefull (in parts) and indentation significant. While this is a fact, the format has been design with specific contrainst that, leveraged will simplify parsing enormously. 

	In order to do so, we do need to group tokens into higher orders ones: first to lines , then levels. 

2. Pipeline & Transformations:
	
	In order to foster easy experimentation between shifiting complexity between the parser and the lexer, Lex has a extendable and declarative pipeline manager that can chain transformations.

	The pipeline is a declarative sequence of transformations to be applied to a TokenStream of core tokens.
	All pipelines will start with the core tokenations, convert them to a TokenStream and then apply it's transformations in order, feeding one's TokenStream output as the input to the next until all transoformations have been applied and generated the final TokenStream.

	Transformations are designed to be chained into pipelines. Here is what the full lexing looks like: 

	Lexer:
		Pipeline -> Core Tokens -> Token Stream -> [ Transformation N ...] -> TokenStream
	Parsing:
		Token Stream -> Parsing -> Core Tokens -> AST Building


	Transformations receive a TokenStream and will return one on output. This is the contract that enables us to keep decouples but composable transformations.

3. Token Streams


	The common data structure that allows different transformations to take place is the Token Stream [src/lex/pipeline/stream.rs]. From a single vector to a full tree of tokens, the format offers standard interfaces for various transformations. See the full documentation [docs/dev/guides/on-tokenstreams.lex] for them .