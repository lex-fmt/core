//! Public API for the parser.

use chumsky::prelude::*;
use std::ops::Range;

use super::document::document;
use crate::lex::lexing::Token;
use crate::lex::parsing::ir::ParseNode;

/// Type alias for token with location
type TokenLocation = (Token, Range<usize>);

/// Type alias for parser error
type ParserError = Simple<TokenLocation>;

/// Parse with source text - the primary parsing function
///
/// Parses tokens with location information and source text to produce a Document.
/// All parsed documents include complete location information automatically.
pub fn parse(tokens: Vec<TokenLocation>, source: &str) -> Result<ParseNode, Vec<ParserError>> {
    document(source).parse(tokens)
}
