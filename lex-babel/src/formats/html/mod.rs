//! HTML format implementation
//!
//! Strategy: Direct Lex AST → HTML generation (one-way only)
//!
//! # Data Model
//!
//! Semantic HTML generation mappings:
//!
//! | Lex Element | HTML Element | Notes |
//! |-------------|--------------|-------|
//! | Session | `<section>` with `<h1>`-`<h6>` | Heading level based on nesting depth |
//! | Paragraph | `<p>` | Direct mapping |
//! | List | `<ul>` or `<ol>` | Ordered/unordered based on list type |
//! | ListItem | `<li>` | List item |
//! | Definition | `<dl><dt><dd>` | HTML definition list |
//! | VerbatimBlock | `<pre><code class="language-X">` | Code block with language class |
//! | VerbatimLine | `<code>` | Inline code |
//! | Annotation | `<aside>` or custom data attributes | Configurable strategy |
//!
//! # Architecture
//!
//! The HTML formatter generates semantic HTML directly from the Lex AST through
//! traversal. This is a serialization-only format (no parsing from HTML to Lex).
//!
//! ## Design Decisions
//!
//! - **Semantic HTML**: Uses meaningful HTML5 elements (`<section>`, `<article>`, etc.)
//!   rather than generic `<div>` containers
//! - **Heading Levels**: Automatically calculated based on Session nesting depth
//! - **CSS Classes**: Minimal by default, with option to add custom classes
//! - **Accessibility**: Generated HTML should follow ARIA best practices
//!
//! # Implementation Notes
//!
//! - Use direct AST traversal to build HTML (no intermediate representation)
//! - Consider using a HTML builder utility from `interop::html` module
//! - Support configurable options via `HtmlOptions` struct:
//!   - Include CSS classes
//!   - Custom class prefixes
//!   - Annotation rendering strategy
//!   - Pretty-print vs minified output
//!
//! # Testing
//!
//! - Unit tests for each AST element → HTML mapping
//! - Integration tests for complete documents
//! - Snapshot tests for complex document structures
//! - Validation tests to ensure generated HTML is well-formed
//!
//! # Example
//!
//! ```ignore
//! use lex_babel::formats::html::HtmlFormat;
//! use lex_babel::Format;
//!
//! let format = HtmlFormat::default();
//! let html = format.serialize(&document)?;
//! ```
//!
//! # Future Enhancements
//!
//! - Custom CSS injection
//! - Template support for wrapper HTML
//! - MathML or MathJax support for math expressions
//! - Syntax highlighting integration for code blocks

use crate::error::FormatError;
use crate::format::Format;
use lex_parser::lex::ast::Document;

/// HTML format with configurable options
///
/// Generates semantic HTML5 from Lex documents. This is a one-way
/// serialization format only.
#[derive(Default)]
pub struct HtmlFormat {
    // options: HtmlOptions, // TODO: Add options struct
}

impl Format for HtmlFormat {
    fn name(&self) -> &str {
        "html"
    }

    fn description(&self) -> &str {
        "Semantic HTML5 output"
    }

    fn file_extensions(&self) -> &[&str] {
        &["html", "htm"]
    }

    fn supports_parsing(&self) -> bool {
        false // HTML to Lex is not supported
    }

    fn supports_serialization(&self) -> bool {
        true
    }

    fn serialize(&self, _doc: &Document) -> Result<String, FormatError> {
        // TODO: Implement HTML serialization
        // Direct AST traversal to build semantic HTML
        // Use interop::html builder utilities
        Err(FormatError::NotSupported(
            "HTML serialization not yet implemented".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_format_name() {
        let format = HtmlFormat::default();
        assert_eq!(format.name(), "html");
    }

    #[test]
    fn test_html_format_capabilities() {
        let format = HtmlFormat::default();
        assert!(!format.supports_parsing());
        assert!(format.supports_serialization());
    }

    #[test]
    fn test_html_format_extensions() {
        let format = HtmlFormat::default();
        assert_eq!(format.file_extensions(), &["html", "htm"]);
    }
}
