//! Parser implementation for the lex format using chumsky
//!
//! This module implements a parser combinator-based parser for lex documents.
//! It builds on the token stream from the lexer and produces an AST.
//!
//! ## Testing
//!
//! All parser tests must follow strict guidelines. See the [testing module](crate::lex::testing)
//! for comprehensive documentation on using verified lex sources and AST assertions.

use chumsky::prelude::*;
use std::ops::Range;

use crate::lex::lexing::Token;
use crate::lex::parsing::ir::ParseNode;

/// Type alias for token with location
pub(crate) type TokenLocation = (Token, Range<usize>);

/// Type alias for parser error
pub(crate) type ParserError = Simple<TokenLocation>;

// Parser combinators - kept for test support if needed
#[allow(unused_imports)]
use super::builders::paragraph;

// Import parser builders from element modules
use super::annotations::build_annotation_parser;
use super::definitions::build_definition_parser;
use super::lists::build_list_parser;
use super::sessions::build_session_parser;
use super::verbatim::verbatim_block;
use std::sync::Arc;

/// Build the Multi-Parser Bundle for document-level content parsing.
///
/// This parser builds final ContentItem types directly using refactored combinators.
/// All combinators now take source parameter and return final types.
pub(crate) fn build_document_content_parser(
    source: &str,
) -> impl Parser<TokenLocation, Vec<ParseNode>, Error = ParserError> + Clone {
    let source = Arc::new(source.to_string());

    recursive(move |items| {
        let source = source.clone();
        let single_item = {
            // Session parser - now builds final Session type with location
            let session_parser = build_session_parser(source.clone(), items.clone());

            // Definition parser - now builds final Definition type with location
            let definition_parser = build_definition_parser(source.clone(), items.clone());

            // List parser - now builds final List type with location
            let list_parser = build_list_parser(source.clone(), items.clone());

            // Annotation parser - now builds final Annotation type with location
            let annotation_parser = build_annotation_parser(source.clone(), items.clone());

            choice((
                verbatim_block(source.clone()),
                annotation_parser,
                list_parser,
                definition_parser,
                session_parser,
                paragraph(source.clone()),
            ))
        };

        choice((
            filter(|(t, _)| matches!(t, Token::BlankLine(_)))
                .repeated()
                .at_least(1)
                .ignore_then(choice((
                    filter(|(t, _)| matches!(t, Token::Dedent(_)))
                        .rewind()
                        .to(vec![]),
                    items.clone(),
                ))),
            single_item
                .then(items.clone().or_not())
                .map(|(first, rest)| {
                    let mut result = vec![first];
                    if let Some(mut rest_items) = rest {
                        result.append(&mut rest_items);
                    }
                    result
                }),
            filter(|(t, _)| matches!(t, Token::Dedent(_)))
                .rewind()
                .to(vec![]),
        ))
    })
}
