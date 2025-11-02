//! Definition element parsing
//!
//! This module handles parsing of definition elements in the txxt format.
//! Definitions have a subject (label) followed by a colon and indented content.

use chumsky::prelude::*;
use chumsky::primitive::filter;
use std::ops::Range;
use std::sync::Arc;

use super::combinators::{
    aggregate_locations, byte_range_to_location, extract_tokens_to_text_and_location, token,
};
use crate::txxt::ast::{ContentItem, Definition, TextContent};
use crate::txxt::lexers::Token;

/// Type alias for token with location
type TokenLocation = (Token, Range<usize>);

/// Type alias for parser error
type ParserError = Simple<TokenLocation>;

/// Parse a definition subject
/// Phase 5: Now returns extracted text with location information
pub(crate) fn definition_subject(
    source: Arc<String>,
) -> impl Parser<TokenLocation, (String, Range<usize>), Error = ParserError> + Clone {
    filter(|(t, _location): &TokenLocation| !matches!(t, Token::Colon | Token::Newline))
        .repeated()
        .at_least(1)
        .map(move |tokenss| extract_tokens_to_text_and_location(&source, tokenss))
        .then_ignore(filter(|(t, _): &TokenLocation| matches!(t, Token::Colon)).ignored())
        .then_ignore(filter(|(t, _): &TokenLocation| matches!(t, Token::Newline)).ignored())
}

/// Build a definition parser
pub(crate) fn build_definition_parser<P>(
    source: Arc<String>,
    items: P,
) -> impl Parser<TokenLocation, ContentItem, Error = ParserError> + Clone
where
    P: Parser<TokenLocation, Vec<ContentItem>, Error = ParserError> + Clone + 'static,
{
    let source_for_definition = source.clone();
    definition_subject(source.clone())
        .then(
            token(Token::IndentLevel)
                .ignore_then(items)
                .then_ignore(token(Token::DedentLevel)),
        )
        .map(move |((subject_text, subject_location), content)| {
            let subject_location =
                byte_range_to_location(&source_for_definition, &subject_location);
            let subject = TextContent::from_string(subject_text, Some(subject_location));

            let location = aggregate_locations(subject_location, &content);

            ContentItem::Definition(Definition {
                subject,
                content,
                location,
            })
        })
}

#[cfg(test)]
mod tests {
    use crate::txxt::ast::Container;
    use crate::txxt::ast::ContentItem;
    use crate::txxt::lexers::lex;
    use crate::txxt::parser::reference::api::parse;
    use crate::txxt::processor::txxt_sources::TxxtSources;
    use crate::txxt::testing::assert_ast;

    #[test]
    fn test_unified_recursive_parser_simple() {
        // Minimal test for the unified recursive parser
        let source = "First paragraph\n\nDefinition:\n    Content of definition\n";
        let tokens = lex(source);
        println!("Testing simple definition with unified parser:");
        println!("Source: {:?}", source);

        let result = parse(tokens, source);
        if let Err(ref e) = result {
            println!("Parse error: {:?}", e);
        }
        assert!(result.is_ok(), "Failed to parse simple definition");
        let doc = result.unwrap();
        assert_eq!(
            doc.root.content.len(),
            2,
            "Should have paragraph and definition"
        );
    }

    #[test]
    fn test_unified_recursive_nested_definitions() {
        // Test nested definitions with the unified parser
        let source = "Outer:\n    Inner:\n        Nested content\n";
        let tokens = lex(source);
        println!("Testing nested definitions with unified parser:");
        println!("Source: {:?}", source);

        let result = parse(tokens, source);
        if let Err(ref e) = result {
            println!("Parse error: {:?}", e);
        }
        assert!(result.is_ok(), "Failed to parse nested definitions");

        let doc = result.unwrap();
        assert_eq!(
            doc.root.content.len(),
            1,
            "Should have one outer definition"
        );

        // Check outer definition
        let outer_def = doc.root.content[0]
            .as_definition()
            .expect("Should be a definition");
        assert_eq!(outer_def.label(), "Outer");
        assert_eq!(
            outer_def.content.len(),
            1,
            "Outer should have one inner item"
        );

        // Check inner definition
        let inner_def = outer_def.content[0]
            .as_definition()
            .expect("Inner should be a definition");
        assert_eq!(inner_def.label(), "Inner");
        assert_eq!(inner_def.content.len(), 1, "Inner should have content");

        // Check nested content
        let nested_para = inner_def.content[0]
            .as_paragraph()
            .expect("Should be a paragraph");
        if let ContentItem::TextLine(tl) = &nested_para.lines[0] {
            assert_eq!(tl.text(), "Nested content");
        } else {
            panic!("Expected TextLine");
        }
    }

    #[test]
    // Previously ignored for issue #35 - now testing if fixed
    fn test_unified_parser_paragraph_then_definition() {
        // Test paragraph followed by definition - similar to failing test
        let source = "Simple paragraph\n\nAnother paragraph\n\nFirst Definition:\n    Definition content\n\nSecond Definition:\n    More content\n";
        let tokens = lex(source);
        println!("Testing paragraph then definition:");
        println!("Source: {:?}", source);

        let result = parse(tokens, source);
        if let Err(ref e) = result {
            println!("Parse error: {:?}", e);
            println!("Error at span: {:?}", &source[e[0].span().clone()]);
        }
        assert!(result.is_ok(), "Failed to parse paragraph then definition");

        let doc = result.unwrap();
        println!("Parsed {} items", doc.root.content.len());
        for (i, item) in doc.root.content.iter().enumerate() {
            match item {
                ContentItem::Paragraph(p) => {
                    println!("  Item {}: Paragraph with {} lines", i, p.lines.len())
                }
                ContentItem::Definition(d) => {
                    println!("  Item {}: Definition '{}'", i, d.subject.as_string())
                }
                _ => println!("  Item {}: Other", i),
            }
        }
        assert_eq!(
            doc.root.content.len(),
            4,
            "Should have 2 paragraphs and 2 definitions"
        );
    }

    #[test]
    // Previously ignored for issue #35 - now testing if fixed
    fn test_verified_definitions_simple() {
        let source = TxxtSources::get_string("090-definitions-simple.txxt")
            .expect("Failed to load sample file");
        let tokens = lex(&source);

        // Debug: print first few tokens
        println!("First 10 tokens:");
        for (i, token) in tokens.iter().take(10).enumerate() {
            println!("  {}: {:?}", i, token);
        }

        let result = parse(tokens, &source);
        if let Err(ref e) = result {
            println!("Parse error: {:?}", e);
        }
        let doc = result.unwrap();

        // Item 0-1: Opening paragraphs
        assert_ast(&doc)
            .item(0, |item| {
                item.assert_paragraph() // "Simple Definitions Test"
                    .text_contains("Simple Definitions Test");
            })
            .item(1, |item| {
                item.assert_paragraph() // "basic Definition element"
                    .text_contains("basic Definition element");
            });

        // Item 2: First Definition
        assert_ast(&doc).item(2, |item| {
            item.assert_definition()
                .subject("First Definition")
                .child_count(1)
                .child(0, |child| {
                    child
                        .assert_paragraph()
                        .text_contains("content of the first definition");
                });
        });

        // Item 3: Second Definition
        assert_ast(&doc).item(3, |item| {
            item.assert_definition()
                .subject("Second Definition")
                .child_count(1)
                .child(0, |child| {
                    child
                        .assert_paragraph()
                        .text_contains("content that explains the second term");
                });
        });

        // Item 4: Glossary Term (with multiple paragraphs)
        assert_ast(&doc).item(4, |item| {
            item.assert_definition()
                .subject("Glossary Term")
                .child_count(2)
                .child(0, |child| {
                    child
                        .assert_paragraph()
                        .text_contains("word or phrase that needs explanation");
                })
                .child(1, |child| {
                    child
                        .assert_paragraph()
                        .text_contains("definitions can have complex content");
                });
        });

        // Item 5: API Endpoint
        assert_ast(&doc).item(5, |item| {
            item.assert_definition()
                .subject("API Endpoint")
                .child_count(1)
                .child(0, |child| {
                    child.assert_paragraph().text_contains("specific URL path");
                });
        });

        // Item 6: Regular paragraph
        assert_ast(&doc).item(6, |item| {
            item.assert_paragraph()
                .text_contains("Regular paragraph after definitions");
        });

        // Item 7: Another Term
        assert_ast(&doc).item(7, |item| {
            item.assert_definition()
                .subject("Another Term")
                .child_count(1)
                .child(0, |child| {
                    child
                        .assert_paragraph()
                        .text_contains("appear anywhere in the document");
                });
        });

        // Item 8: Final paragraph
        assert_ast(&doc).item(8, |item| {
            item.assert_paragraph().text_contains("Final paragraph");
        });
    }

    #[test]
    fn test_verified_definitions_mixed_content() {
        let source = TxxtSources::get_string("100-definitions-mixed-content.txxt")
            .expect("Failed to load sample file");
        let tokens = lex(&source);
        let doc = parse(tokens, &source).unwrap();

        // Item 0-1: Opening paragraphs
        assert_ast(&doc)
            .item(0, |item| {
                item.assert_paragraph() // "Definitions with Mixed Content Test"
                    .text_contains("Definitions with Mixed Content Test");
            })
            .item(1, |item| {
                item.assert_paragraph() // "both paragraphs and lists"
                    .text_contains("both paragraphs and lists");
            });

        // Item 2: Programming Language (paragraph + list)
        assert_ast(&doc).item(2, |item| {
            item.assert_definition()
                .subject("Programming Language")
                .child_count(2)
                .child(0, |child| {
                    child
                        .assert_paragraph()
                        .text_contains("formal language comprising");
                })
                .child(1, |child| {
                    child.assert_list().item_count(3);
                });
        });

        // Item 3: HTTP Methods (list only)
        assert_ast(&doc).item(3, |item| {
            item.assert_definition()
                .subject("HTTP Methods")
                .child_count(1)
                .child(0, |child| {
                    child.assert_list().item_count(4);
                });
        });

        // Item 4: Data Structure (paragraph + 2 lists)
        assert_ast(&doc).item(4, |item| {
            item.assert_definition()
                .subject("Data Structure")
                .child_count(3)
                .child(0, |child| {
                    child
                        .assert_paragraph()
                        .text_contains("organizing and storing data");
                })
                .child(1, |child| {
                    child.assert_list().item_count(4);
                })
                .child(2, |child| {
                    child.assert_list().item_count(3);
                });
        });

        // Item 5: Regular paragraph
        assert_ast(&doc).item(5, |item| {
            item.assert_paragraph()
                .text_contains("Regular paragraph between definitions");
        });

        // Item 6: Design Pattern (paragraph + 3 lists)
        assert_ast(&doc).item(6, |item| {
            item.assert_definition()
                .subject("Design Pattern")
                .child_count(4)
                .child(0, |child| {
                    child.assert_paragraph().text_contains("reusable solution");
                })
                .child(1, |child| {
                    child.assert_list().item_count(3);
                })
                .child(2, |child| {
                    child.assert_list().item_count(3);
                })
                .child(3, |child| {
                    child.assert_list().item_count(3);
                });
        });

        // Item 7: End paragraph
        assert_ast(&doc).item(7, |item| {
            item.assert_paragraph().text_contains("End of document");
        });
    }
}
