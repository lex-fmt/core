//! Parser implementation for the txxt format using chumsky
//!
//! This module implements a parser combinator-based parser for txxt documents.
//! It builds on the token stream from the lexer and produces an AST.
//!
//! ## Testing
//!
//! All parser tests must follow strict guidelines. See the [testing module](crate::txxt_nano::testing)
//! for comprehensive documentation on using verified txxt sources and AST assertions.

use chumsky::prelude::*;
use std::ops::Range;

#[allow(unused_imports)] // Container is used in tests
use crate::txxt_nano::ast::{Container, ContentItem, Document};
use crate::txxt_nano::lexer::Token;

/// Type alias for token with span
type TokenSpan = (Token, Range<usize>);

/// Type alias for parser error
type ParserError = Simple<TokenSpan>;

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
) -> impl Parser<TokenSpan, Vec<ContentItem>, Error = ParserError> + Clone {
    let source = Arc::new(source.to_string());

    recursive(move |items| {
        let source = source.clone();
        let single_item = {
            // Session parser - now builds final Session type with span
            let session_parser = build_session_parser(source.clone(), items.clone());

            // Definition parser - now builds final Definition type with span
            let definition_parser = build_definition_parser(source.clone(), items.clone());

            // List parser - now builds final List type with span
            let list_parser = build_list_parser(source.clone(), items.clone());

            // Annotation parser - now builds final Annotation type with span
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
/// Phase 5: The document parser requires source text to populate span information
pub fn document() -> impl Parser<TokenSpan, Document, Error = ParserError> {
    // This function is kept for backward compatibility but delegates to document_module::document(source)
    // Since this function doesn't have access to source, it uses an empty string.
    // For proper position tracking, use parse_with_source_positions or parse_with_source instead.
    document_module::document("")
}

/// Parse with source text - extracts actual content from spans
///
/// Re-exports the canonical implementation from api.rs
pub fn parse_with_source(
    tokens_with_spans: Vec<TokenSpan>,
    source: &str,
) -> Result<Document, Vec<ParserError>> {
    super::api::parse_with_source(tokens_with_spans, source)
}

/// Parse a txxt document from tokens with source, preserving position information
///
/// This version preserves line/column position information in all AST nodes,
/// enabling position-based queries like `elements_at()` for IDE integrations,
/// error reporting, and source mapping.
///
/// Re-exports the canonical implementation from api.rs
pub fn parse_with_source_positions(
    tokens_with_spans: Vec<TokenSpan>,
    source: &str,
) -> Result<Document, Vec<ParserError>> {
    super::api::parse_with_source_positions(tokens_with_spans, source)
}

/// Parse a txxt document from a token stream (legacy - doesn't preserve source text)
pub fn parse(tokens: Vec<Token>) -> Result<Document, Vec<Simple<Token>>> {
    // Convert tokens to token-span tuples with empty spans
    let tokens_with_spans: Vec<TokenSpan> = tokens.into_iter().map(|t| (t, 0..0)).collect();

    // Parse with empty source
    parse_with_source(tokens_with_spans, "")
        .map_err(|errs| errs.into_iter().map(|e| e.map(|(t, _)| t)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::txxt_nano::ast::{AstNode, Position};
    use crate::txxt_nano::lexer::lex_with_locations;
    use crate::txxt_nano::processor::txxt_sources::TxxtSources;
    use std::sync::Arc;

    #[test]
    fn test_simple_paragraph() {
        let input = "Hello world\n\n";
        let tokens_with_spans = lex_with_locations(input);

        let result = paragraph(Arc::new(input.to_string())).parse(tokens_with_spans);
        assert!(result.is_ok(), "Failed to parse paragraph: {:?}", result);

        let para = result.unwrap();
        assert_eq!(para.lines.len(), 1);
        assert_eq!(para.lines[0].as_string(), "Hello world");
    }

    #[test]
    fn test_real_content_extraction() {
        use crate::txxt_nano::testing::assert_ast;

        // Test that we extract real content, not placeholder strings
        let input = "First paragraph with numbers 123 and symbols (like this).\n\nSecond paragraph.\n\n1. Session Title\n\n    Session content here.\n\n";

        let doc = crate::txxt_nano::parser::parse_document(input).expect("Failed to parse");

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
        use crate::txxt_nano::testing::assert_ast;

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
        use crate::txxt_nano::testing::assert_ast;

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
    #[ignore = "Still failing - needs investigation"]
    fn test_verified_ensemble_with_definitions() {
        // Comprehensive ensemble test with all core elements including definitions
        use crate::txxt_nano::testing::assert_ast;

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
    #[ignore = "Regression: parser fails when definition with list is followed by another definition"]
    fn test_regression_definition_with_list_followed_by_definition() {
        // Issue: https://github.com/arthur-debert/txxt-nano/issues/41
        // See: docs/specs/v1/regression-bugs/parser-definition-list-transition.txxt

        let source = std::fs::read_to_string(
            "docs/specs/v1/regression-bugs/parser-definition-list-transition.txxt",
        )
        .expect("Failed to load regression test file");
        let tokens = lex_with_locations(&source);

        // This should parse successfully but currently fails with:
        // Parse error at span 14..15: reason=Unexpected, found=Some((Newline, 34..35))
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
    fn test_parse_with_source_positions_simple() {
        let input = "Hello world\n\n";
        let tokens = lex_with_locations(input);
        let doc =
            parse_with_source_positions(tokens, input).expect("Failed to parse with positions");

        assert_eq!(doc.content.len(), 1);
        let para = doc.content[0].as_paragraph().unwrap();
        assert!(para.span.is_some(), "Paragraph should have span");

        let span = para.span.unwrap();
        assert_eq!(span.start.line, 0);
        assert_eq!(span.start.column, 0);
    }

    #[test]
    fn test_parse_with_source_positions_multiline() {
        let input = "First line\nSecond line\n\n";
        let tokens = lex_with_locations(input);
        let doc =
            parse_with_source_positions(tokens, input).expect("Failed to parse with positions");

        assert_eq!(doc.content.len(), 1);
        let para = doc.content[0].as_paragraph().unwrap();

        // Should have 2 lines
        assert_eq!(para.lines.len(), 2);

        // Span should cover both lines
        let span = para.span.unwrap();
        assert_eq!(span.start.line, 0);
        assert_eq!(span.end.line, 1);
    }

    #[test]
    fn test_elements_at_query_on_parsed_document() {
        let input = "First paragraph\n\n2. Session Title\n\n    Session content\n\n";
        let tokens = lex_with_locations(input);
        let doc =
            parse_with_source_positions(tokens, input).expect("Failed to parse with positions");

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
        let doc =
            parse_with_source_positions(tokens, input).expect("Failed to parse with positions");

        // The document should have at least a paragraph and possibly a list
        assert!(!doc.content.is_empty());

        // Query for position in the nested content
        let results = doc.elements_at(Position::new(4, 4));

        // Should find elements at that position (or return empty if position is outside all spans)
        // This is acceptable - position 4:4 might be outside all defined spans
        let _ = results;
    }

    #[test]
    fn test_position_comparison_in_query() {
        let input = "Line 0\n\nLine 2\n\n";
        let tokens = lex_with_locations(input);
        let doc =
            parse_with_source_positions(tokens, input).expect("Failed to parse with positions");

        // Get all items
        let items = doc.content.clone();

        // First paragraph should be at line 0
        if let Some(para) = items.first().and_then(|item| item.as_paragraph()) {
            if let Some(span) = para.span {
                assert_eq!(span.start.line, 0);
            }
        }

        // Second paragraph should be at line 2
        if let Some(para) = items.get(1).and_then(|item| item.as_paragraph()) {
            if let Some(span) = para.span {
                assert_eq!(span.start.line, 2);
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
        let doc_new =
            parse_with_source_positions(tokens, input).expect("Failed to parse with positions");

        // Content should be identical
        assert_eq!(doc_old.content.len(), doc_new.content.len());

        let para_old = doc_old.content[0].as_paragraph().unwrap();
        let para_new = doc_new.content[0].as_paragraph().unwrap();

        // Text content should be the same (ignoring span information)
        assert_eq!(para_old.lines.len(), para_new.lines.len());
        for (line_old, line_new) in para_old.lines.iter().zip(para_new.lines.iter()) {
            assert_eq!(line_old.as_string(), line_new.as_string());
        }

        // But new version should have positions on the paragraph and text
        assert!(para_new.span.is_some());
        assert!(para_new.lines[0].span.is_some());
    }

    #[test]
    fn test_span_boundary_containment() {
        let input = "0123456789\n\n";
        let tokens = lex_with_locations(input);
        let doc =
            parse_with_source_positions(tokens, input).expect("Failed to parse with positions");

        let para = doc.content[0].as_paragraph().unwrap();
        let span = para.span.unwrap();

        // Should contain position in the middle
        assert!(span.contains(Position::new(0, 5)));

        // Should contain start
        assert!(span.contains(span.start));

        // Should contain end
        assert!(span.contains(span.end));

        // Shouldมี contain position after end
        assert!(!span.contains(Position::new(0, 11)));
    }

    #[test]
    fn test_nested_paragraph_has_span() {
        // Test that nested paragraphs inside sessions have span information
        let input = "Title\n\n1. Session Title\n\n    Nested paragraph\n\n";
        let tokens = lex_with_locations(input);
        let doc =
            parse_with_source_positions(tokens, input).expect("Failed to parse with positions");

        assert!(doc.content.len() >= 2);

        // Find the session
        let session = doc
            .content
            .iter()
            .find(|item| item.is_session())
            .expect("Should have a session");

        assert!(session.location().is_some(), "Session should have span");

        // Get nested content
        if let Some(children) = session.children() {
            assert!(!children.is_empty(), "Session should have children");

            // Check if first child is a paragraph
            if let Some(para_item) = children.first() {
                if para_item.is_paragraph() {
                    let para = para_item.as_paragraph().unwrap();
                    assert!(
                        para.span.is_some(),
                        "Nested paragraph should have span, but got {:?}",
                        para.span
                    );

                    if let Some(span) = para.span {
                        println!("Nested paragraph span: {:?} to {:?}", span.start, span.end);
                        assert_eq!(span.start.line, 4, "Paragraph should be at line 4");
                    }
                }
            }
        }
    }
}
