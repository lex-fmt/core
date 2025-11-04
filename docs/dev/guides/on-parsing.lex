Parsing Lex Strings

	This document contains a high level overview of the parsing architecture of Lex.

	Parsers will receive a TokenStream[1] with the core tokens[2], and use adapters if it needs the token structured to be different as in the linebased parser does[3].  And after parsing return the AST tree.

	The final parsing step, building the AST has token stream support (and the adapters for it)
	



Notes: 

 2. The token stream guide:  docs/dev/guides/on-tokenstreams.lex
 1. The core tokens: src/lex/lexers/tokens_core.rs and it's grammar docs/specs/v1/grammar.lex
 3. Adapters: src/lex/pipeline/adapters_linebased.rs and src/lex/pipeline/adapters.rs