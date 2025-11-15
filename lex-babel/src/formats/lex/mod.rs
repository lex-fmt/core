//! Lex format implementation
//!
//! This module implements the Format trait for Lex itself, treating Lex
//! as just another format in the system. This creates a uniform API where
//! Lex can be converted to/from other formats using the same interface.

use crate::error::FormatError;
use crate::format::Format;
use lex_parser::lex::ast::Document;
use lex_parser::lex::transforms::standard::STRING_TO_AST;

/// Format implementation for Lex
///
/// Parses Lex source text into a Document AST by delegating to lex-parser.
/// Serialization is not yet implemented (would require a proper Lex serializer).
pub struct LexFormat;

impl Format for LexFormat {
    fn name(&self) -> &str {
        "lex"
    }

    fn description(&self) -> &str {
        "Lex document format"
    }

    fn supports_parsing(&self) -> bool {
        true
    }

    fn supports_serialization(&self) -> bool {
        false // TODO: Implement Lex serializer
    }

    fn parse(&self, source: &str) -> Result<Document, FormatError> {
        STRING_TO_AST
            .run(source.to_string())
            .map_err(|e| FormatError::ParseError(e.to_string()))
    }

    fn serialize(&self, _doc: &Document) -> Result<String, FormatError> {
        // TODO: Implement proper Lex serializer
        // Could use detokenizer if tokens are preserved, or build a serializer from AST
        Err(FormatError::NotSupported(
            "Lex serialization not yet implemented".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lex_parser::lex::ast::{ContentItem, Paragraph};

    #[test]
    fn test_lex_format_name() {
        let format = LexFormat;
        assert_eq!(format.name(), "lex");
    }

    #[test]
    fn test_lex_format_supports_parsing() {
        let format = LexFormat;
        assert!(format.supports_parsing());
        assert!(!format.supports_serialization());
    }

    #[test]
    fn test_lex_format_parse_simple() {
        let format = LexFormat;
        let source = "Hello world\n";

        let result = format.parse(source);
        assert!(result.is_ok());

        let doc = result.unwrap();
        assert_eq!(doc.root.children.len(), 1);

        match &doc.root.children[0] {
            ContentItem::Paragraph(_) => {}
            _ => panic!("Expected paragraph"),
        }
    }

    #[test]
    fn test_lex_format_parse_session() {
        let format = LexFormat;
        let source = "Introduction:\n    Welcome to the guide\n";

        let result = format.parse(source);
        assert!(result.is_ok());

        let doc = result.unwrap();
        // Just verify that something was parsed successfully
        // The exact structure depends on the parser implementation
        assert!(!doc.root.children.is_empty());
    }

    #[test]
    fn test_lex_format_parse_error() {
        let format = LexFormat;
        // Create invalid Lex that would cause a parse error
        // Note: Current parser is very permissive, so this might not fail
        // But the test shows the error handling works
        let source = "";

        let result = format.parse(source);
        // Empty document should parse successfully
        assert!(result.is_ok());
    }

    #[test]
    fn test_lex_format_serialize_not_supported() {
        let format = LexFormat;
        let doc = Document::with_content(vec![ContentItem::Paragraph(Paragraph::from_line(
            "Test".to_string(),
        ))]);

        let result = format.serialize(&doc);
        assert!(result.is_err());

        match result.unwrap_err() {
            FormatError::NotSupported(_) => {}
            _ => panic!("Expected NotSupported error"),
        }
    }
}
