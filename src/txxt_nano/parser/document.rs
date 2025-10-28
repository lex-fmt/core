//! Document-level parser responsible for parsing the entire txxt document.

use chumsky::prelude::*;
use std::ops::Range;

use crate::txxt_nano::lexer::Token;
use crate::txxt_nano::parser::combinators::{
    annotation_header, definition_subject, foreign_block, list_item_line, paragraph, session_title,
    token,
};
use crate::txxt_nano::parser::intermediate_ast::{
    AnnotationWithSpans, ContentItemWithSpans, DefinitionWithSpans, DocumentWithSpans,
    ListItemWithSpans, ListWithSpans, ParagraphWithSpans, SessionWithSpans,
};

/// Type alias for token with span
type TokenSpan = (Token, Range<usize>);

/// Type alias for parser error
type ParserError = Simple<TokenSpan>;

/// Build the Multi-Parser Bundle for document-level content parsing.
pub(crate) fn build_document_content_parser(
) -> impl Parser<TokenSpan, Vec<ContentItemWithSpans>, Error = ParserError> + Clone {
    recursive(|items| {
        let single_item = {
            let session_parser = session_title()
                .then(
                    token(Token::IndentLevel)
                        .ignore_then(items.clone())
                        .then_ignore(token(Token::DedentLevel)),
                )
                .map(|(title_spans, content)| {
                    ContentItemWithSpans::Session(SessionWithSpans {
                        title_spans,
                        content,
                    })
                });

            let definition_parser = definition_subject()
                .then(
                    token(Token::IndentLevel)
                        .ignore_then(items.clone())
                        .then_ignore(token(Token::DedentLevel)),
                )
                .map(|(subject_spans, content)| {
                    ContentItemWithSpans::Definition(DefinitionWithSpans {
                        subject_spans,
                        content,
                    })
                });

            let list_parser = {
                let single_list_item = list_item_line()
                    .then_ignore(token(Token::Newline))
                    .then(
                        token(Token::IndentLevel)
                            .ignore_then(items.clone())
                            .then_ignore(token(Token::DedentLevel))
                            .or_not(),
                    )
                    .map(|(text_spans, maybe_content)| ListItemWithSpans {
                        text_spans,
                        content: maybe_content.unwrap_or_default(),
                    });

                single_list_item
                    .repeated()
                    .at_least(2)
                    .map(|items| ContentItemWithSpans::List(ListWithSpans { items }))
            };

            let annotation_parser = {
                let header = token(Token::TxxtMarker)
                    .ignore_then(annotation_header())
                    .then_ignore(token(Token::TxxtMarker));

                let block_form = header
                    .clone()
                    .then_ignore(token(Token::Newline))
                    .then(
                        token(Token::IndentLevel)
                            .ignore_then(items.clone())
                            .then_ignore(token(Token::DedentLevel)),
                    )
                    .then_ignore(token(Token::TxxtMarker))
                    .then_ignore(token(Token::Newline).or_not())
                    .map(|((label_span, parameters), content)| AnnotationWithSpans {
                        label_span,
                        parameters,
                        content,
                    });

                let single_line_or_marker = header
                    .then(
                        token(Token::Whitespace)
                            .ignore_then(crate::txxt_nano::parser::combinators::text_line())
                            .or_not(),
                    )
                    .then_ignore(token(Token::Newline).or_not())
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

                block_form
                    .or(single_line_or_marker)
                    .map(ContentItemWithSpans::Annotation)
            };

            choice((
                foreign_block().map(ContentItemWithSpans::ForeignBlock),
                annotation_parser,
                list_parser,
                definition_parser,
                session_parser,
                paragraph().map(ContentItemWithSpans::Paragraph),
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
#[allow(private_interfaces)]
pub fn document() -> impl Parser<TokenSpan, DocumentWithSpans, Error = ParserError> {
    let content_item = build_document_content_parser();

    token(Token::DocStart)
        .ignore_then(content_item)
        .then_ignore(token(Token::DocEnd))
        .map(|content| DocumentWithSpans {
            metadata: Vec::new(),
            content,
        })
}
