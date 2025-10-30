//! Annotation element parsing
//!
//! This module handles parsing of annotation elements in the txxt format.
//! Annotations are marked with :: markers and can appear in single-line or block form.

use chumsky::prelude::*;
use std::ops::Range;
use std::sync::Arc;

use crate::txxt_nano::ast::position::SourceLocation;
use crate::txxt_nano::ast::{Annotation, ContentItem, Label, Location, Paragraph, TextContent};
use crate::txxt_nano::lexer::Token;
use crate::txxt_nano::parser::combinators::{
    annotation_header, compute_span_from_spans, extract_text_from_spans, text_line, token,
};

/// Type alias for token with span
type TokenSpan = (Token, Range<usize>);

/// Type alias for parser error
type ParserError = Simple<TokenSpan>;

/// Helper: convert a byte range to a Span using source location
fn byte_range_to_span(source: &str, range: &Range<usize>) -> Option<Location> {
    if range.start > range.end {
        return None;
    }
    let source_loc = SourceLocation::new(source);
    Some(source_loc.range_to_span(range))
}

/// Build an annotation parser
pub(crate) fn build_annotation_parser<P>(
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
                    let content_spans: Vec<Location> = content
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
            .then(token(Token::Whitespace).ignore_then(text_line()).or_not())
            .then_ignore(token(Token::Newline).or_not())
            .map(move |((label_opt, label_span, parameters), content_span)| {
                let label_text = label_opt.unwrap_or_default();
                let label_position =
                    label_span.and_then(|s| byte_range_to_span(&source_for_single_line, &s));
                let label = Label::new(label_text).with_span(label_position);

                // Handle content if present
                let content = if let Some(spans) = content_span {
                    let text = extract_text_from_spans(&source_for_single_line, &spans);
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

#[cfg(test)]
mod tests {
    use crate::txxt_nano::lexer::lex_with_spans;
    use crate::txxt_nano::parser::api::parse_with_source;
    use crate::txxt_nano::processor::txxt_sources::TxxtSources;

    #[test]
    fn test_annotation_marker_minimal() {
        let source = "Para one. {{paragraph}}\n\n:: note ::\n\nPara two. {{paragraph}}\n";
        let tokens = lex_with_spans(source);
        let doc = parse_with_source(tokens, source).unwrap();

        assert_eq!(doc.content.len(), 3); // paragraph, annotation, paragraph
        assert!(doc.content[1].is_annotation());
    }

    #[test]
    fn test_annotation_single_line() {
        let source = "Para one. {{paragraph}}\n\n:: note :: This is inline text\n\nPara two. {{paragraph}}\n";
        let tokens = lex_with_spans(source);
        let doc = parse_with_source(tokens, source).unwrap();

        assert_eq!(doc.content.len(), 3); // paragraph, annotation, paragraph
        let annotation = doc.content[1].as_annotation().unwrap();
        assert_eq!(annotation.label.value, "note");
        assert_eq!(annotation.content.len(), 1); // One paragraph with inline text
        assert!(annotation.content[0].is_paragraph());
    }

    #[test]
    fn test_verified_annotations_simple() {
        let source = TxxtSources::get_string("120-annotations-simple.txxt")
            .expect("Failed to load sample file");
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Verify document parses successfully and contains expected structure

        // Find and verify :: note :: annotation
        let note_annotation = doc
            .content
            .iter()
            .find(|item| {
                item.as_annotation() //
                    .map(|a| a.label.value == "note")
                    .unwrap_or(false)
            })
            .expect("Should contain :: note :: annotation");
        assert!(note_annotation
            .as_annotation()
            .unwrap()
            .parameters
            .is_empty());
        assert!(note_annotation.as_annotation().unwrap().content.is_empty());

        // Find and verify :: warning severity=high :: annotation
        let warning_annotation = doc
            .content
            .iter()
            .find(|item| {
                item.as_annotation()
                    .map(|a| a.label.value == "warning")
                    .unwrap_or(false)
            })
            .expect("Should contain :: warning :: annotation");
        let warning = warning_annotation.as_annotation().unwrap();
        assert_eq!(warning.parameters.len(), 1);
        assert_eq!(warning.parameters[0].key, "severity");
        assert_eq!(warning.parameters[0].value, Some("high".to_string()));

        // Find and verify :: python.typing :: annotation (namespaced label)
        let python_annotation = doc
            .content
            .iter()
            .find(|item| {
                item.as_annotation()
                    .map(|a| a.label.value.contains("python"))
                    .unwrap_or(false)
            })
            .expect("Should contain :: python.typing :: annotation");
        assert_eq!(
            python_annotation.as_annotation().unwrap().label.value,
            "python.typing"
        );
    }

    #[test]
    fn test_verified_annotations_block_content() {
        let source = TxxtSources::get_string("130-annotations-block-content.txxt")
            .expect("Failed to load sample file");
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Find and verify :: note author="Jane Doe" :: annotation with block content
        let note_annotation = doc
            .content
            .iter()
            .find(|item| {
                item.as_annotation()
                    .map(|a| a.label.value == "note")
                    .unwrap_or(false)
            })
            .expect("Should contain :: note :: annotation");
        let note = note_annotation.as_annotation().unwrap();
        assert_eq!(note.parameters.len(), 2);
        assert_eq!(note.parameters[0].key, "author");
        assert_eq!(note.parameters[0].value, Some("Jane Doe".to_string()));
        assert_eq!(note.parameters[1].key, "date");
        assert_eq!(note.parameters[1].value, Some("2025-01-15".to_string()));
        assert_eq!(note.content.len(), 2); // Two paragraphs
        assert!(note.content[0].is_paragraph());
        assert!(note.content[1].is_paragraph());

        // Find and verify :: warning severity=critical :: annotation with list
        let warning_annotation = doc
            .content
            .iter()
            .find(|item| {
                item.as_annotation()
                    .map(|a| a.label.value == "warning")
                    .unwrap_or(false)
            })
            .expect("Should contain :: warning :: annotation");
        let warning = warning_annotation.as_annotation().unwrap();
        assert_eq!(warning.parameters.len(), 3);
        assert_eq!(warning.parameters[0].key, "severity");
        assert_eq!(warning.parameters[0].value, Some("critical".to_string()));
        assert_eq!(warning.parameters[1].key, "priority");
        assert_eq!(warning.parameters[1].value, Some("high".to_string()));
        assert_eq!(warning.parameters[2].key, "reviewer");
        assert_eq!(warning.parameters[2].value, Some("Alice Smith".to_string()));
        assert_eq!(warning.content.len(), 2); // Paragraph + List
        assert!(warning.content[0].is_paragraph());
        assert!(warning.content[1].is_list());

        // Verify the list has 3 items
        let list = warning.content[1].as_list().unwrap();
        assert_eq!(list.items.len(), 3);
    }
}
