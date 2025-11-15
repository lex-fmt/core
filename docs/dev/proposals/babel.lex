





	6.3. HTML Format

		Strategy: Direct Lex AST � HTML generation (one-way only)

		Implementation:
			pub struct HtmlFormat {
			    options: HtmlOptions,
			}

			impl Format for HtmlFormat {
			    fn name(&self) -> &str { "html" }
			    fn supports_parsing(&self) -> bool { false }
			    fn supports_serialization(&self) -> bool { true }

			    fn serialize(&self, doc: &Document) -> Result<String, FormatError> {
			        // Direct AST traversal to build semantic HTML
			        // Use interop::html builder utilities
			    }
			}
		:: rust ::

	6.4. Pandoc JSON Format

		Strategy: Bidirectional conversion via Pandoc's JSON AST

		Implementation:
			pub struct PandocJsonFormat;

			impl Format for PandocJsonFormat {
			    fn name(&self) -> &str { "pandoc-json" }
			    fn supports_parsing(&self) -> bool { true }
			    fn supports_serialization(&self) -> bool { true }

			    fn parse(&self, source: &str) -> Result<Document, FormatError> {
			        // Parse Pandoc JSON � Lex AST via interop::pandoc
			    }

			    fn serialize(&self, doc: &Document) -> Result<String, FormatError> {
			        // Lex AST � Pandoc JSON via interop::pandoc
			    }
			}
		:: rust ::

		Use case: Enables lex � pandoc json � (pandoc CLI) � docx/pdf/epub/etc.

7. Semantic Mappings

	7.1. Lex � Markdown

		-   Session � Markdown heading (# level based on nesting depth)
		-   Paragraph � Paragraph
		-   List/ListItem � Markdown list (-, *, 1.)
		-   Definition � Multiple strategies (configurable):
		    -   Strategy 1: **Term**: Description (bold + colon)
		    -   Strategy 2: ### Term\n\nDescription (heading)
		    -   Strategy 3: &lt;dl&gt;&lt;dt&gt;Term&lt;/dt&gt;&lt;dd&gt;Description&lt;/dd&gt;&lt;/dl&gt; (HTML in markdown)
		-   VerbatimBlock � Code block (```language\ncode\n```)
		-   VerbatimLine � Inline code (`code`)
		-   Annotation � Markdown comment or YAML frontmatter (configurable)

	7.2. Markdown � Lex

		Reverse mappings:
			-   Heading � Session (nesting based on heading level)
			-   Paragraph � Paragraph
			-   List � List/ListItem
			-   Code block � VerbatimBlock
			-   Inline code � VerbatimLine
			-   Bold/emphasis � TextContent with metadata (if supported in future)

	7.3. Lex � HTML

		Semantic HTML generation:
			-   Session � &lt;section&gt; with &lt;h1-h6&gt;
			-   Paragraph � &lt;p&gt;
			-   List � &lt;ul&gt; or &lt;ol&gt;, ListItem � &lt;li&gt;
			-   Definition � &lt;dl&gt;&lt;dt&gt;&lt;dd&gt;
			-   VerbatimBlock � &lt;pre&gt;&lt;code class="language-X"&gt;
			-   Annotation � &lt;aside&gt; or custom data attributes


9. Implementation Phases

	Phase 1: Foundation
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

	Phase 4: HTML Support
		-   Implement interop::html (Lex � HTML builders)
		-   Implement HtmlFormat (serialization only)
		-   Add lex convert --from lex --to html

	Phase 5: Pandoc Support
		-   Implement interop::pandoc (Lex � Pandoc JSON AST)
		-   Implement PandocJsonFormat (bidirectional)
		-   Document workflow: lex � pandoc-json � (pandoc CLI) � other formats
