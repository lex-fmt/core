Actionable Testing Guide for lex

	This document goes over some specifics of testing the lex parser.

	1. lex is a novel format, with no outside reference parsers nor document corpus. This means that you cannot create reliable lex sources, few people can. 
     2. If you can't generate the initial source string, never mind correctly imagine how that would look like in the various transformations steps, but them in the lexer or in the parser phases.
    3. The spec is still in flux, and changes are frequent. If tests created their own ad hoc in test-file lex sources, at each change we must review all test code and judge the spec changes and string by string, that is not doable.