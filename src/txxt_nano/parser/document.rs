//! Document-level parser responsible for parsing the entire txxt document.

use chumsky::prelude::*;
use std::ops::Range;
use std::sync::Arc;

use crate::txxt_nano::ast::position::SourceLocation;
use crate::txxt_nano::ast::Document;
use crate::txxt_nano::ast::{ContentItem, Definition, List, ListItem, Session, Span, TextContent};
use crate::txxt_nano::lexer::Token;
use crate::txxt_nano::parser::combinators::{
    compute_span_from_optional_spans, definition_subject, list_item_line, session_title, token,
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
pub(crate) fn build_session_parser<P>(
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
pub(crate) fn build_definition_parser<P>(
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
pub(crate) fn build_list_parser<P>(
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

/// Parse a document
///
/// Parses the entire token stream as document content.
/// This function is focused on document-level parsing and delegates to parser.rs
/// for the actual document content parsing logic.
pub fn document(source: &str) -> impl Parser<TokenSpan, Document, Error = ParserError> + Clone {
    super::parser::build_document_content_parser(source).map(|content| Document {
        metadata: Vec::new(),
        content,
        span: None,
    })
}
