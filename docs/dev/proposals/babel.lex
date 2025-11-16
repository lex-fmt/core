Lex Babel

	is the interop library for lex, that can covert to and from various formats, such as markdown, html and pandoc.

	As the implementation is midway, we've moved the initial proposal into actual comments as in lex-babel/src/formats/markdown/mod.rs  or lex-babel/src/lib.rs

what here remains are the implementation phases: 


Implementation Phases


	Phase 1: Foundation DONE
		-   Create lex-babel crate structure
		-   Define Format trait and FormatRegistry
		-   Move existing formatters (tag, treeviz) from lex-parser to lex-babel
		-   Update lex-cli to use lex-babel for tag/treeviz

	Phase 2: Lex Format
		-   Implement LexFormat (parser delegation, serializer)
		-   Ensure round-trip capability for Lex documents

	Phase 3: Markdown Support
		-   Add comrak dependency to lex-babel
		-   Implement interop::markdown (Lex � comrak AST)
		-   Implement MarkdownFormat (bidirectional)
		-   Add lex convert --from markdown --to lex
		-   Add lex convert --from lex --to markdown

	from there on it's all abouting adding additional format, being html then pandoc the next priorities.	