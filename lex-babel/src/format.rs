//! Format trait definition
//!
//! This module defines the core Format trait that all format implementations must implement.
//! The trait provides a uniform interface for parsing and serializing documents.

use crate::error::FormatError;
use lex_parser::lex::ast::Document;

/// Trait for document formats
///
/// Implementors provide bidirectional conversion between string representation and Document AST.
/// Formats can support parsing, serialization, or both.
///
/// # Examples
///
/// ```ignore
/// struct MyFormat;
///
/// impl Format for MyFormat {
///     fn name(&self) -> &str {
///         "my-format"
///     }
///
///     fn supports_parsing(&self) -> bool {
///         true
///     }
///
///     fn supports_serialization(&self) -> bool {
///         true
///     }
///
///     fn parse(&self, source: &str) -> Result<Document, FormatError> {
///         // Parse source to Document
///         todo!()
///     }
///
///     fn serialize(&self, doc: &Document) -> Result<String, FormatError> {
///         // Serialize Document to string
///         todo!()
///     }
/// }
/// ```
pub trait Format: Send + Sync {
    /// The name of this format (e.g., "lex", "markdown", "html")
    fn name(&self) -> &str;

    /// Optional description of this format
    fn description(&self) -> &str {
        ""
    }

    /// Whether this format supports parsing (source → Document)
    fn supports_parsing(&self) -> bool {
        false
    }

    /// Whether this format supports serialization (Document → source)
    fn supports_serialization(&self) -> bool {
        false
    }

    /// Parse source text into a Document
    ///
    /// Default implementation returns NotSupported error.
    /// Formats that support parsing should override this method.
    fn parse(&self, _source: &str) -> Result<Document, FormatError> {
        Err(FormatError::NotSupported(format!(
            "Format '{}' does not support parsing",
            self.name()
        )))
    }

    /// Serialize a Document into source text
    ///
    /// Default implementation returns NotSupported error.
    /// Formats that support serialization should override this method.
    fn serialize(&self, _doc: &Document) -> Result<String, FormatError> {
        Err(FormatError::NotSupported(format!(
            "Format '{}' does not support serialization",
            self.name()
        )))
    }
}
