//! Foreign block element parsing
//!
//! This module handles parsing of foreign block elements in the txxt format.
//! Foreign blocks are used for code blocks, images, and other non-txxt content.

use chumsky::prelude::*;
use std::ops::Range;
use std::sync::Arc;

use crate::txxt_nano::ast::{Annotation, ContentItem, ForeignBlock, Label, Paragraph, TextContent};
use crate::txxt_nano::lexer::Token;
use crate::txxt_nano::parser::combinators::{
    annotation_header, definition_subject, extract_text_from_spans, text_line, token,
};

/// Type alias for token with span
type TokenSpan = (Token, Range<usize>);

/// Type alias for parser error
type ParserError = Simple<TokenSpan>;

/// Parse a foreign block
/// Phase 4: Now builds final ForeignBlock type directly
pub(crate) fn foreign_block(
    source: Arc<String>,
) -> impl Parser<TokenSpan, ForeignBlock, Error = ParserError> + Clone {
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
                // Return a dummy TokenSpan to match the filter branch type
                token(Token::IndentLevel)
                    .ignore_then(nested_content.clone())
                    .then_ignore(token(Token::DedentLevel))
                    .map(|_| (Token::IndentLevel, 0..0)), // Dummy token, won't be used
                // Regular content token (not TxxtMarker, not DedentLevel)
                filter(|(t, _span): &TokenSpan| {
                    !matches!(t, Token::TxxtMarker | Token::DedentLevel)
                }),
            ))
            .repeated()
            .at_least(1)
        }))
        .then_ignore(token(Token::DedentLevel))
        .map(|tokens: Vec<TokenSpan>| {
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

#[cfg(test)]
mod tests {
    use crate::txxt_nano::lexer::lex_with_locations;
    use crate::txxt_nano::parser::api::parse_with_source;
    use crate::txxt_nano::processor::txxt_sources::TxxtSources;

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
