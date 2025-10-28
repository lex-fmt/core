//! Document-level parser responsible for parsing the entire txxt document.

use chumsky::prelude::*;
use std::ops::Range;

use crate::txxt_nano::ast::{
    Annotation, ContentItem, Definition, Label, List, ListItem, Session, TextContent,
};
use crate::txxt_nano::lexer::Token;
use crate::txxt_nano::parser::combinators::{
    annotation_header, definition_subject, foreign_block, list_item_line, paragraph, session_title,
    token,
};
use crate::txxt_nano::parser::intermediate_ast::DocumentWithSpans;

/// Type alias for token with span
type TokenSpan = (Token, Range<usize>);

/// Type alias for parser error
type ParserError = Simple<TokenSpan>;

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
            // Session parser - now builds final Session type
            let session_parser = session_title(&source)
                .then(
                    token(Token::IndentLevel)
                        .ignore_then(items.clone())
                        .then_ignore(token(Token::DedentLevel)),
                )
                .map(|(title, content)| {
                    ContentItem::Session(Session {
                        title: TextContent::from_string(title, None),
                        content,
                        span: None,
                    })
                });

            // Definition parser - now builds final Definition type
            let definition_parser = definition_subject(&source)
                .then(
                    token(Token::IndentLevel)
                        .ignore_then(items.clone())
                        .then_ignore(token(Token::DedentLevel)),
                )
                .map(|(subject, content)| {
                    ContentItem::Definition(Definition {
                        subject: TextContent::from_string(subject, None),
                        content,
                        span: None,
                    })
                });

            // List parser - now builds final List type
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
                    .map(|(text, maybe_content)| {
                        ListItem::with_content(text, maybe_content.unwrap_or_default())
                    });

                single_list_item
                    .repeated()
                    .at_least(2)
                    .map(|items| ContentItem::List(List { items, span: None }))
            };

            // Annotation parser - now builds final Annotation type
            let annotation_parser = {
                let source = source.clone();
                let header = token(Token::TxxtMarker)
                    .ignore_then(annotation_header(&source))
                    .then_ignore(token(Token::TxxtMarker));

                let block_form = {
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
                        .map(move |((label_opt, parameters), content)| {
                            let label = Label::new(label_opt.unwrap_or_default());
                            ContentItem::Annotation(Annotation {
                                label,
                                parameters,
                                content,
                                span: None,
                            })
                        })
                };

                let single_line_or_marker = {
                    header
                        .then(
                            token(Token::Whitespace)
                                .ignore_then(crate::txxt_nano::parser::combinators::text_line())
                                .or_not(),
                        )
                        .then_ignore(token(Token::Newline).or_not())
                        .map(move |((label_opt, parameters), _content_span)| {
                            let label = Label::new(label_opt.unwrap_or_default());
                            let content = vec![]; // Single-line annotations have no content

                            ContentItem::Annotation(Annotation {
                                label,
                                parameters,
                                content,
                                span: None,
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
/// Phase 3b: Requires source text to convert intermediate types to final types at parse time.
#[allow(private_interfaces)]
pub fn document(
    source: &str,
) -> impl Parser<TokenSpan, DocumentWithSpans, Error = ParserError> + Clone {
    let content_item = build_document_content_parser(source);

    token(Token::DocStart)
        .ignore_then(content_item)
        .then_ignore(token(Token::DocEnd))
        .map(|content| DocumentWithSpans {
            metadata: Vec::new(),
            content,
        })
}
