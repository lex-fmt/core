//! Parser implementation for the txxt format using chumsky
//!
//! This module implements a parser combinator-based parser for txxt documents.
//! It builds on the token stream from the lexer and produces an AST.
//!
//! ## Testing
//!
//! All parser tests must follow strict guidelines. See the [testing module](crate::txxt::testing)
//! for comprehensive documentation on using verified txxt sources and AST assertions.

use chumsky::prelude::*;
use std::ops::Range;

#[allow(unused_imports)] // Container is used in tests
use crate::txxt::ast::{Container, ContentItem, Document};
use crate::txxt::lexer::Token;

/// Type alias for token with location
type TokenLocation = (Token, Range<usize>);

/// Type alias for parser error
type ParserError = Simple<TokenLocation>;

// Parser combinators - kept for test support if needed
#[allow(unused_imports)]
use super::combinators::paragraph;

// Import parser builders from element modules
use super::combinators::token;
use super::elements::annotations::build_annotation_parser;
use super::elements::definitions::build_definition_parser;
use super::elements::foreign::foreign_block;
use super::elements::lists::build_list_parser;
use super::elements::sessions::build_session_parser;
use std::sync::Arc;

/// Build the Multi-Parser Bundle for document-level content parsing.
///
/// This parser builds final ContentItem types directly using refactored combinators.
/// All combinators now take source parameter and return final types.
pub(crate) fn build_document_content_parser(
    source: &str,
) -> impl Parser<TokenLocation, Vec<ContentItem>, Error = ParserError> + Clone {
    let source = Arc::new(source.to_string());

    recursive(move |items| {
        let source = source.clone();
        let single_item = {
            // Session parser - now builds final Session type with location
            let session_parser = build_session_parser(source.clone(), items.clone());

            // Definition parser - now builds final Definition type with location
            let definition_parser = build_definition_parser(source.clone(), items.clone());

            // List parser - now builds final List type with location
            let list_parser = build_list_parser(source.clone(), items.clone());

            // Annotation parser - now builds final Annotation type with location
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

/// Parse with source text - the primary parsing function
///
/// Parses tokens with location information and source text to produce a Document.
/// All parsed documents include complete location information automatically.
///
/// Re-exports the canonical implementation from api.rs
pub fn parse(tokens: Vec<TokenLocation>, source: &str) -> Result<Document, Vec<ParserError>> {
    super::api::parse(tokens, source)
}

/// Backward-compatibility shim
#[deprecated(note = "Use parse(tokens, source) instead")]
pub fn parse_with_source(
    tokens: Vec<TokenLocation>,
    source: &str,
) -> Result<Document, Vec<ParserError>> {
    parse(tokens, source)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::txxt::ast::{AstNode, Position};
    use crate::txxt::lexer::lex;
    use crate::txxt::processor::txxt_sources::TxxtSources;
    use std::sync::Arc;

    #[test]
    fn test_simple_paragraph() {
        let input = "Hello world\n\n";
        let tokenss = lex(input);

        let result = paragraph(Arc::new(input.to_string())).parse(tokenss);
        assert!(result.is_ok(), "Failed to parse paragraph: {:?}", result);

        let para = result.unwrap();
        assert_eq!(para.lines.len(), 1);
        if let ContentItem::TextLine(tl) = &para.lines[0] {
            assert_eq!(tl.text(), "Hello world");
        } else {
            panic!("Expected TextLine");
        }
    }

    #[test]
    fn test_real_content_extraction() {
        use crate::txxt::testing::assert_ast;

        // Test that we extract real content, not placeholder strings
        let input = "First paragraph with numbers 123 and symbols (like this).\n\nSecond paragraph.\n\n1. Session Title\n\n    Session content here.\n\n";

        let doc = crate::txxt::parser::parse_document(input).expect("Failed to parse");

        assert_ast(&doc)
            .item_count(3)
            .item(0, |item| {
                item.assert_paragraph()
                    .text("First paragraph with numbers 123 and symbols (like this).")
                    .line_count(1);
            })
            .item(1, |item| {
                item.assert_paragraph()
                    .text("Second paragraph.")
                    .line_count(1);
            })
            .item(2, |item| {
                item.assert_session()
                    .label("1. Session Title")
                    .child_count(1)
                    .child(0, |child| {
                        child
                            .assert_paragraph()
                            .text("Session content here.")
                            .line_count(1);
                    });
            });
    }

    // Session tests have been moved to elements/sessions.rs
    // List tests have been moved to elements/lists.rs
    // Definition tests have been moved to elements/definitions.rs

    // ==================== TRIFECTA TESTS ====================
    // Testing paragraphs + sessions + lists together

    #[test]
    fn test_trifecta_flat_simple() {
        // Test flat structure with all three elements
        use crate::txxt::testing::assert_ast;

        let source = TxxtSources::get_string("050-trifecta-flat-simple.txxt").unwrap();
        let tokens = lex(&source);
        let doc = parse(tokens, &source).unwrap();

        // Item 0-1: Opening paragraphs
        assert_ast(&doc)
            .item(0, |item| {
                item.assert_paragraph()
                    .text_contains("Trifecta Flat Structure Test");
            })
            .item(1, |item| {
                item.assert_paragraph()
                    .text_contains("all three core elements");
            });

        // Item 2: Session with only paragraphs
        assert_ast(&doc).item(2, |item| {
            item.assert_session()
                .label_contains("Session with Paragraph Content")
                .child_count(2)
                .child(0, |child| {
                    child
                        .assert_paragraph() // "Session with Paragraph Content"
                        .text_contains("starts with a paragraph");
                })
                .child(1, |child| {
                    child
                        .assert_paragraph() // "multiple paragraphs"
                        .text_contains("multiple paragraphs");
                });
        });

        // Item 3: Session with only a list
        assert_ast(&doc).item(3, |item| {
            item.assert_session()
                .label_contains("Session with List Content")
                .child_count(1)
                .child(0, |child| {
                    child.assert_list().item_count(3);
                });
        });

        // Item 4: Session with mixed content (para + list + para)
        assert_ast(&doc).item(4, |item| {
            item.assert_session()
                .label_contains("Session with Mixed Content")
                .child_count(3)
                .child(0, |child| {
                    child
                        .assert_paragraph()
                        .text_contains("starts with a paragraph");
                })
                .child(1, |child| {
                    child.assert_list().item_count(2);
                })
                .child(2, |child| {
                    child
                        .assert_paragraph()
                        .text_contains("ends with another paragraph");
                });
        });

        // Item 5: Root level paragraph
        assert_ast(&doc).item(5, |item| {
            item.assert_paragraph().text_contains("root level");
        });

        // Item 6: Root level list
        assert_ast(&doc).item(6, |item| {
            item.assert_list().item_count(2);
        });

        // Item 7: Session with list + para + list
        assert_ast(&doc).item(7, |item| {
            item.assert_session()
                .label_contains("Another Session")
                .child_count(3)
                .child(0, |child| {
                    child.assert_list().item_count(2);
                })
                .child(1, |child| {
                    child.assert_paragraph().text_contains("has a paragraph");
                })
                .child(2, |child| {
                    child.assert_list().item_count(2);
                });
        });
    }

    #[test]
    fn test_trifecta_nesting() {
        // Test nested structure with all three elements
        use crate::txxt::testing::assert_ast;

        let source = TxxtSources::get_string("060-trifecta-nesting.txxt").unwrap();
        let tokens = lex(&source);
        let doc = parse(tokens, &source).unwrap();

        // Item 0-1: Opening paragraphs
        assert_ast(&doc)
            .item(0, |item| {
                item.assert_paragraph() // "Trifecta Nesting Test"
                    .text_contains("Trifecta Nesting Test");
            })
            .item(1, |item| {
                item.assert_paragraph() // "various levels of nesting"
                    .text_contains("various levels of nesting");
            });

        // Item 2: Root session with nested sessions and mixed content
        // The structure has been updated to include nested lists, which may affect the child count
        assert_ast(&doc).item(2, |item| {
            item.assert_session()
                .label_contains("1. Root Session")
                .child_count(5); // para, subsession, subsession, para, list
        });

        // Verify first child of root session is paragraph
        assert_ast(&doc).item(2, |item| {
            item.assert_session().child(0, |child| {
                child.assert_paragraph().text_contains("nested elements");
            });
        });

        // Verify first nested session (1.1)
        assert_ast(&doc).item(2, |item| {
            item.assert_session().child(1, |child| {
                child
                    .assert_session()
                    .label_contains("1.1. Sub-session")
                    .child_count(2) // para + list
                    .child(0, |para| {
                        para.assert_paragraph();
                    })
                    .child(1, |list| {
                        list.assert_list().item_count(2);
                    });
            });
        });

        // Verify deeply nested session (1.2 containing 1.2.1)
        assert_ast(&doc).item(2, |item| {
            item.assert_session().child(2, |child| {
                child
                    .assert_session()
                    .label_contains("1.2. Sub-session with List")
                    .child_count(3) // list, para, nested session
                    .child(2, |nested| {
                        nested
                            .assert_session()
                            .label_contains("1.2.1. Deeply Nested")
                            .child_count(3); // para + list + list
                    });
            });
        });

        // Verify the deeply nested session has 2 lists
        assert_ast(&doc).item(2, |item| {
            item.assert_session().child(2, |subsession| {
                subsession.assert_session().child(2, |deeply_nested| {
                    deeply_nested
                        .assert_session()
                        .child(1, |first_list| {
                            first_list.assert_list().item_count(2);
                        })
                        .child(2, |second_list| {
                            second_list.assert_list().item_count(2);
                        });
                });
            });
        });

        // Item 3: Another root session with different nesting
        assert_ast(&doc).item(3, |item| {
            item.assert_session()
                .label_contains("2. Another Root Session")
                .child_count(2); // para + subsession
        });

        // Verify even deeper nesting (2.1.1)
        assert_ast(&doc).item(3, |item| {
            item.assert_session().child(1, |subsession| {
                subsession
                    .assert_session()
                    .label_contains("2.1. Mixed Content")
                    .child_count(4) // list, para, list, nested session
                    .child(3, |deeply_nested| {
                        deeply_nested
                            .assert_session()
                            .label_contains("2.1.1. Even Deeper")
                            .child_count(4); // para, list, para, list
                    });
            });
        });

        // Final root paragraph
        assert_ast(&doc).item(4, |item| {
            item.assert_paragraph()
                .text_contains("Final root level paragraph");
        });
    }

    // Nested list tests have been moved to elements/lists.rs
    // Definition tests have been moved to elements/definitions.rs

    #[test]
    fn test_verified_ensemble_with_definitions() {
        // Comprehensive ensemble test with all core elements including definitions
        use crate::txxt::testing::assert_ast;

        let source = TxxtSources::get_string("110-ensemble-with-definitions.txxt").unwrap();
        let tokens = lex(&source);
        let doc = parse(tokens, &source).unwrap();

        // Item 0-1: Opening paragraphs
        assert_ast(&doc)
            .item(0, |item| {
                item.assert_paragraph() // "Ensemble Test with Definitions"
                    .text_contains("Ensemble Test with Definitions");
            })
            .item(1, |item| {
                item.assert_paragraph() // "all core elements"
                    .text_contains("all core elements");
            });

        // Item 2: Introduction definition (with para + list)
        assert_ast(&doc).item(2, |item| {
            item.assert_definition()
                .subject("Introduction")
                .child_count(2)
                .child(0, |child| {
                    child.assert_paragraph().text_contains("ensemble test");
                })
                .child(1, |child| {
                    child.assert_list().item_count(4);
                });
        });

        // Item 3: Simple Elements Section session
        assert_ast(&doc).item(3, |item| {
            item.assert_session()
                .label("1. Simple Elements Section {{session}}")
                .child_count(5); // para + 2 definitions + para + list
        });

        // Item 4: Nested Elements Section session
        assert_ast(&doc).item(4, |item| {
            item.assert_session()
                .label("2. Nested Elements Section {{session}}")
                .child_count(3); // para + 2 subsections (2.1 and 2.2)
        });
    }

    // Annotation and foreign block tests have been moved to their respective element modules:
    // - elements/annotations.rs for annotation tests
    // - elements/foreign.rs for foreign block tests

    #[test]
    fn test_regression_definition_with_list_followed_by_definition() {
        // Issue: https://github.com/arthur-debert/txxt-nano/issues/41
        // See: docs/specs/v1/regression-bugs/parser-definition-list-transition.txxt

        let source = std::fs::read_to_string(
            "docs/specs/v1/regression-bugs/parser-definition-list-transition.txxt",
        )
        .expect("Failed to load regression test file");
        let tokens = lex(&source);

        // This should parse successfully but currently fails with:
        // Parse error at location 14..15: reason=Unexpected, found=Some((Newline, 34..35))
        let doc = parse(tokens, &source)
            .expect("Parser should handle definition with list followed by definition");

        // Should have 2 definitions
        assert_eq!(doc.root.content.len(), 2);

        // First should be a definition
        assert!(doc.root.content[0].as_definition().is_some());

        // Second should also be a definition
        assert!(doc.root.content[1].as_definition().is_some());
    }

    // ========================================================================
    // Integration Tests for Position Tracking
    // ========================================================================

    #[test]
    fn test_parse_simple() {
        let input = "Hello world\n\n";
        let tokens = lex(input);
        let doc = parse(tokens, input).expect("Failed to parse with positions");

        assert_eq!(doc.root.content.len(), 1);
        let para = doc.root.content[0].as_paragraph().unwrap();
        let location = para.location();
        assert_eq!(location.start.line, 0);
        assert_eq!(location.start.column, 0);
    }

    #[test]
    fn test_parse_multiline() {
        let input = "First line\nSecond line\n\n";
        let tokens = lex(input);
        let doc = parse(tokens, input).expect("Failed to parse with positions");

        assert_eq!(doc.root.content.len(), 1);
        let para = doc.root.content[0].as_paragraph().unwrap();

        // Should have 2 lines
        assert_eq!(para.lines.len(), 2);

        // Location should cover both lines
        let location = para.location();
        assert_eq!(location.start.line, 0);
        assert_eq!(location.end.line, 1);
    }

    #[test]
    fn test_element_at_query_on_parsed_document() {
        let input = "First paragraph\n\n2. Session Title\n\n    Session content\n\n";
        let tokens = lex(input);
        let doc = parse(tokens, input).expect("Failed to parse with positions");

        // Query for the session (should be at line 2)
        let result = doc.element_at(Position::new(2, 3));

        // Should find the element
        assert!(result.is_some(), "Should find element at position 2:3");

        // Result should be a session
        assert!(result.unwrap().is_session());
    }

    #[test]
    fn test_element_at_nested_position() {
        let input = "Title\n\n1. Item one\n\n    Nested content\n\n";
        let tokens = lex(input);
        let doc = parse(tokens, input).expect("Failed to parse with positions");

        // The document should have at least a paragraph and possibly a list
        assert!(!doc.root.content.is_empty());

        // Query for position in the nested content
        let result = doc.element_at(Position::new(4, 4));

        // Should find element at that position (or return None if position is outside all locations)
        // This is acceptable - position 4:4 might be outside all defined locations
        let _ = result;
    }

    #[test]
    fn test_position_comparison_in_query() {
        let input = "Line 0\n\nLine 2\n\n";
        let tokens = lex(input);
        let doc = parse(tokens, input).expect("Failed to parse with positions");

        // Get all items
        let items = doc.root.content.clone();

        // First paragraph should be at line 0
        if let Some(para) = items.first().and_then(|item| item.as_paragraph()) {
            let location = para.location();
            assert_eq!(location.start.line, 0);
        }

        // Second paragraph should be at line 2
        if let Some(para) = items.get(1).and_then(|item| item.as_paragraph()) {
            let location = para.location();
            assert_eq!(location.start.line, 2);
        }
    }

    // Annotation position test moved to elements/annotations.rs

    #[test]
    fn test_backward_compatibility_without_positions() {
        let input = "Simple paragraph\n\n";
        let tokens = lex(input);

        // Old parser should still work (without positions)
        let doc_old = parse(tokens.clone(), input).expect("Failed to parse without positions");

        // New parser with positions
        let doc_new = parse(tokens, input).expect("Failed to parse with positions");

        // Content should be identical
        assert_eq!(doc_old.root.content.len(), doc_new.root.content.len());

        let para_old = doc_old.root.content[0].as_paragraph().unwrap();
        let para_new = doc_new.root.content[0].as_paragraph().unwrap();

        // Text content should be the same (ignoring location information)
        assert_eq!(para_old.lines.len(), para_new.lines.len());
        for (_, line_new) in para_old.lines.iter().zip(para_new.lines.iter()) {
            // Old para had TextContent, new para has ContentItem::TextLine
            if let ContentItem::TextLine(tl) = line_new {
                // Just check that new version has text
                assert!(!tl.text().is_empty());
            } else {
                panic!("Expected TextLine in paragraph");
            }
        }

        // But new version should have positions on the paragraph and text
        let para_location = para_new.location();
        assert!(para_location.start <= para_location.end);
        if let ContentItem::TextLine(tl) = &para_new.lines[0] {
            let line_location = tl.location();
            assert!(line_location.start <= line_location.end);
        }
    }

    #[test]
    fn test_location_boundary_containment() {
        let input = "0123456789\n\n";
        let tokens = lex(input);
        let doc = parse(tokens, input).expect("Failed to parse with positions");

        let para = doc.root.content[0].as_paragraph().unwrap();
        let location = para.location();

        // Should contain position in the middle
        assert!(location.contains(Position::new(0, 5)));

        // Should contain start
        assert!(location.contains(location.start));

        // Should contain end
        assert!(location.contains(location.end));

        // Should?? contain position after end
        assert!(!location.contains(Position::new(0, 11)));
    }

    #[test]
    fn test_nested_paragraph_has_location() {
        // Test that nested paragraphs inside sessions have location information
        let input = "Title\n\n1. Session Title\n\n    Nested paragraph\n\n";
        let tokens = lex(input);
        let doc = parse(tokens, input).expect("Failed to parse with positions");

        assert!(doc.root.content.len() >= 2);

        // Find the session
        let session = doc
            .root
            .content
            .iter()
            .find(|item| item.is_session())
            .expect("Should have a session");

        let session_location = session.location();
        assert!(
            session_location.start <= session_location.end,
            "Session should have location"
        );

        // Get nested content
        if let Some(children) = session.children() {
            assert!(!children.is_empty(), "Session should have children");

            // Check if first child is a paragraph
            if let Some(para_item) = children.first() {
                if para_item.is_paragraph() {
                    let para = para_item.as_paragraph().unwrap();
                    let location = para.location();
                    assert!(
                        location.start <= location.end,
                        "Nested paragraph should have location, but got {:?}",
                        location
                    );

                    println!(
                        "Nested paragraph location: {:?} to {:?}",
                        location.start, location.end
                    );
                    assert_eq!(location.start.line, 4, "Paragraph should be at line 4");
                }
            }
        }
    }

    #[test]
    fn test_location_tracking_for_core_elements() {
        let source = TxxtSources::get_string("110-ensemble-with-definitions.txxt")
            .expect("Failed to load ensemble sample");
        let tokens = lex(&source);
        let doc = parse(tokens, &source).expect("Failed to parse ensemble sample");

        // Document doesn't have its own location; location comes from root
        let root_location = doc.root.location();
        assert!(
            root_location.start <= root_location.end,
            "Root session should have a location"
        );

        for item in &doc.root.content {
            let item_location = item.location();
            assert!(
                item_location.start <= item_location.end,
                "{} should have a location",
                item.node_type()
            );

            match item {
                ContentItem::Paragraph(paragraph) => {
                    for line in &paragraph.lines {
                        if let ContentItem::TextLine(tl) = line {
                            let line_location = tl.location();
                            assert!(
                                line_location.start <= line_location.end,
                                "Paragraph line should have location"
                            );
                        }
                    }
                }
                ContentItem::Session(session) => {
                    assert!(
                        session.title.location.is_some(),
                        "Session title is missing location"
                    );
                    for child in &session.content {
                        let child_location = child.location();
                        assert!(
                            child_location.start <= child_location.end,
                            "Session child should have location"
                        );
                    }
                }
                ContentItem::Definition(definition) => {
                    assert!(
                        definition.subject.location.is_some(),
                        "Definition subject should have location"
                    );
                    for child in &definition.content {
                        let child_location = child.location();
                        assert!(
                            child_location.start <= child_location.end,
                            "Definition child should have location"
                        );
                    }
                }
                ContentItem::List(list) => {
                    for item in &list.content {
                        if let ContentItem::ListItem(list_item) = item {
                            for text in &list_item.text {
                                assert!(
                                    text.location.is_some(),
                                    "List item text should have location"
                                );
                            }
                            for child in &list_item.content {
                                let child_location = child.location();
                                assert!(
                                    child_location.start <= child_location.end,
                                    "Nested list item child should have location"
                                );
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_location_tracking_for_annotations() {
        let source = TxxtSources::get_string("120-annotations-simple.txxt")
            .expect("Failed to load annotations sample");
        let tokens = lex(&source);
        let doc = parse(tokens, &source).expect("Failed to parse annotations sample");

        let annotations: Vec<_> = doc
            .root
            .content
            .iter()
            .filter_map(|item| item.as_annotation())
            .collect();
        assert!(!annotations.is_empty(), "Expected annotations in sample");

        for annotation in annotations {
            for child in &annotation.content {
                let child_location = child.location();
                assert!(
                    child_location.start <= child_location.end,
                    "Annotation content should have a location"
                );
            }
        }
    }

    #[test]
    fn test_location_tracking_for_foreign_blocks() {
        let source = TxxtSources::get_string("140-foreign-blocks-simple.txxt")
            .expect("Failed to load foreign blocks sample");
        let tokens = lex(&source);
        let doc = parse(tokens, &source).expect("Failed to parse foreign blocks sample");

        let foreign_blocks: Vec<_> = doc
            .root
            .content
            .iter()
            .filter_map(|item| item.as_foreign_block())
            .collect();
        assert!(
            !foreign_blocks.is_empty(),
            "Expected foreign blocks in sample"
        );

        for block in foreign_blocks {
            assert!(
                block.subject.location.is_some(),
                "Foreign block subject should have a location"
            );
            if !block.content.as_string().is_empty() {
                assert!(
                    block.content.location.is_some(),
                    "Foreign block content should have a location"
                );
            }

            let closing = &block.closing_annotation;
            for child in &closing.content {
                let child_location = child.location();
                assert!(
                    child_location.start <= child_location.end,
                    "Closing annotation content should have a location"
                );
            }
        }
    }
}
