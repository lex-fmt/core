//! Parser combinator functions for building the txxt parser.

use chumsky::prelude::*;
use std::ops::Range;
use std::sync::Arc;

use crate::txxt_nano::ast::position::SourceLocation;
use crate::txxt_nano::ast::{
    Annotation, ContentItem, ForeignBlock, Label, Paragraph, Parameter, Span, TextContent,
};
use crate::txxt_nano::lexer::Token;
use crate::txxt_nano::parser::conversion::helpers::is_text_token;
use crate::txxt_nano::parser::labels::parse_label_from_tokens;
use crate::txxt_nano::parser::parameters::{convert_parameter, parse_parameters_from_tokens};

/// Type alias for token with span
type TokenSpan = (Token, Range<usize>);

/// Type alias for parser error
type ParserError = Simple<TokenSpan>;

/// Helper: convert a byte range to a Span using source location
fn byte_range_to_span(source: &str, range: &Range<usize>) -> Option<Span> {
    if range.start > range.end {
        return None;
    }
    let source_loc = SourceLocation::new(source);
    Some(source_loc.range_to_span(range))
}

/// Helper: compute span bounds from multiple spans
pub(crate) fn compute_span_from_spans(spans: &[Span]) -> Span {
    use crate::txxt_nano::ast::span::Position;
    let start_line = spans.iter().map(|sp| sp.start.line).min().unwrap_or(0);
    let start_col = spans.iter().map(|sp| sp.start.column).min().unwrap_or(0);
    let end_line = spans.iter().map(|sp| sp.end.line).max().unwrap_or(0);
    let end_col = spans.iter().map(|sp| sp.end.column).max().unwrap_or(0);
    Span::new(
        Position::new(start_line, start_col),
        Position::new(end_line, end_col),
    )
}

/// Helper: compute span bounds from multiple optional spans
pub(crate) fn compute_span_from_optional_spans(spans: &[Option<Span>]) -> Option<Span> {
    let actual_spans: Vec<Span> = spans.iter().filter_map(|s| *s).collect();
    if actual_spans.is_empty() {
        None
    } else {
        Some(compute_span_from_spans(&actual_spans))
    }
}

/// Helper: compute span bounds from byte ranges
pub(crate) fn compute_byte_range_bounds(ranges: &[Range<usize>]) -> Range<usize> {
    if ranges.is_empty() {
        0..0
    } else {
        let start = ranges.iter().map(|r| r.start).min().unwrap_or(0);
        let end = ranges.iter().map(|r| r.end).max().unwrap_or(0);
        start..end
    }
}

/// Helper: extract text from multiple spans
pub(crate) fn extract_text_from_spans(source: &str, spans: &[Range<usize>]) -> String {
    if spans.is_empty() {
        return String::new();
    }
    let start = spans.first().map(|s| s.start).unwrap_or(0);
    let end = spans.last().map(|s| s.end).unwrap_or(0);

    if start >= end || end > source.len() {
        return String::new();
    }

    source[start..end].trim().to_string()
}

/// Helper: extract tokens to text and byte range span
/// Converts a vector of token-span pairs to (extracted_text, byte_range)
pub(crate) fn extract_tokens_to_text_and_span(
    source: &Arc<String>,
    tokens: Vec<TokenSpan>,
) -> (String, Range<usize>) {
    let spans: Vec<Range<usize>> = tokens.into_iter().map(|(_, s)| s).collect();
    let text = extract_text_from_spans(source, &spans);
    let span = compute_byte_range_bounds(&spans);
    (text, span)
}

/// Helper: match a specific token type, ignoring the span
pub(crate) fn token(t: Token) -> impl Parser<TokenSpan, (), Error = ParserError> + Clone {
    filter(move |(tok, _)| tok == &t).ignored()
}

/// Parse a text line (sequence of text and whitespace tokens)
/// Returns the collected spans for this line
pub(crate) fn text_line() -> impl Parser<TokenSpan, Vec<Range<usize>>, Error = ParserError> + Clone
{
    filter(|(t, _span): &TokenSpan| is_text_token(t))
        .repeated()
        .at_least(1)
        .map(|tokens_with_spans: Vec<TokenSpan>| {
            // Collect all spans for this line
            tokens_with_spans.into_iter().map(|(_, s)| s).collect()
        })
}

/// Parse a list item line - a line that starts with a list marker
/// Phase 5: Now returns extracted text with span information
pub(crate) fn list_item_line(
    source: Arc<String>,
) -> impl Parser<TokenSpan, (String, Range<usize>), Error = ParserError> + Clone {
    let rest_of_line = filter(|(t, _span): &TokenSpan| is_text_token(t)).repeated();

    let dash_pattern = filter(|(t, _): &TokenSpan| matches!(t, Token::Dash))
        .then(filter(|(t, _): &TokenSpan| matches!(t, Token::Whitespace)))
        .chain(rest_of_line);

    let ordered_pattern =
        filter(|(t, _): &TokenSpan| matches!(t, Token::Number(_) | Token::Text(_)))
            .then(filter(|(t, _): &TokenSpan| {
                matches!(t, Token::Period | Token::CloseParen)
            }))
            .then(filter(|(t, _): &TokenSpan| matches!(t, Token::Whitespace)))
            .chain(rest_of_line);

    let paren_pattern = filter(|(t, _): &TokenSpan| matches!(t, Token::OpenParen))
        .then(filter(|(t, _): &TokenSpan| matches!(t, Token::Number(_))))
        .then(filter(|(t, _): &TokenSpan| matches!(t, Token::CloseParen)))
        .then(filter(|(t, _): &TokenSpan| matches!(t, Token::Whitespace)))
        .chain(rest_of_line);

    dash_pattern
        .or(ordered_pattern)
        .or(paren_pattern)
        .map(move |tokens_with_spans| extract_tokens_to_text_and_span(&source, tokens_with_spans))
}

/// Parse a paragraph
/// Phase 5: Now populates span information
pub(crate) fn paragraph(
    source: Arc<String>,
) -> impl Parser<TokenSpan, Paragraph, Error = ParserError> + Clone {
    text_line()
        .then_ignore(token(Token::Newline))
        .repeated()
        .at_least(1)
        .map(move |line_spans_list: Vec<Vec<Range<usize>>>| {
            let lines = line_spans_list
                .iter()
                .map(|spans| {
                    let text = extract_text_from_spans(&source, spans);
                    // Compute span for this line
                    let line_span = if spans.is_empty() {
                        None
                    } else {
                        let range = compute_byte_range_bounds(spans);
                        byte_range_to_span(&source, &range)
                    };
                    TextContent::from_string(text, line_span)
                })
                .collect();

            // Compute overall span from all collected line spans
            let span = {
                let all_spans: Vec<Range<usize>> = line_spans_list.into_iter().flatten().collect();
                if all_spans.is_empty() {
                    None
                } else {
                    let range = compute_byte_range_bounds(&all_spans);
                    byte_range_to_span(&source, &range)
                }
            };

            Paragraph { lines, span }
        })
}

/// Parse a definition subject
/// Phase 5: Now returns extracted text with span information
pub(crate) fn definition_subject(
    source: Arc<String>,
) -> impl Parser<TokenSpan, (String, Range<usize>), Error = ParserError> + Clone {
    filter(|(t, _span): &TokenSpan| !matches!(t, Token::Colon | Token::Newline))
        .repeated()
        .at_least(1)
        .map(move |tokens_with_spans| extract_tokens_to_text_and_span(&source, tokens_with_spans))
        .then_ignore(token(Token::Colon))
        .then_ignore(token(Token::Newline))
}

/// Parse a session title
/// Phase 5: Now returns extracted text with span information
pub(crate) fn session_title(
    source: Arc<String>,
) -> impl Parser<TokenSpan, (String, Range<usize>), Error = ParserError> + Clone {
    text_line()
        .then_ignore(token(Token::Newline))
        .then_ignore(token(Token::BlankLine))
        .map(move |spans| {
            let text = extract_text_from_spans(&source, &spans);
            let span = compute_byte_range_bounds(&spans);
            (text, span)
        })
}

/// Parse the bounded region between :: markers
/// Phase 5: Now returns extracted label text, label span, and final Parameter types
pub(crate) fn annotation_header(
    source: Arc<String>,
) -> impl Parser<
    TokenSpan,
    (Option<String>, Option<Range<usize>>, Vec<Parameter>),
    Error = ParserError,
> + Clone {
    let bounded_region =
        filter(|(t, _): &TokenSpan| !matches!(t, Token::TxxtMarker | Token::Newline))
            .repeated()
            .at_least(1);

    bounded_region.validate(move |tokens, span, emit| {
        if tokens.is_empty() {
            emit(ParserError::expected_input_found(span, None, None));
            return (None, None, Vec::new());
        }

        let (label_span, mut i) = parse_label_from_tokens(&tokens);

        if label_span.is_none() && i == 0 {
            while i < tokens.len() && matches!(tokens[i].0, Token::Whitespace) {
                i += 1;
            }
        }

        let params_with_spans = parse_parameters_from_tokens(&tokens[i..]);

        // Extract label text if present
        let label = label_span.as_ref().map(|span| {
            let text = if span.start < span.end && span.end <= source.len() {
                source[span.start..span.end].trim().to_string()
            } else {
                String::new()
            };
            text
        });

        // Convert parameters to final types
        let params = params_with_spans
            .into_iter()
            .map(|p| convert_parameter(&source, p))
            .collect();

        (label, label_span, params)
    })
}

/// Parse a foreign block
/// Phase 4: Now builds final ForeignBlock type directly
pub(crate) fn foreign_block(
    source: Arc<String>,
) -> impl Parser<TokenSpan, ForeignBlock, Error = ParserError> + Clone {
    let subject_parser = definition_subject(source.clone());

    let content_token = filter(|(t, _span): &TokenSpan| !matches!(t, Token::TxxtMarker));

    let with_content = token(Token::IndentLevel)
        .ignore_then(content_token.repeated().at_least(1))
        .map(|tokens: Vec<TokenSpan>| {
            let mut content_tokens = tokens;
            while content_tokens
                .last()
                .map(|(t, _)| matches!(t, Token::DedentLevel | Token::BlankLine | Token::Newline))
                .unwrap_or(false)
            {
                content_tokens.pop();
            }

            content_tokens
                .into_iter()
                .map(|(_, s)| s)
                .collect::<Vec<_>>()
        });

    let source_for_annotation = source.clone();
    let closing_annotation_parser = token(Token::TxxtMarker)
        .ignore_then(annotation_header(source_for_annotation.clone()))
        .then_ignore(token(Token::TxxtMarker))
        .then(token(Token::Whitespace).ignore_then(text_line()).or_not())
        .map(
            move |((label_opt, _label_span, parameters), content_span)| {
                // Build Annotation from extracted label and parameters
                let label = Label::new(label_opt.unwrap_or_default());

                let content = content_span
                    .map(|spans| {
                        let text = extract_text_from_spans(&source_for_annotation, &spans);
                        vec![ContentItem::Paragraph(Paragraph {
                            lines: vec![TextContent::from_string(text, None)],
                            span: None,
                        })]
                    })
                    .unwrap_or_default();

                Annotation {
                    label,
                    parameters,
                    content,
                    span: None,
                }
            },
        );

    subject_parser
        .then_ignore(token(Token::BlankLine).repeated())
        .then(with_content.or_not())
        .then(closing_annotation_parser)
        .then_ignore(token(Token::Newline).or_not())
        .map(
            move |(((subject_text, _subject_span), content_spans), closing_annotation)| {
                let content = content_spans
                    .map(|spans| extract_text_from_spans(&source, &spans))
                    .unwrap_or_default();

                ForeignBlock {
                    subject: TextContent::from_string(subject_text, None),
                    content: TextContent::from_string(content, None),
                    closing_annotation,
                    span: None,
                }
            },
        )
}
