//! Pandoc JSON format implementation
//!
//! Strategy: Bidirectional conversion via Pandoc's JSON AST
//!
//! # Overview
//!
//! Pandoc is a universal document converter that uses a JSON representation of its
//! internal AST. This format enables Lex to integrate with Pandoc's extensive format
//! ecosystem, allowing conversion to/from formats like DOCX, PDF, EPUB, LaTeX, and more.
//!
//! # Use Case
//!
//! The typical workflow is:
//! ```text
//! lex → pandoc-json → (pandoc CLI) → docx/pdf/epub/etc.
//! ```
//!
//! This enables Lex documents to be converted to any format that Pandoc supports,
//! without implementing each format directly in Lex.
//!
//! # Data Model
//!
//! Pandoc's AST is similar to Lex but with some key differences:
//!
//! | Lex Element | Pandoc Element | Notes |
//! |-------------|----------------|-------|
//! | Session | Header + Div | Pandoc uses headers for structure, divs for grouping |
//! | Paragraph | Para | Direct mapping |
//! | List | BulletList / OrderedList | Based on list type |
//! | ListItem | List item blocks | Pandoc list items can contain block content |
//! | Definition | DefinitionList | Direct mapping to Pandoc's definition lists |
//! | VerbatimBlock | CodeBlock | With optional language attribute |
//! | VerbatimLine | Code (inline) | Inline code span |
//! | Annotation | Div with attributes | Custom attributes for metadata |
//!
//! # Architecture
//!
//! The implementation provides bidirectional conversion:
//!
//! ## Lex → Pandoc JSON (Serialization)
//! - Traverse Lex AST
//! - Map each Lex node to corresponding Pandoc AST node
//! - Serialize Pandoc AST to JSON using Pandoc's schema
//!
//! ## Pandoc JSON → Lex (Parsing)
//! - Parse Pandoc JSON to Pandoc AST
//! - Map Pandoc nodes to Lex nodes
//! - Handle structural differences (e.g., flat headers → nested sessions)
//! - Build Lex Document AST
//!
//! # Implementation Notes
//!
//! - Use the `pandoc_types` crate for Pandoc AST types
//! - Implement mapping logic in `interop::pandoc` module
//! - Handle version compatibility with Pandoc's API version
//! - Preserve metadata and attributes where possible
//!
//! ## Challenges
//!
//! 1. **Structural Mismatch**: Pandoc uses flat headers while Lex has nested sessions
//!    - Solution: Track header levels and reconstruct nesting during parsing
//!
//! 2. **Metadata Preservation**: Pandoc has rich metadata support
//!    - Solution: Map to Lex annotations where possible
//!
//! 3. **Inline Formatting**: Pandoc has extensive inline formatting
//!    - Solution: For now, preserve as plain text; future versions may support TextContent metadata
//!
//! # Testing
//!
//! - Round-trip tests: Lex → Pandoc → Lex should be lossless where possible
//! - Compatibility tests with various Pandoc versions
//! - Integration tests with actual Pandoc CLI for common workflows
//! - Snapshot tests for complex document structures
//!
//! # Example
//!
//! ```ignore
//! use lex_babel::formats::pandoc::PandocJsonFormat;
//! use lex_babel::Format;
//!
//! let format = PandocJsonFormat;
//!
//! // Lex → Pandoc JSON
//! let pandoc_json = format.serialize(&lex_doc)?;
//!
//! // Pandoc JSON → Lex
//! let lex_doc = format.parse(&pandoc_json)?;
//! ```
//!
//! # External Dependencies
//!
//! To actually convert to other formats, users need:
//! - Pandoc CLI installed (`brew install pandoc`, `apt-get install pandoc`, etc.)
//! - Example workflow:
//!   ```bash
//!   lex convert doc.lex --to pandoc-json > doc.json
//!   pandoc doc.json -f json -t docx -o doc.docx
//!   ```
//!
//! # Future Enhancements
//!
//! - Direct Pandoc CLI integration (spawn process internally)
//! - Support for Pandoc's filter system
//! - Citation and bibliography support
//! - Math expression handling (MathML, LaTeX math)

use crate::error::FormatError;
use crate::format::Format;
use lex_parser::lex::ast::Document;

/// Pandoc JSON format for bidirectional conversion
///
/// Enables integration with Pandoc's extensive format ecosystem.
pub struct PandocJsonFormat;

impl Format for PandocJsonFormat {
    fn name(&self) -> &str {
        "pandoc-json"
    }

    fn description(&self) -> &str {
        "Pandoc JSON AST format for universal document conversion"
    }

    fn file_extensions(&self) -> &[&str] {
        &["pandoc.json", "json"]
    }

    fn supports_parsing(&self) -> bool {
        true
    }

    fn supports_serialization(&self) -> bool {
        true
    }

    fn parse(&self, _source: &str) -> Result<Document, FormatError> {
        // TODO: Implement Pandoc JSON parsing
        // Parse Pandoc JSON → Lex AST via interop::pandoc
        Err(FormatError::NotSupported(
            "Pandoc JSON parsing not yet implemented".to_string(),
        ))
    }

    fn serialize(&self, _doc: &Document) -> Result<String, FormatError> {
        // TODO: Implement Pandoc JSON serialization
        // Lex AST → Pandoc JSON via interop::pandoc
        Err(FormatError::NotSupported(
            "Pandoc JSON serialization not yet implemented".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pandoc_format_name() {
        let format = PandocJsonFormat;
        assert_eq!(format.name(), "pandoc-json");
    }

    #[test]
    fn test_pandoc_format_capabilities() {
        let format = PandocJsonFormat;
        assert!(format.supports_parsing());
        assert!(format.supports_serialization());
    }

    #[test]
    fn test_pandoc_format_extensions() {
        let format = PandocJsonFormat;
        assert!(format.file_extensions().contains(&"json"));
        assert!(format.file_extensions().contains(&"pandoc.json"));
    }
}
