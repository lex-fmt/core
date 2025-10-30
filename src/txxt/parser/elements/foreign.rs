//! Foreign block element parsing
//!
//! This module handles parsing of foreign block elements in the txxt format.
//! Foreign blocks are used for code blocks, images, and other non-txxt content.

use chumsky::prelude::*;
use std::ops::Range;
use std::sync::Arc;

use crate::txxt::ast::location::SourceLocation;
use crate::txxt::ast::{
    Annotation, ContentItem, ForeignBlock, Label, Location, Paragraph, TextContent,
};
use crate::txxt::lexer::Token;
use crate::txxt::parser::combinators::{
    compute_byte_range_bounds, compute_location_from_optional_locations,
    extract_text_from_locations, text_line, token,
};
use crate::txxt::parser::elements::annotations::{annotation_header, AnnotationHeader};
use crate::txxt::parser::elements::definitions::definition_subject;

/// Type alias for token with location
type TokenLocation = (Token, Range<usize>);

/// Type alias for parser error
type ParserError = Simple<TokenLocation>;

/// Helper: convert a byte range to a location using source location
fn byte_range_to_location(source: &str, range: &Range<usize>) -> Option<Location> {
    if range.start > range.end {
        return None;
    }
    let source_loc = SourceLocation::new(source);
    Some(source_loc.range_to_location(range))
}

/// Parse a foreign block
/// Phase 4: Now builds final ForeignBlock type directly
pub(crate) fn foreign_block(
    source: Arc<String>,
) -> impl Parser<TokenLocation, ForeignBlock, Error = ParserError> + Clone {
    let subject_parser = definition_subject(source.clone());

    // Parse content that handles nested indentation structures.
    // Content can be either:
    // 1. Regular tokens (not TxxtMarker, not DedentLevel)
    // 2. Nested indentation blocks: IndentLevel + [content] + DedentLevel
    //
    // This ensures that nested structures (like code with braces) are properly
    // consumed, and DedentLevel tokens mark clear boundaries. This approach
    // mirrors how lists.rs handles nested content with the `items` parser.
    let with_content = token(Token::IndentLevel)
        .ignore_then(recursive(|nested_content| {
            choice((
                // Handle nested indentation: properly matched pairs
                // Return a dummy TokenLocation to match the filter branch type
                token(Token::IndentLevel)
                    .ignore_then(nested_content.clone())
                    .then_ignore(token(Token::DedentLevel))
                    .map(|_| (Token::IndentLevel, 0..0)), // Dummy token, won't be used
                // Regular content token (not TxxtMarker, not DedentLevel)
                filter(|(t, _location): &TokenLocation| {
                    !matches!(t, Token::TxxtMarker | Token::DedentLevel)
                }),
            ))
            .repeated()
            .at_least(1)
        }))
        .then_ignore(token(Token::DedentLevel))
        .map(|tokens: Vec<TokenLocation>| {
            tokens
                .into_iter()
                .map(|(_, s)| s)
                .filter(|s| s.start < s.end) // Filter out dummy ranges (0..0)
                .collect::<Vec<_>>()
        });

    let source_for_annotation = source.clone();
    let closing_annotation_parser = token(Token::TxxtMarker)
        .ignore_then(annotation_header(source_for_annotation.clone()))
        .then_ignore(token(Token::TxxtMarker))
        .then(token(Token::Whitespace).ignore_then(text_line()).or_not())
        .map(move |(header_info, content_location)| {
            let AnnotationHeader {
                label,
                label_range,
                parameters,
                header_range,
            } = header_info;

            let label_text = label.unwrap_or_default();
            let label_location = label_range
                .and_then(|range| byte_range_to_location(&source_for_annotation, &range));
            let label = Label::new(label_text).with_location(label_location);

            let (content, paragraph_location) = if let Some(locations) = content_location {
                let text = extract_text_from_locations(&source_for_annotation, &locations);
                let range = compute_byte_range_bounds(&locations);
                let paragraph_location = byte_range_to_location(&source_for_annotation, &range);
                let text_content = TextContent::from_string(text, paragraph_location);
                let paragraph = Paragraph {
                    lines: vec![text_content],
                    location: paragraph_location,
                };
                (vec![ContentItem::Paragraph(paragraph)], paragraph_location)
            } else {
                (vec![], None)
            };

            let header_location = byte_range_to_location(&source_for_annotation, &header_range);

            let location_sources = vec![header_location, label_location, paragraph_location];
            let location = compute_location_from_optional_locations(&location_sources);

            Annotation {
                label,
                parameters,
                content,
                location,
            }
        });

    subject_parser
        .then_ignore(token(Token::BlankLine).repeated())
        .then(with_content.or_not())
        .then(closing_annotation_parser)
        .then_ignore(token(Token::Newline).or_not())
        .map(
            move |(((subject_text, subject_location), content_locations), closing_annotation)| {
                let subject_location = byte_range_to_location(&source, &subject_location);
                let subject = TextContent::from_string(subject_text, subject_location);

                let (content_text, content_location) = if let Some(locations) = content_locations {
                    if locations.is_empty() {
                        (String::new(), None)
                    } else {
                        let text = extract_text_from_locations(&source, &locations);
                        let range = compute_byte_range_bounds(&locations);
                        let location = byte_range_to_location(&source, &range);
                        (text, location)
                    }
                } else {
                    (String::new(), None)
                };

                let content = TextContent::from_string(content_text, content_location);
                let location_sources = vec![
                    subject_location,
                    content_location,
                    closing_annotation.location,
                ];
                let location = compute_location_from_optional_locations(&location_sources);

                ForeignBlock {
                    subject,
                    content,
                    closing_annotation,
                    location,
                }
            },
        )
}

#[cfg(test)]
mod tests {
    use crate::txxt::lexer::lex_with_locations;
    use crate::txxt::parser::api::parse_with_source;
    use crate::txxt::processor::txxt_sources::TxxtSources;

    #[test]
    fn test_foreign_block_simple_with_content() {
        let source = "Code Example:\n    function hello() {\n        return \"world\";\n    }\n:: javascript caption=\"Hello World\" ::\n\n";
        let tokens = lex_with_locations(source);
        println!("Tokens: {:?}", tokens);
        let doc = parse_with_source(tokens, source).unwrap();

        assert_eq!(doc.content.len(), 1);
        let foreign_block = doc.content[0].as_foreign_block().unwrap();
        assert_eq!(foreign_block.subject.as_string(), "Code Example");
        assert!(foreign_block
            .content
            .as_string()
            .contains("function hello()"));
        assert!(foreign_block
            .content
            .as_string()
            .contains("return \"world\""));
        assert_eq!(foreign_block.closing_annotation.label.value, "javascript");
        assert_eq!(foreign_block.closing_annotation.parameters.len(), 1);
        assert_eq!(
            foreign_block.closing_annotation.parameters[0].key,
            "caption"
        );
        assert_eq!(
            foreign_block.closing_annotation.parameters[0].value,
            Some("Hello World".to_string())
        );
    }

    #[test]
    fn test_foreign_block_marker_form() {
        let source = "Image Reference:\n\n:: image type=jpg, src=sunset.jpg :: As the sun sets, we see a colored sea bed.\n\n";
        let tokens = lex_with_locations(source);
        let doc = parse_with_source(tokens, source).unwrap();

        assert_eq!(doc.content.len(), 1);
        let foreign_block = doc.content[0].as_foreign_block().unwrap();
        assert_eq!(foreign_block.subject.as_string(), "Image Reference");
        assert_eq!(foreign_block.content.as_string(), ""); // No content in marker form
        assert_eq!(foreign_block.closing_annotation.label.value, "image");
        assert_eq!(foreign_block.closing_annotation.parameters.len(), 2);
        assert_eq!(foreign_block.closing_annotation.parameters[0].key, "type");
        assert_eq!(
            foreign_block.closing_annotation.parameters[0].value,
            Some("jpg".to_string())
        );
        assert_eq!(foreign_block.closing_annotation.parameters[1].key, "src");
        assert_eq!(
            foreign_block.closing_annotation.parameters[1].value,
            Some("sunset.jpg".to_string())
        );
    }

    #[test]
    fn test_foreign_block_preserves_whitespace() {
        let source = "Indented Code:\n\n    // This has    multiple    spaces\n    const regex = /[a-z]+/g;\n    \n    console.log(\"Hello, World!\");\n\n:: javascript ::\n\n";
        let tokens = lex_with_locations(source);
        let doc = parse_with_source(tokens, source).unwrap();

        let foreign_block = doc.content[0].as_foreign_block().unwrap();
        assert!(foreign_block
            .content
            .as_string()
            .contains("    multiple    spaces")); // Preserves multiple spaces
        assert!(foreign_block.content.as_string().contains("    \n")); // Preserves blank lines
    }

    #[test]
    fn test_foreign_block_multiple_blocks() {
        // Fixed by reordering parsers: foreign_block before session
        // Since foreign blocks have stricter requirements (must have closing annotation),
        // trying them first resolves the ambiguity

        let source = "First Block:\n\n    code1\n\n:: lang1 ::\n\nSecond Block:\n\n    code2\n\n:: lang2 ::\n\n";
        let tokens = lex_with_locations(source);
        let doc = parse_with_source(tokens, source).unwrap();

        assert_eq!(doc.content.len(), 2);

        let first_block = doc.content[0].as_foreign_block().unwrap();
        assert_eq!(first_block.subject.as_string(), "First Block");
        assert!(first_block.content.as_string().contains("code1"));
        assert_eq!(first_block.closing_annotation.label.value, "lang1");

        let second_block = doc.content[1].as_foreign_block().unwrap();
        assert_eq!(second_block.subject.as_string(), "Second Block");
        assert!(second_block.content.as_string().contains("code2"));
        assert_eq!(second_block.closing_annotation.label.value, "lang2");
    }

    #[test]
    fn test_foreign_block_with_paragraphs() {
        let source = "Intro paragraph.\n\nCode Block:\n\n    function test() {\n        return true;\n    }\n\n:: javascript ::\n\nOutro paragraph.\n\n";
        let tokens = lex_with_locations(source);
        let doc = parse_with_source(tokens, source).unwrap();

        assert_eq!(doc.content.len(), 3);
        assert!(doc.content[0].is_paragraph());
        assert!(doc.content[1].is_foreign_block());
        assert!(doc.content[2].is_paragraph());
    }

    #[test]
    fn test_verified_foreign_blocks_simple() {
        let source = TxxtSources::get_string("140-foreign-blocks-simple.txxt")
            .expect("Failed to load sample file");
        let tokens = lex_with_locations(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Find JavaScript code block
        let js_block = doc
            .content
            .iter()
            .find(|item| {
                item.as_foreign_block()
                    .map(|fb| fb.closing_annotation.label.value == "javascript")
                    .unwrap_or(false)
            })
            .expect("Should contain JavaScript foreign block");
        let js = js_block.as_foreign_block().unwrap();
        assert_eq!(js.subject.as_string(), "Code Example");
        assert!(js.content.as_string().contains("function hello()"));
        assert!(js.content.as_string().contains("console.log"));
        assert_eq!(js.closing_annotation.parameters.len(), 1);
        assert_eq!(js.closing_annotation.parameters[0].key, "caption");

        // Find Python code block
        let py_block = doc
            .content
            .iter()
            .find(|item| {
                item.as_foreign_block()
                    .map(|fb| fb.closing_annotation.label.value == "python")
                    .unwrap_or(false)
            })
            .expect("Should contain Python foreign block");
        let py = py_block.as_foreign_block().unwrap();
        assert_eq!(py.subject.as_string(), "Another Code Block");
        assert!(py.content.as_string().contains("def fibonacci"));
        assert!(py.content.as_string().contains("for i in range"));

        // Find SQL block
        let sql_block = doc
            .content
            .iter()
            .find(|item| {
                item.as_foreign_block()
                    .map(|fb| fb.closing_annotation.label.value == "sql")
                    .unwrap_or(false)
            })
            .expect("Should contain SQL foreign block");
        let sql = sql_block.as_foreign_block().unwrap();
        assert_eq!(sql.subject.as_string(), "SQL Example");
        assert!(sql.content.as_string().contains("SELECT"));
        assert!(sql.content.as_string().contains("FROM users"));
    }

    #[test]
    fn test_verified_foreign_blocks_no_content() {
        let source = TxxtSources::get_string("150-foreign-blocks-no-content.txxt")
            .expect("Failed to load sample file");
        let tokens = lex_with_locations(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Find image reference
        let image_block = doc
            .content
            .iter()
            .find(|item| {
                item.as_foreign_block()
                    .map(|fb| fb.closing_annotation.label.value == "image")
                    .unwrap_or(false)
            })
            .expect("Should contain image foreign block");
        let image = image_block.as_foreign_block().unwrap();
        assert_eq!(image.subject.as_string(), "Image Reference");
        assert_eq!(image.content.as_string(), ""); // No content in marker form
        assert_eq!(image.closing_annotation.parameters.len(), 2);
        assert_eq!(image.closing_annotation.parameters[0].key, "type");
        assert_eq!(
            image.closing_annotation.parameters[0].value,
            Some("jpg".to_string())
        );

        // Find binary file reference
        let binary_block = doc
            .content
            .iter()
            .find(|item| {
                item.as_foreign_block()
                    .map(|fb| fb.closing_annotation.label.value == "binary")
                    .unwrap_or(false)
            })
            .expect("Should contain binary foreign block");
        let binary = binary_block.as_foreign_block().unwrap();
        assert_eq!(binary.subject.as_string(), "Binary File Reference");
        assert_eq!(binary.content.as_string(), "");
        assert_eq!(binary.closing_annotation.parameters.len(), 2);
        assert_eq!(binary.closing_annotation.parameters[0].key, "type");
        assert_eq!(
            binary.closing_annotation.parameters[0].value,
            Some("pdf".to_string())
        );
    }
}
