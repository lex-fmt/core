Fullwidth Verbatim Blocks


1. Introduction
	Verbatim blocks embed non-lex content within a document, following the general indentation structure of lex, which means the block's content is +1 indented from the block's subject line.

	This is a Verbatim block:

        |<- content wall starts here
        |<- characters to the left are illegal (they are the indentation wall , not part of the content)
		def hello():
			print("hello")
    :: text ::


2. Rationale

    For some wider contents, such as tables, when the block is already deep in the document, the columns lost to the indentation wall can hinder the readability of the content. For this reason, verbatim blocks have a fullwidth mode, in which the content starts at the FULLWIDTH_INDENT, which is by default absolute 2 [1]. 

3. Conceptual Model

	From both a parsing and a code perspective we define a fullwidth block as one having the indentation wall at column absolute one, whereas the regular block (henceforth called inflow) has the wall at +1 indentation level from the block itself.

	That change aside, the internal code should behave the same. For example: 
  | Mode   | Block Start Index | Content Start Index |
  | ------ | ------------------| ------------------- |
  | Inflow 		| 4            | 8              	 |
  | Fullwidth	| 1            | 2              	 |
	:: table ::

   The example above shows a full width block with the content starting at column 2, and the block starting at column 5. Note that both the blocks start line (the subject line) and the end line (the closing annotation) are at the same indentation level as the content they are attached to.

4. Syntax 

	True to Lex's ethos, there is no explicit syntax for fullwidth blocks. The block's type is inferred from the position of the first content line character's position. Note that , for example the first line being FULLWIDTH_INDENT will make that a fullwidth block. The logic for verbatim content is that contend is indented at the wall level or greater. Full width blocks, being the same rule, behaver the same way, and as such any content starting at FULLWIDTH_INDENT +1 is valid, with the empty spaces between the wall and the first char being part of the content itself, as before.

	The opposite is not true. If the first content line is indentation +1 , no other content lines can be to the left of the wall, regardless of it being at FULLWIDTH_INDENT or not.
   
 
5. Implementation

	1. The AST / Builder

    The implementation should have a value for the block wall position, which is set according to the block mode. That is, the internal code, say for example when on strips the indentation from the content to form the VerbatimLines that code should strip from the indentation wall, and should behave the same for both modes.


	2. Scanning

	The tokenization however has to be aware of fullwidth blocks. In lexing, at later stages we will convert indentation tokens into events like indent and dedent. If not specially marked, these lines will trigger dedent events, which will break the document flow, preventing the detection of the verbatim block itself.

    Hence the tokenizer should, define $\s\s the <fullwidth> token, which will be used to mark lines that are part of a fullwidth block. Latert in parsing, say that line does not belong to a verbatim block, it should be converted back to regular spaces.

6. NOTES

	1. Full Width Indent  

		The FULLWIDTH_INDENT being the second column (col 1 for 0 zero based indexing), that is absolute one. It's easier, in this spec to use base 1 indexing, but do note that the actual implementation uses zero based indexing.

		The reason for this is that: 
			- Were it to start at col 1, there would be no way to differentiate a inner content's annotation mark like line from the actual verbatim block end line without escaping , which is antethical to the goal of lex to be a simple and forgiving language.
		- Logically then anything from column two onwards are good candadiates, and the sorter the better.
		- However, tests shows that col 2 is not a good candidate, because it's too close to the indentation wall, and people often mistake it for an error. Hence position 2 makes is more obvioius to be a deliberate choice.

