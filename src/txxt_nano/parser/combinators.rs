//! Parser combinator functions for building the txxt parser.

use chumsky::prelude::*;
use std::ops::Range;

use crate::txxt_nano::lexer::Token;
use crate::txxt_nano::parser::conversion::helpers::is_text_token;
use crate::txxt_nano::parser::intermediate_ast::{
    AnnotationWithSpans, ContentItemWithSpans, ForeignBlockWithSpans, ParagraphWithSpans,
};
use crate::txxt_nano::parser::labels::parse_label_from_tokens;
use crate::txxt_nano::parser::parameters::{parse_parameters_from_tokens, ParameterWithSpans};

/// Type alias for token with span
type TokenSpan = (Token, Range<usize>);

/// Type alias for parser error
type ParserError = Simple<TokenSpan>;

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
pub(crate) fn list_item_line(
) -> impl Parser<TokenSpan, Vec<Range<usize>>, Error = ParserError> + Clone {
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
        .map(|tokens_with_spans: Vec<TokenSpan>| {
            tokens_with_spans.into_iter().map(|(_, s)| s).collect()
        })
}

/// Parse a paragraph
pub(crate) fn paragraph() -> impl Parser<TokenSpan, ParagraphWithSpans, Error = ParserError> + Clone
{
    text_line()
        .then_ignore(token(Token::Newline))
        .repeated()
        .at_least(1)
        .map(|line_spans| ParagraphWithSpans { line_spans })
}

/// Parse a definition subject
pub(crate) fn definition_subject(
) -> impl Parser<TokenSpan, Vec<Range<usize>>, Error = ParserError> + Clone {
    filter(|(t, _span): &TokenSpan| !matches!(t, Token::Colon | Token::Newline))
        .repeated()
        .at_least(1)
        .map(|tokens_with_spans: Vec<TokenSpan>| {
            tokens_with_spans.into_iter().map(|(_, s)| s).collect()
        })
        .then_ignore(token(Token::Colon))
        .then_ignore(token(Token::Newline))
}

/// Parse a session title
pub(crate) fn session_title(
) -> impl Parser<TokenSpan, Vec<Range<usize>>, Error = ParserError> + Clone {
    text_line()
        .then_ignore(token(Token::Newline))
        .then_ignore(token(Token::BlankLine))
}

/// Parse the bounded region between :: markers
pub(crate) fn annotation_header(
) -> impl Parser<TokenSpan, (Option<Range<usize>>, Vec<ParameterWithSpans>), Error = ParserError> + Clone
{
    let bounded_region =
        filter(|(t, _): &TokenSpan| !matches!(t, Token::TxxtMarker | Token::Newline))
            .repeated()
            .at_least(1);

    bounded_region.validate(|tokens, span, emit| {
        if tokens.is_empty() {
            emit(ParserError::expected_input_found(span, None, None));
            return (None, Vec::new());
        }

        let (label_span, mut i) = parse_label_from_tokens(&tokens);

        if label_span.is_none() && i == 0 {
            while i < tokens.len() && matches!(tokens[i].0, Token::Whitespace) {
                i += 1;
            }
        }

        let params = parse_parameters_from_tokens(&tokens[i..]);

        (label_span, params)
    })
}

/// Parse a foreign block
pub(crate) fn foreign_block(
) -> impl Parser<TokenSpan, ForeignBlockWithSpans, Error = ParserError> + Clone {
    let subject_parser = definition_subject();

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

    let closing_annotation_parser = token(Token::TxxtMarker)
        .ignore_then(annotation_header())
        .then_ignore(token(Token::TxxtMarker))
        .then(token(Token::Whitespace).ignore_then(text_line()).or_not())
        .map(|((label_span, parameters), content_span)| {
            let content = content_span
                .map(|span| {
                    vec![ContentItemWithSpans::Paragraph(ParagraphWithSpans {
                        line_spans: vec![span],
                    })]
                })
                .unwrap_or_default();

            AnnotationWithSpans {
                label_span,
                parameters,
                content,
            }
        });

    subject_parser
        .then_ignore(token(Token::BlankLine).repeated())
        .then(with_content.or_not())
        .then(closing_annotation_parser)
        .then_ignore(token(Token::Newline).or_not())
        .map(
            |((subject_spans, content_spans), closing_annotation)| ForeignBlockWithSpans {
                subject_spans,
                content_spans,
                closing_annotation,
            },
        )
}
