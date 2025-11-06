//! Document-level parser responsible for parsing the entire lex document.

use chumsky::prelude::*;
use std::ops::Range;

use crate::lex::lexers::Token;
use crate::lex::parsers::ir::{NodeType, ParseNode};

/// Type alias for token with location
type TokenLocation = (Token, Range<usize>);

/// Type alias for parser error
type ParserError = Simple<TokenLocation>;

/// Parse a document
///
/// Parses the entire token stream as document content.
/// This function is focused on document-level parsing and delegates to parser.rs
/// for the actual document content parsing logic.
pub fn document(source: &str) -> impl Parser<TokenLocation, ParseNode, Error = ParserError> + Clone {
    super::parser::build_document_content_parser(source).map(|children| {
        ParseNode::new(NodeType::Document, vec![], children)
    })
}
