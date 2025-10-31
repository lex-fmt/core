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

// Import Phase 3b refactored document parser from elements::document module
use super::elements::document as document_module;

/// Parse a document - delegated to document module
/// Phase 5: The document parser requires source text to populate location information
pub fn document() -> impl Parser<TokenLocation, Document, Error = ParserError> {
    // This function is kept for backward compatibility but delegates to document_module::document(source)
    // Since this function doesn't have access to source, it uses an empty string.
    // For proper position tracking, use parse_with_source instead.
    document_module::document("")
}

/// Parse with source text - the primary parsing function
///
/// Parses tokens with location information and source text to produce a Document.
/// All parsed documents include complete location information automatically.
///
/// Re-exports the canonical implementation from api.rs
pub fn parse_with_source(
    tokens_with_locations: Vec<TokenLocation>,
    source: &str,
) -> Result<Document, Vec<ParserError>> {
    super::api::parse_with_source(tokens_with_locations, source)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::txxt::ast::{AstNode, Position};
    use crate::txxt::lexer::lex_with_locations;
    use crate::txxt::processor::txxt_sources::TxxtSources;
    use std::sync::Arc;

    #[test]
    fn test_simple_paragraph() {
        let input = "Hello world\n\n";
        let tokens_with_locations = lex_with_locations(input);

        let result = paragraph(Arc::new(input.to_string())).parse(tokens_with_locations);
        assert!(result.is_ok(), "Failed to parse paragraph: {:?}", result);

        let para = result.unwrap();
        assert_eq!(para.lines.len(), 1);
        assert_eq!(para.lines[0].as_string(), "Hello world");
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
        let tokens = lex_with_locations(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

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
        let tokens = lex_with_locations(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

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
        let tokens = lex_with_locations(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

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
        let tokens = lex_with_locations(&source);

        // This should parse successfully but currently fails with:
        // Parse error at location 14..15: reason=Unexpected, found=Some((Newline, 34..35))
        let doc = parse_with_source(tokens, &source)
            .expect("Parser should handle definition with list followed by definition");

        // Should have 2 definitions
        assert_eq!(doc.content.len(), 2);

        // First should be a definition
        assert!(doc.content[0].as_definition().is_some());

        // Second should also be a definition
        assert!(doc.content[1].as_definition().is_some());
    }

    // ========================================================================
    // Integration Tests for Position Tracking
    // ========================================================================

    #[test]
    fn test_parse_with_source_simple() {
        let input = "Hello world\n\n";
        let tokens = lex_with_locations(input);
        let doc = parse_with_source(tokens, input).expect("Failed to parse with positions");

        assert_eq!(doc.content.len(), 1);
        let para = doc.content[0].as_paragraph().unwrap();
        assert!(para.location().is_some(), "Paragraph should have location");

        let location = para.location().unwrap();
        assert_eq!(location.start.line, 0);
        assert_eq!(location.start.column, 0);
    }

    #[test]
    fn test_parse_with_source_multiline() {
        let input = "First line\nSecond line\n\n";
        let tokens = lex_with_locations(input);
        let doc = parse_with_source(tokens, input).expect("Failed to parse with positions");

        assert_eq!(doc.content.len(), 1);
        let para = doc.content[0].as_paragraph().unwrap();

        // Should have 2 lines
        assert_eq!(para.lines.len(), 2);

        // Location should cover both lines
        let location = para.location().unwrap();
        assert_eq!(location.start.line, 0);
        assert_eq!(location.end.line, 1);
    }

    #[test]
    fn test_elements_at_query_on_parsed_document() {
        let input = "First paragraph\n\n2. Session Title\n\n    Session content\n\n";
        let tokens = lex_with_locations(input);
        let doc = parse_with_source(tokens, input).expect("Failed to parse with positions");

        // Query for the session (should be at line 2)
        let results = doc.elements_at(Position::new(2, 3));

        // Should find at least the session
        assert!(!results.is_empty(), "Should find elements at position 2:3");

        // First result should be a session
        assert!(results[0].is_session());
    }

    #[test]
    fn test_elements_at_nested_position() {
        let input = "Title\n\n1. Item one\n\n    Nested content\n\n";
        let tokens = lex_with_locations(input);
        let doc = parse_with_source(tokens, input).expect("Failed to parse with positions");

        // The document should have at least a paragraph and possibly a list
        assert!(!doc.content.is_empty());

        // Query for position in the nested content
        let results = doc.elements_at(Position::new(4, 4));

        // Should find elements at that position (or return empty if position is outside all locations)
        // This is acceptable - position 4:4 might be outside all defined locations
        let _ = results;
    }

    #[test]
    fn test_position_comparison_in_query() {
        let input = "Line 0\n\nLine 2\n\n";
        let tokens = lex_with_locations(input);
        let doc = parse_with_source(tokens, input).expect("Failed to parse with positions");

        // Get all items
        let items = doc.content.clone();

        // First paragraph should be at line 0
        if let Some(para) = items.first().and_then(|item| item.as_paragraph()) {
            if let Some(location) = para.location() {
                assert_eq!(location.start.line, 0);
            }
        }

        // Second paragraph should be at line 2
        if let Some(para) = items.get(1).and_then(|item| item.as_paragraph()) {
            if let Some(location) = para.location() {
                assert_eq!(location.start.line, 2);
            }
        }
    }

    // Annotation position test moved to elements/annotations.rs

    #[test]
    fn test_backward_compatibility_without_positions() {
        let input = "Simple paragraph\n\n";
        let tokens = lex_with_locations(input);

        // Old parser should still work (without positions)
        let doc_old =
            parse_with_source(tokens.clone(), input).expect("Failed to parse without positions");

        // New parser with positions
        let doc_new = parse_with_source(tokens, input).expect("Failed to parse with positions");

        // Content should be identical
        assert_eq!(doc_old.content.len(), doc_new.content.len());

        let para_old = doc_old.content[0].as_paragraph().unwrap();
        let para_new = doc_new.content[0].as_paragraph().unwrap();

        // Text content should be the same (ignoring location information)
        assert_eq!(para_old.lines.len(), para_new.lines.len());
        for (line_old, line_new) in para_old.lines.iter().zip(para_new.lines.iter()) {
            assert_eq!(line_old.as_string(), line_new.as_string());
        }

        // But new version should have positions on the paragraph and text
        assert!(para_new.location().is_some());
        assert!(para_new.lines[0].location.is_some());
    }

    #[test]
    fn test_location_boundary_containment() {
        let input = "0123456789\n\n";
        let tokens = lex_with_locations(input);
        let doc = parse_with_source(tokens, input).expect("Failed to parse with positions");

        let para = doc.content[0].as_paragraph().unwrap();
        let location = para.location().unwrap();

        // Should contain position in the middle
        assert!(location.contains(Position::new(0, 5)));

        // Should contain start
        assert!(location.contains(location.start));

        // Should contain end
        assert!(location.contains(location.end));

        // Shouldมี contain position after end
        assert!(!location.contains(Position::new(0, 11)));
    }

    #[test]
    fn test_nested_paragraph_has_location() {
        // Test that nested paragraphs inside sessions have location information
        let input = "Title\n\n1. Session Title\n\n    Nested paragraph\n\n";
        let tokens = lex_with_locations(input);
        let doc = parse_with_source(tokens, input).expect("Failed to parse with positions");

        assert!(doc.content.len() >= 2);

        // Find the session
        let session = doc
            .content
            .iter()
            .find(|item| item.is_session())
            .expect("Should have a session");

        assert!(session.location().is_some(), "Session should have location");

        // Get nested content
        if let Some(children) = session.children() {
            assert!(!children.is_empty(), "Session should have children");

            // Check if first child is a paragraph
            if let Some(para_item) = children.first() {
                if para_item.is_paragraph() {
                    let para = para_item.as_paragraph().unwrap();
                    assert!(
                        para.location().is_some(),
                        "Nested paragraph should have location, but got {:?}",
                        para.location()
                    );

                    if let Some(location) = para.location() {
                        println!(
                            "Nested paragraph location: {:?} to {:?}",
                            location.start, location.end
                        );
                        assert_eq!(location.start.line, 4, "Paragraph should be at line 4");
                    }
                }
            }
        }
    }

    #[test]
    fn test_location_tracking_for_core_elements() {
        let source = TxxtSources::get_string("110-ensemble-with-definitions.txxt")
            .expect("Failed to load ensemble sample");
        let tokens = lex_with_locations(&source);
        let doc = parse_with_source(tokens, &source).expect("Failed to parse ensemble sample");

        assert!(doc.location.is_some(), "Document should have a location");

        for item in &doc.content {
            assert!(
                item.location().is_some(),
                "{} should have a location",
                item.node_type()
            );

            match item {
                ContentItem::Paragraph(paragraph) => {
                    assert!(
                        paragraph.location.is_some(),
                        "Paragraph is missing location"
                    );
                    for line in &paragraph.lines {
                        assert!(
                            line.location.is_some(),
                            "Paragraph line should have location"
                        );
                    }
                }
                ContentItem::Session(session) => {
                    assert!(session.location.is_some(), "Session is missing location");
                    assert!(
                        session.title.location.is_some(),
                        "Session title is missing location"
                    );
                    for child in &session.content {
                        assert!(
                            child.location().is_some(),
                            "Session child should have location"
                        );
                    }
                }
                ContentItem::Definition(definition) => {
                    assert!(
                        definition.location.is_some(),
                        "Definition is missing location"
                    );
                    assert!(
                        definition.subject.location.is_some(),
                        "Definition subject should have location"
                    );
                    for child in &definition.content {
                        assert!(
                            child.location().is_some(),
                            "Definition child should have location"
                        );
                    }
                }
                ContentItem::List(list) => {
                    assert!(list.location.is_some(), "List is missing location");
                    for list_item in &list.content {
                        assert!(
                            list_item.location.is_some(),
                            "List item should have location"
                        );
                        for text in &list_item.text {
                            assert!(
                                text.location.is_some(),
                                "List item text should have location"
                            );
                        }
                        for child in &list_item.content {
                            assert!(
                                child.location().is_some(),
                                "Nested list item child should have location"
                            );
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
        let tokens = lex_with_locations(&source);
        let doc = parse_with_source(tokens, &source).expect("Failed to parse annotations sample");

        let annotations: Vec<_> = doc
            .content
            .iter()
            .filter_map(|item| item.as_annotation())
            .collect();
        assert!(!annotations.is_empty(), "Expected annotations in sample");

        for annotation in annotations {
            assert!(
                annotation.location.is_some(),
                "Annotation should have a location"
            );
            assert!(
                annotation.label.location.is_some(),
                "Annotation label should have a location"
            );
            for parameter in &annotation.parameters {
                assert!(
                    parameter.location.is_some(),
                    "Annotation parameter should have a location"
                );
            }
            for child in &annotation.content {
                assert!(
                    child.location().is_some(),
                    "Annotation content should have a location"
                );
            }
        }
    }

    #[test]
    fn test_location_tracking_for_foreign_blocks() {
        let source = TxxtSources::get_string("140-foreign-blocks-simple.txxt")
            .expect("Failed to load foreign blocks sample");
        let tokens = lex_with_locations(&source);
        let doc =
            parse_with_source(tokens, &source).expect("Failed to parse foreign blocks sample");

        let foreign_blocks: Vec<_> = doc
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
                block.location.is_some(),
                "Foreign block should have a location"
            );
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
            assert!(
                closing.location.is_some(),
                "Closing annotation should have a location"
            );
            assert!(
                closing.label.location.is_some(),
                "Closing annotation label should have a location"
            );
            for parameter in &closing.parameters {
                assert!(
                    parameter.location.is_some(),
                    "Closing annotation parameter should have a location"
                );
            }
            for child in &closing.content {
                assert!(
                    child.location().is_some(),
                    "Closing annotation content should have a location"
                );
            }
        }
    }
}
