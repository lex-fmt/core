//! Document-level parser responsible for parsing the entire txxt document.

use chumsky::prelude::*;
use std::ops::Range;

use super::combinators::compute_location_from_locations;
use crate::txxt::ast::{AstNode, Document};
use crate::txxt::lexers::Token;

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
    super::parser::build_document_content_parser(source).map(|content| {
        let content_locations: Vec<crate::txxt::ast::range::Range> = content
            .iter()
            .map(|item| item.range().clone())
            .collect::<Vec<_>>();
        let location = compute_location_from_locations(&content_locations);

        Document::with_content(content).with_root_location(location)
    })
}
