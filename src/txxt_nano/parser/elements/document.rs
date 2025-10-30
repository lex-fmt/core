//! Document-level parser responsible for parsing the entire txxt document.

use chumsky::prelude::*;
use std::ops::Range;

use crate::txxt_nano::ast::Document;
use crate::txxt_nano::lexer::Token;

/// Type alias for token with location
type TokenLocation = (Token, Range<usize>);

/// Type alias for parser error
type ParserError = Simple<TokenLocation>;

/// Parse a document
///
/// Parses the entire token stream as document content.
/// This function is focused on document-level parsing and delegates to parser.rs
/// for the actual document content parsing logic.
pub fn document(source: &str) -> impl Parser<TokenLocation, Document, Error = ParserError> + Clone {
    crate::txxt_nano::parser::parser::build_document_content_parser(source).map(|content| {
        Document {
            metadata: Vec::new(),
            content,
            location: None,
        }
    })
}
