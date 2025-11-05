Whitespace Handling in Lex

	Lex, being a indentation significant format has very specific whitespace handling , which this document covers. 


1. Blank lines

	Blank lines are lines of text where only whitespace characters appears before the new like char, regardless of bing spaces, tabs or others.

	Semantically a  empty blankline (i.e. "\n") is equivalent to others "   	\n", in which it is considered and treated like a blank line. Of course tokenization presevers the whitespaces as these affect location tracking and round trip checks, but semantically the are the same.

3. Indentation

	Indentation is defined in spaces, the value for one level being 4. Tabs are converted to spaces by mutliplying tabs by tab width. So replace tabs , multiplying by 4. 

	Then each 4 tabs become an indentation token. Any remainder is kept as spaces and is considered part of the actual line content.

	However indentation tokens are not very useful, since mid-parsing keeping the state machine controlling them is far from ideal. A lexer transformation will implement the state machine and emit only events like indention increased -> indent, and indentation decrease (dedent). These are the very same thing as an open bracket or close bracket in c-style languages, and parsing tooling is good at handling them.


	Some specific parsers (like the line parser) will further process this, nesting indenting blocks, since that particular engine is regex base, and regular languages cannot count and balance tokens.

3. On Blank Lines and Indentation

	The first corollary to this is that indentation levels is only defined in nob blank lines. That is to say, the number of whitespaces chars in a blank line has no bearing on it. The first non whitespace character's position is what determine the indentation chars, hence it's levels.

4. On a Line's Text Content

	The content is defined as all characters after indentation tokens, even whitespaces (wince these should, in theory, be remainers of the indentation counting.

	Consider: 
		raw string "....hello"
		first pass :"<indentation>hello"
		second pass (if indentation level just increased): "<indent>hello"

	Now consider a string that has non integer indentation stops: 
		raw string "......hello"
		first pass :"<indentation>..hello"
		second pass (if indentation level just increased): "<indent>..hello"


5. Foreign Blocks:

	The above mentioned rules apply all elements but Foreign blocks. Lets see why. 

	5.1 The Indentation Wall

		This is a foreign block: 

			def hello():
				print ("hello")

		:: python ::

		Bellow, I'll replace spaces for dots so the point is easier to make.  The foreing block raw lines are: 
			"\n" 
			"....def hello():
			"........print("hello);
			"\n" 
		:: text ::

		While this is their raw string, it's not it's content (that would not be a valid python program).  The block starts at column 0, since it's a the document's top level. This means that indentation +1 is column 3 (1 level up). 
		Anything to the left of that is illegal (such string would terminate the block, which needs to terminate on a annotation line.)

		So the block sets a wall, o column number, where the content actually starts. In this case the content is: 
			"\n"
			def hello():
			....print("hello");
			"\n"
		:: test ::

		To the right of the wall anything character, including whitespace chars are part of the content

		To understand why consider this scenario. 

		This is a Session 

			And here is the sessions's content. The very same Foreign block is shown bellow.
			This is a foreign block: 

				def hello():
					print ("hello")

			:: python ::

		Now, since the block is in a deeper document session, instead of starting at column 3, it starts at column 7.
		But it's content is exactely the same, because we strip the indentation wll of the container block. Anything after that is the content for the block, including the leading spaces on the 3rd line (print).


	5.2 Post Indentation wall

		As we've shown , any number of spaces after the indentation wall is valid for a foreing block. That conetnt can be prefixed, by say 20 spaces as a grammar rule, and we can't manipulate it. 

		But there is a catch here: by the time we actually get to foreign block parsing, we are looking at tokens. Hence all spaces are converted to indent/dedent/indentation tokes, because the tokenizer does not parse elements and it can't know this is a line holding foreign block content: it transforms whitespaces to indentation tokens for any line. 

		To cater for this, when parsing a foreign block, one must subtract the indentation wall of the block, and if present, subsitute indentation or indent tokens for spaces. After that, after the first non whitestpace char, the content is to be taken as is, no further processing.

		That is to say, while we cannot process a foreign block line's content, we must split it from the txxt indentation wall, but otherwise keep them untouched.
