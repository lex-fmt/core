//! Document-level parser responsible for parsing the entire txxt document.

use chumsky::prelude::*;
use std::ops::Range;

use crate::txxt_nano::ast::position::SourceLocation;
use crate::txxt_nano::ast::Document;
use crate::txxt_nano::ast::{
    Annotation, ContentItem, Definition, Label, List, ListItem, Paragraph, Session, Span,
    TextContent,
};
use crate::txxt_nano::lexer::Token;
use crate::txxt_nano::parser::combinators::{
    annotation_header, definition_subject, foreign_block, list_item_line, paragraph, session_title,
    token,
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

/// Build the Multi-Parser Bundle for document-level content parsing.
///
/// Phase 4: This parser now builds final ContentItem types directly using refactored combinators.
/// All combinators now take source parameter and return final types.
pub(crate) fn build_document_content_parser(
    source: &str,
) -> impl Parser<TokenSpan, Vec<ContentItem>, Error = ParserError> + Clone {
    let source = source.to_string();

    recursive(move |items| {
        let source = source.clone();
        let single_item = {
            // Session parser - now builds final Session type with span
            let session_parser = {
                let source_for_session = source.clone();
                session_title(&source)
                    .then(
                        token(Token::IndentLevel)
                            .ignore_then(items.clone())
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
            };

            // Definition parser - now builds final Definition type with span
            let definition_parser = {
                let source_for_definition = source.clone();
                definition_subject(&source)
                    .then(
                        token(Token::IndentLevel)
                            .ignore_then(items.clone())
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
            };

            // List parser - now builds final List type with span
            let list_parser = {
                let source = source.clone();
                let single_list_item = list_item_line(&source)
                    .then_ignore(token(Token::Newline))
                    .then(
                        token(Token::IndentLevel)
                            .ignore_then(items.clone())
                            .then_ignore(token(Token::DedentLevel))
                            .or_not(),
                    )
                    .map(move |((text, text_span), maybe_content)| {
                        let span = byte_range_to_span(&source, &text_span);
                        ListItem::with_content(text, maybe_content.unwrap_or_default())
                            .with_span(span)
                    });

                single_list_item.repeated().at_least(2).map(|items| {
                    // Compute span from all list item spans
                    let spans: Vec<Option<Span>> = items.iter().map(|item| item.span).collect();
                    let span = if spans.iter().any(|s| s.is_some()) {
                        let start_line = spans
                            .iter()
                            .filter_map(|s| s.map(|sp| sp.start.line))
                            .min()
                            .unwrap_or(0);
                        let start_col = spans
                            .iter()
                            .filter_map(|s| s.map(|sp| sp.start.column))
                            .min()
                            .unwrap_or(0);
                        let end_line = spans
                            .iter()
                            .filter_map(|s| s.map(|sp| sp.end.line))
                            .max()
                            .unwrap_or(0);
                        let end_col = spans
                            .iter()
                            .filter_map(|s| s.map(|sp| sp.end.column))
                            .max()
                            .unwrap_or(0);
                        use crate::txxt_nano::ast::span::Position;
                        Some(Span::new(
                            Position::new(start_line, start_col),
                            Position::new(end_line, end_col),
                        ))
                    } else {
                        None
                    };
                    ContentItem::List(List { items, span })
                })
            };

            // Annotation parser - now builds final Annotation type with span
            let annotation_parser = {
                let source = source.clone();
                let header = token(Token::TxxtMarker)
                    .ignore_then(annotation_header(&source))
                    .then_ignore(token(Token::TxxtMarker));

                let block_form = {
                    let source_for_block = source.clone();
                    header
                        .clone()
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
                                    let start_line = content_spans
                                        .iter()
                                        .map(|sp| sp.start.line)
                                        .min()
                                        .unwrap_or(0);
                                    let start_col = content_spans
                                        .iter()
                                        .map(|sp| sp.start.column)
                                        .min()
                                        .unwrap_or(0);
                                    let end_line = content_spans
                                        .iter()
                                        .map(|sp| sp.end.line)
                                        .max()
                                        .unwrap_or(0);
                                    let end_col = content_spans
                                        .iter()
                                        .map(|sp| sp.end.column)
                                        .max()
                                        .unwrap_or(0);
                                    use crate::txxt_nano::ast::span::Position;
                                    Some(Span::new(
                                        Position::new(start_line, start_col),
                                        Position::new(end_line, end_col),
                                    ))
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
                    header
                        .then(
                            token(Token::Whitespace)
                                .ignore_then(crate::txxt_nano::parser::combinators::text_line())
                                .or_not(),
                        )
                        .then_ignore(token(Token::Newline).or_not())
                        .map(move |((label_opt, label_span, parameters), content_span)| {
                            let label_text = label_opt.unwrap_or_default();
                            let label_position = label_span
                                .and_then(|s| byte_range_to_span(&source_for_single_line, &s));
                            let label = Label::new(label_text).with_span(label_position);

                            // Handle content if present
                            let content = if let Some(spans) = content_span {
                                let text =
                                    crate::txxt_nano::parser::combinators::extract_text_from_spans(
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
            };

            choice((
                foreign_block(&source).map(ContentItem::ForeignBlock),
                annotation_parser,
                list_parser,
                definition_parser,
                session_parser,
                paragraph(&source).map(ContentItem::Paragraph),
            ))
        };

        choice((
            token(Token::BlankLine)
                .repeated()
                .at_least(1)
                .ignore_then(choice((
                    filter(|(t, _)| matches!(t, Token::DocEnd | Token::DedentLevel))
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
            filter(|(t, _)| matches!(t, Token::DocEnd | Token::DedentLevel))
                .rewind()
                .to(vec![]),
        ))
    })
}

/// Parse a document
///
/// Phase 5: Returns final Document directly with no intermediate conversion.
pub fn document(source: &str) -> impl Parser<TokenSpan, Document, Error = ParserError> + Clone {
    let content_item = build_document_content_parser(source);

    token(Token::DocStart)
        .ignore_then(content_item)
        .then_ignore(token(Token::DocEnd))
        .map(|content| Document {
            metadata: Vec::new(),
            content,
            span: None,
        })
}
