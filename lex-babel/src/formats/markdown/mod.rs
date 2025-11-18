//! Markdown format implementation
//!
//! This module implements bidirectional conversion between Lex and CommonMark Markdown.
//!
//! # Library Choice
//!
//! We use the `comrak` crate for Markdown parsing and serialization. This choice is based on:
//! - Single crate for both parsing and serialization
//! - Feature-rich with CommonMark compliance
//! - Robust and well-maintained
//! - Supports extensions (tables, strikethrough, etc.)
//!
//! # Element Mapping Table
//!
//! Complete Lex ↔ Markdown Mapping:
//!
//! | Lex Element      | Markdown Equivalent     | Export Notes                           | Import Notes                          |
//! |------------------|-------------------------|----------------------------------------|---------------------------------------|
//! | Session          | Heading (# ## ###)      | Session level → heading level (1-6)    | Heading level → session nesting       |
//! | Paragraph        | Paragraph               | Direct mapping                         | Direct mapping                        |
//! | List             | Unordered list (- *)    | Direct mapping                         | Both ordered/unordered → Lex list    |
//! | ListItem         | List item (- item)      | Direct mapping with nesting            | Direct mapping with nesting           |
//! | Definition       | **Term**: Description   | Bold term + colon + content            | Parse bold + colon pattern            |
//! | Verbatim         | Code block (```)        | Language → info string                 | Info string → language                |
//! | Annotation       | HTML comment            | `<!-- lex:label key=val -->` format    | Parse lex: prefixed HTML comments     |
//! | InlineContent:   |                         |                                        |                                       |
//! |   Text           | Plain text              | Direct                                 | Direct                                |
//! |   Bold           | **bold** or __bold__    | Use **                                 | Parse both                            |
//! |   Italic         | *italic* or _italic_    | Use *                                  | Parse both                            |
//! |   Code           | `code`                  | Direct                                 | Direct                                |
//! |   Math           | $math$ or $$math$$      | Use $...$                              | Parse if extension enabled            |
//! |   Reference      | [text](url)             | Convert to markdown link               | Parse link/reference syntax           |
//!
//! # Lossy Conversions
//!
//! The following conversions lose information on round-trip:
//! - Lex sessions beyond level 6 → h6 with nested content (Markdown max is h6)
//! - Lex annotations → HTML comments (may be stripped by some parsers)
//! - Lex definition structure → bold text pattern (not native Markdown)
//! - Multiple blank lines → single blank line (Markdown normalization)
//!
//! # Architecture Notes
//!
//! There is a fundamental mismatch between Markdown's flat model and Lex's hierarchical structure.
//! We leverage the IR event system (lex-babel/src/mappings/) to handle the nested-to-flat and
//! flat-to-nested conversions. This keeps format-specific code focused on Markdown AST transformations.
//!
//! Lists are the only Markdown element that are truly nested, making them straightforward to map.
//!
//! # Testing
//!
//! Export tests use Lex spec files from docs/specs/v1/elements/ for isolated element testing.
//! Integration tests use the kitchensink benchmark and a CommonMark reference document.
//! See the testing guide in docs/local/tasks/86-babel-markdown.lex for details.
//!
//! # Implementation Status
//!
//! - [x] Export (Lex → Markdown)
//!   - [x] Paragraph
//!   - [x] Heading (Session) - nested sessions → flat heading hierarchy
//!   - [x] Bold, Italic, Code inlines
//!   - [x] Lists - auto-wraps inline content in paragraphs
//!   - [x] Code blocks (Verbatim)
//!   - [x] Definitions - term paragraph + description siblings
//!   - [x] Annotations - as HTML comments
//!   - [x] Math - rendered as $...$ text
//!   - [x] References - rendered as links
//! - [ ] Import (Markdown → Lex) - TODO
//!   - [ ] Paragraph
//!   - [ ] Heading → Session
//!   - [ ] Bold, Italic, Code inlines
//!   - [ ] Lists
//!   - [ ] Code blocks → Verbatim
//!   - [ ] Definitions (pattern matching)
//!   - [ ] Annotations (HTML comment parsing)

mod serializer;
// parser module will be implemented after export is fully tested

use crate::error::FormatError;
use crate::format::Format;
use lex_parser::lex::ast::Document;

/// Format implementation for Markdown
pub struct MarkdownFormat;

impl Format for MarkdownFormat {
    fn name(&self) -> &str {
        "markdown"
    }

    fn description(&self) -> &str {
        "CommonMark Markdown format"
    }

    fn file_extensions(&self) -> &[&str] {
        &["md", "markdown"]
    }

    fn supports_parsing(&self) -> bool {
        false // TODO: Implement parser
    }

    fn supports_serialization(&self) -> bool {
        true
    }

    fn parse(&self, _source: &str) -> Result<Document, FormatError> {
        Err(FormatError::ParseError(
            "Parser not yet implemented".to_string(),
        ))
        //parser::parse_from_markdown(source)
    }

    fn serialize(&self, doc: &Document) -> Result<String, FormatError> {
        serializer::serialize_to_markdown(doc)
    }
}
