//! AST Node Builders for the Reference Parser
//!
//! This module consolidates all AST node building logic from the reference parser.
//! It contains:
//! - Location utilities (conversion from byte ranges to line/column positions)
//! - Text extraction helpers
//! - Builder functions for creating AST nodes
//! - All tests related to AST building
//!
//! This centralizes the duplication previously spread across multiple modules
//! (annotations.rs, definitions.rs, sessions.rs, lists.rs, foreign.rs, combinators.rs, labels.rs, parameters.rs)

use chumsky::prelude::*;
use chumsky::primitive::filter;
use std::ops::Range as ByteRange;
use std::sync::Arc;

use crate::lex::lexing::Token;
use crate::lex::parsing::ir::{NodeType, ParseNode};

/// Type alias for token with location
pub(crate) type TokenLocation = (Token, ByteRange<usize>);

/// Type alias for parser error
pub(crate) type ParserError = Simple<TokenLocation>;

// ============================================================================
// TOKEN PROCESSING UTILITIES
// ============================================================================

// ============================================================================
// LOCATION UTILITIES
// ============================================================================
//
// Location utilities are now provided by crate::lex::parsing::ast::location
// See that module for byte_range_to_location, compute_location_from_locations, etc.

/// Check if a token is a text-like token (content that can appear in lines)
///
/// This includes: Text, Whitespace, Numbers, Punctuation, and common symbols
pub(crate) fn is_text_token(token: &Token) -> bool {
    matches!(
        token,
        Token::Text(_)
            | Token::Whitespace
            | Token::Number(_)
            | Token::Dash
            | Token::Period
            | Token::OpenParen
            | Token::CloseParen
            | Token::Colon
            | Token::Comma
            | Token::Quote
            | Token::Equals
    )
}

// ============================================================================
// PARSER COMBINATORS
// ============================================================================

/// Helper: match a specific token type, ignoring the location
pub(crate) fn token(t: Token) -> impl Parser<TokenLocation, (), Error = ParserError> + Clone {
    filter(move |(tok, _)| tok == &t).ignored()
}

/// Parse a text line (sequence of text and whitespace tokens)
/// Returns the collected tokens (preserving both token and location info)
pub(crate) fn text_line(
) -> impl Parser<TokenLocation, Vec<TokenLocation>, Error = ParserError> + Clone {
    filter(|(t, _location): &TokenLocation| is_text_token(t))
        .repeated()
        .at_least(1)
    // No .map() - preserve tokens!
}

/// Parse a paragraph
pub(crate) fn paragraph(
    _source: Arc<String>,
) -> impl Parser<TokenLocation, ParseNode, Error = ParserError> + Clone {
    let line_with_newline = text_line().then(filter(|(tok, _)| tok == &Token::Newline));

    line_with_newline.repeated().at_least(1).map(
        move |lines: Vec<(Vec<TokenLocation>, TokenLocation)>| {
            let mut tokens = vec![];
            for (line_tokens, newline_token) in lines {
                tokens.extend(line_tokens);
                tokens.push(newline_token);
            }
            ParseNode::new(NodeType::Paragraph, tokens, vec![])
        },
    )
}

// ============================================================================
// ANNOTATION BUILDING
// ============================================================================

/// Parse the tokens between :: markers (for annotation headers).
///
/// This just collects the tokens - label and parameter parsing is done
/// by the universal pipeline in data_extraction.
pub(crate) fn annotation_header(
) -> impl Parser<TokenLocation, Vec<TokenLocation>, Error = ParserError> + Clone {
    filter(|(t, _): &TokenLocation| !matches!(t, Token::LexMarker | Token::Newline)).repeated()
}

/// Build an annotation parser
pub(crate) fn build_annotation_parser<P>(
    _source: Arc<String>,
    items: P,
) -> impl Parser<TokenLocation, ParseNode, Error = ParserError> + Clone
where
    P: Parser<TokenLocation, Vec<ParseNode>, Error = ParserError> + Clone + 'static,
{
    let header = token(Token::LexMarker)
        .ignore_then(annotation_header())
        .then_ignore(token(Token::LexMarker));

    let block_form = {
        let header_for_block = header.clone();
        header_for_block
            .then_ignore(token(Token::Newline))
            .then(
                filter(|(t, _)| matches!(t, Token::Indent(_)))
                    .ignore_then(items.clone())
                    .then_ignore(filter(|(t, _)| matches!(t, Token::Dedent(_))))
                    .or_not(),
            )
            .then_ignore(token(Token::LexMarker))
            .then_ignore(token(Token::Newline).or_not())
            .map(move |(header_tokens, children)| {
                ParseNode::new(
                    NodeType::Annotation,
                    header_tokens,
                    children.unwrap_or_default(),
                )
            })
    };

    let single_line_or_marker = {
        let header_for_single = header.clone();
        header_for_single
            .then(token(Token::Whitespace).ignore_then(text_line()).or_not())
            .then_ignore(token(Token::Newline).or_not())
            .map(move |(header_tokens, content_tokens)| {
                let children = if let Some(tokens) = content_tokens {
                    let para_node = ParseNode::new(NodeType::Paragraph, tokens, vec![]);
                    vec![para_node]
                } else {
                    vec![]
                };

                ParseNode::new(NodeType::Annotation, header_tokens, children)
            })
    };

    block_form.or(single_line_or_marker)
}

// ============================================================================
// DEFINITION BUILDING
// ============================================================================

/// Parse a definition subject
/// Returns tokens (not pre-extracted text) for universal pipeline
pub(crate) fn definition_subject(
) -> impl Parser<TokenLocation, Vec<TokenLocation>, Error = ParserError> + Clone {
    filter(|(t, _location): &TokenLocation| !matches!(t, Token::Colon | Token::Newline))
        .repeated()
        .at_least(1)
        .then_ignore(filter(|(t, _): &TokenLocation| matches!(t, Token::Colon)).ignored())
        .then_ignore(filter(|(t, _): &TokenLocation| matches!(t, Token::Newline)).ignored())
    // No .map() - preserve tokens!
}

/// Build a definition parser
pub(crate) fn build_definition_parser<P>(
    _source: Arc<String>,
    items: P,
) -> impl Parser<TokenLocation, ParseNode, Error = ParserError> + Clone
where
    P: Parser<TokenLocation, Vec<ParseNode>, Error = ParserError> + Clone + 'static,
{
    definition_subject()
        .then(
            filter(|(t, _)| matches!(t, Token::Indent(_)))
                .ignore_then(items)
                .then_ignore(filter(|(t, _)| matches!(t, Token::Dedent(_)))),
        )
        .map(move |(subject_tokens, children)| {
            ParseNode::new(NodeType::Definition, subject_tokens, children)
        })
}

// ============================================================================
// SESSION BUILDING
// ============================================================================

/// Parse a session title
/// Returns tokens (not pre-extracted text) for universal pipeline
pub(crate) fn session_title(
) -> impl Parser<TokenLocation, Vec<TokenLocation>, Error = ParserError> + Clone {
    text_line()
        .then_ignore(token(Token::Newline))
        .then_ignore(filter(|(t, _)| matches!(t, Token::BlankLine(_))))
    // No .map() - preserve tokens!
}

/// Build a session parser
pub(crate) fn build_session_parser<P>(
    _source: Arc<String>,
    items: P,
) -> impl Parser<TokenLocation, ParseNode, Error = ParserError> + Clone
where
    P: Parser<TokenLocation, Vec<ParseNode>, Error = ParserError> + Clone + 'static,
{
    session_title()
        .then(
            filter(|(t, _)| matches!(t, Token::Indent(_)))
                .ignore_then(items)
                .then_ignore(filter(|(t, _)| matches!(t, Token::Dedent(_)))),
        )
        .map(move |(title_tokens, children)| {
            ParseNode::new(NodeType::Session, title_tokens, children)
        })
}

// ============================================================================
// LIST BUILDING
// ============================================================================

/// Parse a list item line - a line that starts with a list marker
/// Returns tokens (not pre-extracted text) for universal pipeline
pub(crate) fn list_item_line(
) -> impl Parser<TokenLocation, Vec<TokenLocation>, Error = ParserError> + Clone {
    let rest_of_line = filter(|(t, _location): &TokenLocation| is_text_token(t)).repeated();

    let dash_pattern = filter(|(t, _): &TokenLocation| matches!(t, Token::Dash))
        .then(filter(|(t, _): &TokenLocation| {
            matches!(t, Token::Whitespace)
        }))
        .chain(rest_of_line);

    let ordered_pattern =
        filter(|(t, _): &TokenLocation| matches!(t, Token::Number(_) | Token::Text(_)))
            .then(filter(|(t, _): &TokenLocation| {
                matches!(t, Token::Period | Token::CloseParen)
            }))
            .then(filter(|(t, _): &TokenLocation| {
                matches!(t, Token::Whitespace)
            }))
            .chain(rest_of_line);

    let paren_pattern = filter(|(t, _): &TokenLocation| matches!(t, Token::OpenParen))
        .then(filter(|(t, _): &TokenLocation| {
            matches!(t, Token::Number(_))
        }))
        .then(filter(|(t, _): &TokenLocation| {
            matches!(t, Token::CloseParen)
        }))
        .then(filter(|(t, _): &TokenLocation| {
            matches!(t, Token::Whitespace)
        }))
        .chain(rest_of_line);

    dash_pattern.or(ordered_pattern).or(paren_pattern)
    // No .map() - preserve tokens!
}

/// Build a list parser
pub(crate) fn build_list_parser<P>(
    _source: Arc<String>,
    items: P,
) -> impl Parser<TokenLocation, ParseNode, Error = ParserError> + Clone
where
    P: Parser<TokenLocation, Vec<ParseNode>, Error = ParserError> + Clone + 'static,
{
    let single_list_item = list_item_line()
        .then_ignore(token(Token::Newline))
        .then(
            filter(|(t, _)| matches!(t, Token::Indent(_)))
                .ignore_then(items)
                .then_ignore(filter(|(t, _)| matches!(t, Token::Dedent(_))))
                .or_not(),
        )
        .map(move |(marker_tokens, maybe_children)| {
            let children = maybe_children.unwrap_or_default();
            ParseNode::new(NodeType::ListItem, marker_tokens, children)
        });

    single_list_item
        .repeated()
        .at_least(2)
        .map(|list_items| ParseNode::new(NodeType::List, vec![], list_items))
}

// ============================================================================
// FOREIGN BLOCK BUILDING
// ============================================================================

/// Parse a foreign block
pub(crate) fn foreign_block(
    _source: Arc<String>,
) -> impl Parser<TokenLocation, ParseNode, Error = ParserError> + Clone {
    // Parse subject tokens (not just text)
    let subject_token_parser =
        filter(|(t, _location): &TokenLocation| !matches!(t, Token::Colon | Token::Newline))
            .repeated()
            .at_least(1)
            .then_ignore(filter(|(t, _): &TokenLocation| matches!(t, Token::Colon)).ignored())
            .then_ignore(filter(|(t, _): &TokenLocation| matches!(t, Token::Newline)).ignored());

    // Parse content that handles nested indentation structures.
    // Returns tokens (not just byte ranges) so we can do indentation wall stripping
    let with_content = filter(|(t, _)| matches!(t, Token::Indent(_)))
        .ignore_then(recursive(|nested_content| {
            choice((
                // Handle nested indentation: properly matched pairs
                filter(|(t, _)| matches!(t, Token::Indent(_)))
                    .ignore_then(nested_content.clone())
                    .then_ignore(filter(|(t, _)| matches!(t, Token::Dedent(_))))
                    .map(|_| (Token::Indent(vec![]), 0..0)), // Dummy token, won't be used
                // Regular content token (not LexMarker, not Dedent)
                filter(|(t, _location): &TokenLocation| {
                    !matches!(t, Token::LexMarker | Token::Dedent(_))
                }),
            ))
            .repeated()
            .at_least(1)
        }))
        .then_ignore(filter(|(t, _)| matches!(t, Token::Dedent(_))))
        .map(|tokens: Vec<TokenLocation>| {
            // Keep tokens (not just ranges) and filter out dummy tokens
            tokens
                .into_iter()
                .filter(|(_, s)| s.start < s.end) // Filter out dummy ranges (0..0)
                .collect::<Vec<_>>()
        });

    let closing_annotation_parser = token(Token::LexMarker)
        .ignore_then(annotation_header())
        .then_ignore(token(Token::LexMarker))
        .then(token(Token::Whitespace).ignore_then(text_line()).or_not())
        .map(move |(header_tokens, content_tokens)| {
            let children = if let Some(tokens) = content_tokens {
                let para_node = ParseNode::new(NodeType::Paragraph, tokens, vec![]);
                vec![para_node]
            } else {
                vec![]
            };
            ParseNode::new(NodeType::Annotation, header_tokens, children)
        });

    subject_token_parser
        .then_ignore(filter(|(t, _)| matches!(t, Token::BlankLine(_))).repeated())
        .then(with_content.or_not())
        .then(closing_annotation_parser)
        .then_ignore(token(Token::Newline).or_not())
        .map(
            move |((subject_tokens, content_tokens), closing_annotation)| {
                let subject_node =
                    ParseNode::new(NodeType::ForeignBlockSubject, subject_tokens, vec![]);
                let content_node = ParseNode::new(
                    NodeType::ForeignBlockContent,
                    content_tokens.unwrap_or_default(),
                    vec![],
                );
                let closing_node = ParseNode::new(
                    NodeType::ForeignBlockClosing,
                    closing_annotation.tokens,
                    closing_annotation.children,
                );
                ParseNode::new(
                    NodeType::ForeignBlock,
                    vec![],
                    vec![subject_node, content_node, closing_node],
                )
            },
        )
}

// NOTE: Label and parameter parsing logic has been moved to
// src/lex/parsers/common/data_extraction.rs as part of the universal AST construction pipeline.
// This ensures both parsers use the same label/parameter parsing logic.
