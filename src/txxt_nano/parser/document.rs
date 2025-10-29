//! Document-level parser responsible for parsing the entire txxt document.

use chumsky::prelude::*;
use std::ops::Range;
use std::sync::Arc;

use crate::txxt_nano::ast::position::SourceLocation;
use crate::txxt_nano::ast::Document;
use crate::txxt_nano::ast::{
    Annotation, ContentItem, Definition, Label, List, ListItem, Paragraph, Session, Span,
    TextContent,
};
use crate::txxt_nano::lexer::Token;
use crate::txxt_nano::parser::combinators::{
    annotation_header, compute_span_from_optional_spans, compute_span_from_spans,
    definition_subject, foreign_block, list_item_line, paragraph, session_title, token,
};

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

/// Build a session parser
fn build_session_parser<P>(
    source: Arc<String>,
    items: P,
) -> impl Parser<TokenSpan, ContentItem, Error = ParserError> + Clone
where
    P: Parser<TokenSpan, Vec<ContentItem>, Error = ParserError> + Clone + 'static,
{
    let source_for_session = source.clone();
    session_title(source.clone())
        .then(
            token(Token::IndentLevel)
                .ignore_then(items)
                .then_ignore(token(Token::DedentLevel)),
        )
        .map(move |((title_text, title_span), content)| {
            let span = byte_range_to_span(&source_for_session, &title_span);
            ContentItem::Session(Session {
                title: TextContent::from_string(title_text, None),
                content,
                span,
            })
        })
}

/// Build a definition parser
fn build_definition_parser<P>(
    source: Arc<String>,
    items: P,
) -> impl Parser<TokenSpan, ContentItem, Error = ParserError> + Clone
where
    P: Parser<TokenSpan, Vec<ContentItem>, Error = ParserError> + Clone + 'static,
{
    let source_for_definition = source.clone();
    definition_subject(source.clone())
        .then(
            token(Token::IndentLevel)
                .ignore_then(items)
                .then_ignore(token(Token::DedentLevel)),
        )
        .map(move |((subject_text, subject_span), content)| {
            let span = byte_range_to_span(&source_for_definition, &subject_span);
            ContentItem::Definition(Definition {
                subject: TextContent::from_string(subject_text, None),
                content,
                span,
            })
        })
}

/// Build a list parser
fn build_list_parser<P>(
    source: Arc<String>,
    items: P,
) -> impl Parser<TokenSpan, ContentItem, Error = ParserError> + Clone
where
    P: Parser<TokenSpan, Vec<ContentItem>, Error = ParserError> + Clone + 'static,
{
    let source_for_list = source.clone();
    let single_list_item = list_item_line(source.clone())
        .then_ignore(token(Token::Newline))
        .then(
            token(Token::IndentLevel)
                .ignore_then(items)
                .then_ignore(token(Token::DedentLevel))
                .or_not(),
        )
        .map(move |((text, text_span), maybe_content)| {
            let span = byte_range_to_span(&source_for_list, &text_span);
            ListItem::with_content(text, maybe_content.unwrap_or_default()).with_span(span)
        });

    single_list_item.repeated().at_least(2).map(|items| {
        let spans: Vec<Option<Span>> = items.iter().map(|item| item.span).collect();
        let span = compute_span_from_optional_spans(&spans);
        ContentItem::List(List { items, span })
    })
}

/// Build an annotation parser
fn build_annotation_parser<P>(
    source: Arc<String>,
    items: P,
) -> impl Parser<TokenSpan, ContentItem, Error = ParserError> + Clone
where
    P: Parser<TokenSpan, Vec<ContentItem>, Error = ParserError> + Clone + 'static,
{
    let source_for_header = source.clone();
    let header = token(Token::TxxtMarker)
        .ignore_then(annotation_header(source_for_header))
        .then_ignore(token(Token::TxxtMarker));

    let block_form = {
        let source_for_block = source.clone();
        let header_for_block = header.clone();
        header_for_block
            .then_ignore(token(Token::Newline))
            .then(
                token(Token::IndentLevel)
                    .ignore_then(items.clone())
                    .then_ignore(token(Token::DedentLevel)),
            )
            .then_ignore(token(Token::TxxtMarker))
            .then_ignore(token(Token::Newline).or_not())
            .map(move |((label_opt, label_span, parameters), content)| {
                let label_text = label_opt.unwrap_or_default();
                let label_position =
                    label_span.and_then(|s| byte_range_to_span(&source_for_block, &s));
                let label = Label::new(label_text).with_span(label_position);
                // Compute overall span from content spans if available
                let span = if !content.is_empty() {
                    let content_spans: Vec<Span> = content
                        .iter()
                        .filter_map(|item| match item {
                            ContentItem::Paragraph(p) => p.span,
                            ContentItem::Session(s) => s.span,
                            ContentItem::Definition(d) => d.span,
                            ContentItem::List(l) => l.span,
                            ContentItem::Annotation(a) => a.span,
                            ContentItem::ForeignBlock(f) => f.span,
                        })
                        .collect();
                    if !content_spans.is_empty() {
                        Some(compute_span_from_spans(&content_spans))
                    } else {
                        None
                    }
                } else {
                    label_position
                };
                ContentItem::Annotation(Annotation {
                    label,
                    parameters,
                    content,
                    span,
                })
            })
    };

    let single_line_or_marker = {
        let source_for_single_line = source.clone();
        let header_for_single = header.clone();
        header_for_single
            .then(
                token(Token::Whitespace)
                    .ignore_then(crate::txxt_nano::parser::combinators::text_line())
                    .or_not(),
            )
            .then_ignore(token(Token::Newline).or_not())
            .map(move |((label_opt, label_span, parameters), content_span)| {
                let label_text = label_opt.unwrap_or_default();
                let label_position =
                    label_span.and_then(|s| byte_range_to_span(&source_for_single_line, &s));
                let label = Label::new(label_text).with_span(label_position);

                // Handle content if present
                let content = if let Some(spans) = content_span {
                    let text = crate::txxt_nano::parser::combinators::extract_text_from_spans(
                        &source_for_single_line,
                        &spans,
                    );
                    vec![ContentItem::Paragraph(Paragraph {
                        lines: vec![TextContent::from_string(text, None)],
                        span: None,
                    })]
                } else {
                    vec![]
                };
                let span = label_position; // For single-line, span is just the label

                ContentItem::Annotation(Annotation {
                    label,
                    parameters,
                    content,
                    span,
                })
            })
    };

    block_form.or(single_line_or_marker)
}

/// Build the Multi-Parser Bundle for document-level content parsing.
///
/// Phase 4: This parser now builds final ContentItem types directly using refactored combinators.
/// All combinators now take source parameter and return final types.
pub(crate) fn build_document_content_parser(
    source: &str,
) -> impl Parser<TokenSpan, Vec<ContentItem>, Error = ParserError> + Clone {
    let source = Arc::new(source.to_string());

    recursive(move |items| {
        let source = source.clone();
        let single_item = {
            // Session parser - now builds final Session type with span
            let session_parser = build_session_parser(source.clone(), items.clone());

            // Definition parser - now builds final Definition type with span
            let definition_parser = build_definition_parser(source.clone(), items.clone());

            // List parser - now builds final List type with span
            let list_parser = build_list_parser(source.clone(), items.clone());

            // Annotation parser - now builds final Annotation type with span
            let annotation_parser = build_annotation_parser(source.clone(), items.clone());

            choice((
                foreign_block(source.clone()).map(ContentItem::ForeignBlock),
                annotation_parser,
                list_parser,
                definition_parser,
                session_parser,
                paragraph(source.clone()).map(ContentItem::Paragraph),
            ))
        };

        choice((
            token(Token::BlankLine)
                .repeated()
                .at_least(1)
                .ignore_then(choice((
                    filter(|(t, _)| matches!(t, Token::DedentLevel))
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
            filter(|(t, _)| matches!(t, Token::DedentLevel))
                .rewind()
                .to(vec![]),
        ))
    })
}

/// Parse a document
///
/// Parses the entire token stream as document content.
pub fn document(source: &str) -> impl Parser<TokenSpan, Document, Error = ParserError> + Clone {
    build_document_content_parser(source).map(|content| Document {
        metadata: Vec::new(),
        content,
        span: None,
    })
}
