lex Parser Architecture

	lex as a format has some peculiarities, which influenced the parser's design.
	This document goes into some details of the architecture. 


Principles

	As an aspirational value, lex parsing should leverage battle tested parsing libraries as much as possible, as these are much better at writing parsers 
	than we are.  The end goal is to be able to define a grammar and token structure and get the parser working with 0 lines of code. 

	While we are not there yet, we've made significant progress. The previous parser impelmentation was not complete, had quite a few bugs and clocked in at 48kloc, whereas this one sits at 5kloc, both with tests. That is , the current implementation , sans tests, is about 2kloc of rust, not too bad. 

	While a reference parser ready, we aim at experimenting more aggressively on doing less and offloadin to the libraries more.

Lexing

	lex uses snows for lexing, as it's a good library, with significant usage and support and integrates well with our parsing library. 
	The only thing wortt going over is the indentation implemetation.

	Indentations significant languages are not really mainstream, and lexers rarely suppor it directly. The universal advice is to write a hand rolled lexxer that, by countins whitespaces and keeping a state machine emits the semantic indent and dedent tokes. Those are 1o1 matches with what , say braces tokens would do in conventional languages, and the idea is sound.

	However, lex does the indentation handling in two stages. First , it uses ordinarly snow tokens, with simple tabs and spaces substituitions into indentation token, that is, no state machine, and not emmited on change, just on ocurrence. Then, we run the token stream into an indentation transformation that keep tabs of the indentation tokens and replaces them with semantic events: indent and dedent.

	This approach has proven very successful. Allowing snow to do the full tokenization without code is much simpler, and we can just focus on the indentaiton transformation. By being isolated it's much easier to tests, and bugs are local to that functionality, while the core lexer works well. 


	

