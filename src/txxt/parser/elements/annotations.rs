//! Annotation element parsing
//!
//! This module handles parsing of annotation elements in the txxt format.
//! Annotations are marked with :: markers and can appear in single-line or block form.

use chumsky::prelude::*;
use chumsky::primitive::filter;
use std::ops::Range;
use std::sync::Arc;

use crate::txxt::ast::{Annotation, AstNode, ContentItem, Label, Location, Paragraph, TextContent};
use crate::txxt::lexer::Token;
use crate::txxt::parser::combinators::{
    byte_range_to_location, compute_byte_range_bounds, compute_location_from_locations,
    extract_text_from_locations, text_line, token,
};
use crate::txxt::parser::elements::labels::parse_label_from_tokens;
use crate::txxt::parser::elements::parameters::{convert_parameter, parse_parameters_from_tokens};

/// Type alias for token with location
type TokenLocation = (Token, Range<usize>);

/// Type alias for parser error
type ParserError = Simple<TokenLocation>;

/// Parse the bounded region between :: markers
/// Phase 5: Now returns extracted label text, label location, and final Parameter types
#[derive(Clone, Debug)]
pub(crate) struct AnnotationHeader {
    pub label: Option<String>,
    pub label_range: Option<Range<usize>>,
    pub parameters: Vec<crate::txxt::ast::Parameter>,
    pub header_range: Range<usize>,
}

pub(crate) fn annotation_header(
    source: Arc<String>,
) -> impl Parser<TokenLocation, AnnotationHeader, Error = ParserError> + Clone {
    let bounded_region =
        filter(|(t, _): &TokenLocation| !matches!(t, Token::TxxtMarker | Token::Newline))
            .repeated()
            .at_least(1);

    bounded_region.validate(move |tokens, location, emit| {
        if tokens.is_empty() {
            emit(ParserError::expected_input_found(location, None, None));
            return AnnotationHeader {
                label: None,
                label_range: None,
                parameters: Vec::new(),
                header_range: 0..0,
            };
        }

        let (label_location, mut i) = parse_label_from_tokens(&tokens);

        if label_location.is_none() && i == 0 {
            while i < tokens.len() && matches!(tokens[i].0, Token::Whitespace) {
                i += 1;
            }
        }

        let paramss = parse_parameters_from_tokens(&tokens[i..]);

        let header_range_start = tokens.first().map(|(_, span)| span.start).unwrap_or(0);
        let header_range_end = tokens
            .last()
            .map(|(_, span)| span.end)
            .unwrap_or(header_range_start);
        let header_range = header_range_start..header_range_end;

        // Extract label text if present
        let label = label_location.as_ref().map(|location| {
            let text = if location.start < location.end && location.end <= source.len() {
                source[location.start..location.end].trim().to_string()
            } else {
                String::new()
            };
            text
        });

        // Convert parameters to final types
        let params = paramss
            .into_iter()
            .map(|p| convert_parameter(&source, p))
            .collect();

        AnnotationHeader {
            label,
            label_range: label_location,
            parameters: params,
            header_range,
        }
    })
}

/// Build an annotation parser
pub(crate) fn build_annotation_parser<P>(
    source: Arc<String>,
    items: P,
) -> impl Parser<TokenLocation, ContentItem, Error = ParserError> + Clone
where
    P: Parser<TokenLocation, Vec<ContentItem>, Error = ParserError> + Clone + 'static,
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
            .map(move |(header_info, content)| {
                let AnnotationHeader {
                    label,
                    label_range,
                    parameters,
                    header_range,
                } = header_info;

                let label_text = label.unwrap_or_default();
                let label_location = label_range.map_or(Location::default(), |s| {
                    byte_range_to_location(&source_for_block, &s)
                });
                let label = Label::new(label_text).at(label_location);

                let header_location = byte_range_to_location(&source_for_block, &header_range);

                // Collect locations from header and content to compute overall annotation span
                let mut location_sources: Vec<Location> = vec![header_location];
                location_sources.extend(
                    content
                        .iter()
                        .map(|item| item.location().unwrap_or_default()),
                );
                location_sources.push(label_location);
                let location = compute_location_from_locations(&location_sources);

                ContentItem::Annotation(Annotation {
                    label,
                    parameters,
                    content,
                    location,
                })
            })
    };

    let single_line_or_marker = {
        let source_for_single_line = source.clone();
        let header_for_single = header.clone();
        header_for_single
            .then(token(Token::Whitespace).ignore_then(text_line()).or_not())
            .then_ignore(token(Token::Newline).or_not())
            .map(move |(header_info, content_location)| {
                let AnnotationHeader {
                    label,
                    label_range,
                    parameters,
                    header_range,
                } = header_info;

                let label_text = label.unwrap_or_default();
                let label_location = label_range.map_or(Location::default(), |s| {
                    byte_range_to_location(&source_for_single_line, &s)
                });
                let label = Label::new(label_text).at(label_location);

                // Handle content if present and compute paragraph location
                let (content, paragraph_location) = if let Some(locations) = content_location {
                    let text = extract_text_from_locations(&source_for_single_line, &locations);
                    let range = compute_byte_range_bounds(&locations);
                    let paragraph_location =
                        byte_range_to_location(&source_for_single_line, &range);
                    let text_content = TextContent::from_string(text, Some(paragraph_location));
                    let text_line = crate::txxt::ast::TextLine::new(text_content)
                        .with_location(paragraph_location);
                    let paragraph = Paragraph {
                        lines: vec![ContentItem::TextLine(text_line)],
                        location: paragraph_location,
                    };
                    (vec![ContentItem::Paragraph(paragraph)], paragraph_location)
                } else {
                    (vec![], Location::default())
                };

                let header_location =
                    byte_range_to_location(&source_for_single_line, &header_range);

                let location_sources = vec![header_location, label_location, paragraph_location];
                let location = compute_location_from_locations(&location_sources);

                ContentItem::Annotation(Annotation {
                    label,
                    parameters,
                    content,
                    location,
                })
            })
    };

    block_form.or(single_line_or_marker)
}

#[cfg(test)]
mod tests {
    use crate::txxt::lexer::lex;
    use crate::txxt::parser::api::parse;
    use crate::txxt::processor::txxt_sources::TxxtSources;

    #[test]
    fn test_annotation_marker_minimal() {
        let source = "Para one. {{paragraph}}\n\n:: note ::\n\nPara two. {{paragraph}}\n";
        let tokens = lex(source);
        let doc = parse(tokens, source).unwrap();

        assert_eq!(doc.root.content.len(), 3); // paragraph, annotation, paragraph
        assert!(doc.root.content[1].is_annotation());
    }

    #[test]
    fn test_annotation_single_line() {
        let source = "Para one. {{paragraph}}\n\n:: note :: This is inline text\n\nPara two. {{paragraph}}\n";
        let tokens = lex(source);
        let doc = parse(tokens, source).unwrap();

        assert_eq!(doc.root.content.len(), 3); // paragraph, annotation, paragraph
        let annotation = doc.root.content[1].as_annotation().unwrap();
        assert_eq!(annotation.label.value, "note");
        assert_eq!(annotation.content.len(), 1); // One paragraph with inline text
        assert!(annotation.content[0].is_paragraph());
    }

    #[test]
    fn test_verified_annotations_simple() {
        let source = TxxtSources::get_string("120-annotations-simple.txxt")
            .expect("Failed to load sample file");
        let tokens = lex(&source);
        let doc = parse(tokens, &source).unwrap();

        // Verify document parses successfully and contains expected structure

        // Find and verify :: note :: annotation
        let note_annotation = doc
            .root
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
            .root
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
        assert_eq!(warning.parameters[0].value, "high".to_string());

        // Find and verify :: python.typing :: annotation (namespaced label)
        let python_annotation = doc
            .root
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
        let tokens = lex(&source);
        let doc = parse(tokens, &source).unwrap();

        // Find and verify :: note author="Jane Doe" :: annotation with block content
        let note_annotation = doc
            .root
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
        assert_eq!(note.parameters[0].value, "Jane Doe".to_string());
        assert_eq!(note.parameters[1].key, "date");
        assert_eq!(note.parameters[1].value, "2025-01-15".to_string());
        assert_eq!(note.content.len(), 2); // Two paragraphs
        assert!(note.content[0].is_paragraph());
        assert!(note.content[1].is_paragraph());

        // Find and verify :: warning severity=critical :: annotation with list
        let warning_annotation = doc
            .root
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
        assert_eq!(warning.parameters[0].value, "critical".to_string());
        assert_eq!(warning.parameters[1].key, "priority");
        assert_eq!(warning.parameters[1].value, "high".to_string());
        assert_eq!(warning.parameters[2].key, "reviewer");
        assert_eq!(warning.parameters[2].value, "Alice Smith".to_string());
        assert_eq!(warning.content.len(), 2); // Paragraph + List
        assert!(warning.content[0].is_paragraph());
        assert!(warning.content[1].is_list());

        // Verify the list has 3 items
        let list = warning.content[1].as_list().unwrap();
        assert_eq!(list.content.len(), 3);
    }
}
