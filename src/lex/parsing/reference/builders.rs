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
//! (annotations.rs, definitions.rs, sessions.rs, lists.rs, verbatim.rs, combinators.rs, labels.rs, parameters.rs)

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
// Location utilities are now provided by crate::lex::building::location
// See that module for byte_range_to_location, compute_location_from_locations, etc.

/// Check if a token is a text-like token (content that can appear in lines)
///
/// This includes: Text, Whitespace, Numbers, Punctuation, and common symbols.
///
/// # Design Decision: Treating Punctuation as "Text-like"
///
/// This function currently treats all punctuation tokens as "text-like" because
/// the dialog parsing feature needs to include punctuation in text lines.
/// For example, a dialog line like "- Hi mom!!." must have its punctuation tokens
/// (`ExclamationMark`, `Period`) treated as part of the line content.
///
/// # Future Considerations
///
/// If future features need to distinguish between actual words (`Text`, `Number`)
/// and punctuation marks, consider one of these approaches:
///
/// 1. Create a separate `is_word_token()` helper that excludes punctuation
/// 2. Rename this function to `is_line_content_token()` to clarify its purpose
/// 3. Introduce a more granular token classification system
///
/// For now, the current implementation is correct for all existing parsing needs.
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
            | Token::ExclamationMark
            | Token::QuestionMark
            | Token::Semicolon
            | Token::InvertedExclamationMark
            | Token::InvertedQuestionMark
            | Token::Ellipsis
            | Token::IdeographicFullStop
            | Token::FullwidthExclamationMark
            | Token::FullwidthQuestionMark
            | Token::ExclamationQuestionMark
            | Token::QuestionExclamationMark
            | Token::ArabicQuestionMark
            | Token::ArabicFullStop
            | Token::ArabicTripleDot
            | Token::ArabicComma
            | Token::Danda
            | Token::DoubleDanda
            | Token::BengaliCurrencyNumeratorFour
            | Token::EthiopianFullStop
            | Token::ArmenianFullStop
            | Token::TibetanShad
            | Token::ThaiFongman
            | Token::MyanmarComma
            | Token::MyanmarFullStop
    )
}

/// Check if a token represents an actual word (excludes punctuation)
///
/// This helper function is provided for future features that may need to
/// distinguish between word-like tokens (`Text`, `Number`) and punctuation marks.
///
/// Unlike `is_text_token()`, this function does NOT include punctuation,
/// structural tokens, or parameter markers.
///
/// # Example Use Cases
///
/// - Word counting or text analysis that should exclude punctuation
/// - Features that need to process "semantic content" separately from formatting
/// - Validation rules that apply only to actual words, not symbols
#[allow(dead_code)]
pub(crate) fn is_word_token(token: &Token) -> bool {
    matches!(token, Token::Text(_) | Token::Number(_))
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
    let line_with_newline = text_line().then(filter(|(tok, _)| matches!(tok, Token::BlankLine(_))));

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
    filter(|(t, _): &TokenLocation| !matches!(t, Token::LexMarker | Token::BlankLine(_))).repeated()
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
            .then_ignore(filter(|(t, _)| matches!(t, Token::BlankLine(_))))
            .then(
                filter(|(t, _)| matches!(t, Token::Indent(_)))
                    .ignore_then(items.clone())
                    .then_ignore(filter(|(t, _)| matches!(t, Token::Dedent(_))))
                    .or_not(),
            )
            .then_ignore(token(Token::LexMarker))
            .then_ignore(filter(|(t, _)| matches!(t, Token::BlankLine(_))).or_not())
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
            .then_ignore(filter(|(t, _)| matches!(t, Token::BlankLine(_))).or_not())
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
    filter(|(t, _location): &TokenLocation| !matches!(t, Token::Colon | Token::BlankLine(_)))
        .repeated()
        .at_least(1)
        .then_ignore(filter(|(t, _): &TokenLocation| matches!(t, Token::Colon)).ignored())
        .then_ignore(filter(|(t, _): &TokenLocation| matches!(t, Token::BlankLine(_))).ignored())
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
///
/// A session title is a text line followed by one or more blank lines.
/// Multiple consecutive blank lines are semantically equivalent to a single blank line,
/// so we accept 1+ BlankLine tokens (which includes the newline ending the title line).
pub(crate) fn session_title(
) -> impl Parser<TokenLocation, Vec<TokenLocation>, Error = ParserError> + Clone {
    text_line().then_ignore(
        filter(|(t, _)| matches!(t, Token::BlankLine(_)))
            .repeated()
            .at_least(1),
    )
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

/// a dialog line.
pub(crate) fn non_dialog_list_item_line(
) -> impl Parser<TokenLocation, Vec<TokenLocation>, Error = ParserError> + Clone {
    list_item_line().try_map(|tokens: Vec<TokenLocation>, span| {
        let non_whitespace_tokens: Vec<_> = tokens
            .iter()
            .filter(|(t, _)| !t.is_whitespace())
            .map(|(t, _)| t)
            .collect();

        if non_whitespace_tokens.len() >= 2 {
            let last_token = non_whitespace_tokens.last().unwrap();
            let second_to_last_token = non_whitespace_tokens[non_whitespace_tokens.len() - 2];

            if last_token.is_end_punctuation() && second_to_last_token.is_end_punctuation() {
                // This is a dialog line, so we don't want to parse it as a list item.
                Err(Simple::custom(span, "Dialog line mistaken for list item"))
            } else {
                Ok(tokens)
            }
        } else {
            // Not enough tokens to be a dialog line, so it's a valid list item.
            Ok(tokens)
        }
    })
}

/// Build a list parser
pub(crate) fn build_list_parser<P>(
    _source: Arc<String>,
    items: P,
) -> impl Parser<TokenLocation, ParseNode, Error = ParserError> + Clone
where
    P: Parser<TokenLocation, Vec<ParseNode>, Error = ParserError> + Clone + 'static,
{
    let single_list_item = non_dialog_list_item_line()
        .then_ignore(filter(|(t, _)| matches!(t, Token::BlankLine(_))))
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
// VERBATIM BLOCK BUILDING
// ============================================================================

/// Parse a verbatim block
pub(crate) fn verbatim_block(
    _source: Arc<String>,
) -> impl Parser<TokenLocation, ParseNode, Error = ParserError> + Clone {
    // Parse subject tokens (not just text)
    let blank_line = filter(|(t, _): &TokenLocation| matches!(t, Token::BlankLine(_)));
    // Parse subject tokens - must end with colon
    // Using explicit .then() instead of .then_ignore() to ensure proper rewinding
    // when colon is not found (allows session parser to try instead)
    let subject_token_parser =
        filter(|(t, _location): &TokenLocation| !matches!(t, Token::Colon | Token::BlankLine(_)))
            .repeated()
            .at_least(1)
            .then(
                // Explicitly match colon - if not found, this fails and rewinds
                filter(|(t, _): &TokenLocation| matches!(t, Token::Colon)),
            )
            .map(|(tokens, _colon)| {
                let indent_depth = subject_indent_depth(&tokens);
                (tokens, indent_depth)
            })
            .then_ignore(
                // Ignore whitespace after colon (common formatting: "Subject: \n")
                filter(|(t, _): &TokenLocation| matches!(t, Token::Whitespace))
                    .repeated()
                    .then(blank_line.ignored().or_not())
                    .ignored(),
            );

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

    // Parse verbatim group pairs
    // Unified approach: match pairs where blank lines can appear:
    // 1. Before the subject (between pairs)
    // 2. After the subject (before content)
    // 3. After content (between pairs or before closing annotation)
    //
    // The key: we match pairs greedily, consuming blank lines as separators.
    // Blank lines after content of one pair are the same as blank lines before the next subject.
    let pair_with_leading_blanks = blank_line
        .repeated()
        .then(
            subject_token_parser
                .then_ignore(blank_line.repeated())
                .then(with_content.clone().or_not()),
        )
        .map(
            |(_, ((subject_tokens, indent_depth), content_tokens))| VerbatimGroupParseData {
                subject_tokens,
                content_tokens: content_tokens.unwrap_or_default(),
                indent_depth,
            },
        );

    // Match one or more pairs (blank lines before first pair are allowed but optional)
    pair_with_leading_blanks
        .repeated()
        .at_least(1)
        // Consume any remaining blank lines before the closing annotation
        .then_ignore(blank_line.repeated())
        .try_map(|pairs: Vec<VerbatimGroupParseData>, span| {
            let expected_depth = pairs.first().map(|pair| pair.indent_depth).unwrap_or(0);
            if pairs.iter().all(|pair| pair.indent_depth == expected_depth) {
                Ok(pairs)
            } else {
                Err(Simple::custom(
                    span,
                    "Verbatim group subjects must share the same indentation",
                ))
            }
        })
        .map(|pairs| {
            pairs
                .into_iter()
                .flat_map(|pair| {
                    vec![
                        ParseNode::new(
                            NodeType::VerbatimBlockkSubject,
                            pair.subject_tokens,
                            vec![],
                        ),
                        ParseNode::new(
                            NodeType::VerbatimBlockkContent,
                            pair.content_tokens,
                            vec![],
                        ),
                    ]
                })
                .collect::<Vec<_>>()
        })
        .then(closing_annotation_parser)
        .map(move |(mut pair_nodes, closing_annotation)| {
            let closing_node = ParseNode::new(
                NodeType::VerbatimBlockkClosing,
                closing_annotation.tokens,
                closing_annotation.children,
            );
            pair_nodes.push(closing_node);
            ParseNode::new(NodeType::VerbatimBlock, vec![], pair_nodes)
        })
}

#[derive(Clone)]
struct VerbatimGroupParseData {
    subject_tokens: Vec<TokenLocation>,
    content_tokens: Vec<TokenLocation>,
    indent_depth: usize,
}

fn subject_indent_depth(tokens: &[TokenLocation]) -> usize {
    tokens
        .iter()
        .take_while(|(token, _)| matches!(token, Token::Indent(_) | Token::Indentation))
        .count()
}

// NOTE: Label and parameter parsing logic has been moved to
// src/lex/parsers/common/data_extraction.rs as part of the universal AST construction pipeline.
// This ensures both parsers use the same label/parameter parsing logic.
