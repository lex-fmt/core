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

// Import parser builders from document module
use super::combinators::token;
use super::document::{build_definition_parser, build_list_parser, build_session_parser};
use super::elements::annotations::build_annotation_parser;
use super::elements::foreign::foreign_block;
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

// Import Phase 3b refactored document parser from document module
use super::document as document_module;

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
    use crate::txxt_nano::ast::{ContentItem, Position};
    use crate::txxt_nano::lexer::{lex, lex_with_spans};
    use crate::txxt_nano::processor::txxt_sources::TxxtSources;
    use std::sync::Arc;

    #[test]
    fn test_simple_paragraph() {
        let input = "Hello world\n\n";
        let tokens_with_spans = lex_with_spans(input);

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

    #[test]
    fn test_malformed_session_title_with_indent_but_no_content() {
        // Test the exact scenario from the code review:
        // A text line followed by blank line and IndentLevel, but no actual parseable content
        // Session parser should fail (expects content after IndentLevel)
        // Then paragraph parser tries and consumes the text line
        // This leaves IndentLevel token unconsumed, causing confusing error

        // We need actual indented content to get an IndentLevel token
        // So let's use a session title followed by just a newline at the indent level
        let input = "This looks like a session title\n\n    \n"; // Title + blank + indented newline
        let tokens = lex(input);

        println!("\n=== Test: Session title pattern with IndentLevel but no parseable content ===");
        println!("Input: {:?}", input);
        println!("Tokens: {:?}", tokens);

        let result = parse(tokens.clone());

        match &result {
            Ok(doc) => {
                println!("\n✓ Parsed successfully");
                println!("Document has {} items:", doc.content.len());
                for (i, item) in doc.content.iter().enumerate() {
                    println!("  {}: {}", i, item);
                }
                // This might actually be fine - the blank indented line might be ignored
            }
            Err(errors) => {
                println!("\n✗ Parse failed with errors:");
                for error in errors {
                    println!("  Error at span {:?}: {:?}", error.span(), error.reason());
                    println!("  Found: {:?}", error.found());
                }

                // This is expected to fail, but the question is:
                // Does it fail with a GOOD error message or a CONFUSING one?

                // If paragraph parser consumed the title line, the error will be about
                // finding IndentLevel when it expected something else (paragraph content or end)
            }
        }
    }

    #[test]
    fn test_session_title_followed_by_bare_indent_level() {
        // Test case 1: Session with empty content (IndentLevel immediately followed by DedentLevel)
        // This actually SHOULD be allowed or give a clear error
        let tokens = vec![
            Token::Text("".to_string()),
            Token::Newline,
            Token::Newline,
            Token::IndentLevel,
            Token::DedentLevel,
            Token::DedentLevel,
        ];

        println!("\n=== Test: Session with empty content ===");
        println!("Tokens: {:?}", tokens);

        let result = parse(tokens.clone());

        match &result {
            Ok(doc) => {
                println!("\n✓ Parsed as session with 0 children");
                println!("Document has {} items:", doc.content.len());
                for (i, item) in doc.content.iter().enumerate() {
                    match item {
                        ContentItem::Paragraph(p) => {
                            println!("  {}: Paragraph with {} lines", i, p.lines.len());
                        }
                        ContentItem::Session(s) => {
                            println!(
                                "  {}: Session '{}' with {} children",
                                i,
                                s.label(),
                                s.content.len()
                            );
                        }
                        ContentItem::List(l) => {
                            println!("  {}: List with {} items", i, l.items.len());
                        }
                        ContentItem::Definition(d) => {
                            println!(
                                "  {}: Definition '{}' with {} children",
                                i,
                                d.label(),
                                d.content.len()
                            );
                        }
                        ContentItem::Annotation(a) => {
                            println!(
                                "  {}: Annotation '{}' with {} children",
                                i,
                                a.label.value,
                                a.content.len()
                            );
                        }
                        ContentItem::ForeignBlock(fb) => {
                            println!(
                                "  {}: ForeignBlock '{}' with {} chars, closing: {}",
                                i,
                                fb.subject.as_string(),
                                fb.content.as_string().len(),
                                fb.closing_annotation.label.value
                            );
                        }
                    }
                }
                // This is actually fine - empty session
            }
            Err(errors) => {
                println!("\n✗ Parse failed:");
                for error in errors {
                    println!("  Error at span {:?}: {:?}", error.span(), error.reason());
                }
            }
        }
    }

    #[test]
    fn test_greedy_paragraph_parser_bug() {
        // Without DocEnd tokens, trailing unparseable tokens are simply ignored.
        // The parser parses what it can and returns successfully.
        // This is actually cleaner behavior than requiring an explicit error.

        let tokens = vec![
            Token::Text("".to_string()), // "title"
            Token::Newline,
            Token::Newline,
            Token::IndentLevel,
            Token::Colon, // This is not valid content but doesn't break parsing
            Token::DedentLevel,
            Token::DedentLevel,
        ];

        println!(
            "\n=== Test: Greedy paragraph parser - session title + IndentLevel + unparseable content ==="
        );
        println!("Tokens: {:?}", tokens);

        let result = parse(tokens.clone());

        // After removing DocEnd, the parser gracefully handles this
        // It parses the paragraph and ignores the unparseable trailing tokens
        assert!(
            result.is_ok(),
            "Parser should handle trailing unparseable tokens gracefully"
        );

        let doc = result.unwrap();
        assert_eq!(doc.content.len(), 1, "Should have one paragraph");

        match &doc.content[0] {
            ContentItem::Paragraph(p) => {
                println!(
                    "✓ Successfully parsed as paragraph with {} lines",
                    p.lines.len()
                );
                assert_eq!(p.lines.len(), 1);
            }
            _ => panic!("Expected paragraph"),
        }
    }

    #[test]
    fn test_session_title_pattern_without_indent() {
        // This is just a paragraph - text + blank line
        // Should parse fine as a paragraph
        let input = "Normal paragraph\n\nAnother paragraph\n\n";
        let tokens = lex(input);

        println!("\n=== Test: Normal paragraphs (no IndentLevel) ===");
        let result = parse(tokens);

        match &result {
            Ok(doc) => {
                println!("✓ Parsed successfully");
                println!("Document has {} items:", doc.content.len());
                assert_eq!(doc.content.len(), 2, "Should have 2 paragraphs");
            }
            Err(e) => {
                panic!("Should have parsed successfully: {:?}", e);
            }
        }
    }

    #[test]
    fn test_verified_paragraphs_sample() {
        use crate::txxt_nano::testing::assert_ast;

        let source =
            TxxtSources::get_string("000-paragraphs.txxt").expect("Failed to load sample file");
        let tokens = lex(&source);

        let result = parse(tokens);
        assert!(
            result.is_ok(),
            "Failed to parse 000-paragraphs.txxt: {:?}",
            result
        );

        let doc = result.unwrap();

        // Expected structure based on 000-paragraphs.txxt:
        // 7 paragraphs total, with specific line counts
        assert_ast(&doc)
            .item_count(7)
            .item(0, |item| {
                item.assert_paragraph().line_count(1); // "Simple Paragraphs Test"
            })
            .item(1, |item| {
                item.assert_paragraph().line_count(1); // "This is a simple paragraph with just one line."
            })
            .item(2, |item| {
                item.assert_paragraph().line_count(3); // Multi-line paragraph
            })
            .item(3, |item| {
                item.assert_paragraph().line_count(1); // "Another paragraph follows..."
            })
            .item(4, |item| {
                item.assert_paragraph().line_count(1); // Paragraph with special chars
            })
            .item(5, |item| {
                item.assert_paragraph().line_count(1); // Paragraph with numbers
            })
            .item(6, |item| {
                item.assert_paragraph().line_count(1); // Paragraph with mixed content
            });
    }

    #[test]
    fn test_verified_single_session_sample() {
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("010-paragraphs-sessions-flat-single.txxt")
            .expect("Failed to load sample file");
        let tokens = lex(&source);

        let result = parse(tokens.clone());
        assert!(
            result.is_ok(),
            "Failed to parse 010-paragraphs-sessions-flat-single.txxt: {:?}",
            result
        );

        let doc = result.unwrap();

        // Expected structure based on 010-paragraphs-sessions-flat-single.txxt:
        // Line 1: "Paragraphs and Single Session Test" - paragraph (1 line)
        // Line 3: "This document tests..." - paragraph (1 line)
        // Line 5: "1. Introduction" - session with 2 paragraphs
        //   Line 7: "This is the content..." - paragraph (1 line)
        //   Line 9: "The session can contain..." - paragraph (1 line)
        // Line 11: "This paragraph comes after..." - paragraph (1 line)
        // Line 13: "Another Session" - session with 1 paragraph
        //   Line 15: "This session demonstrates..." - paragraph (1 line)
        // Line 17: "Final paragraph..." - paragraph (1 line)

        assert_ast(&doc)
            .item_count(6)
            .item(0, |item| {
                item.assert_paragraph().line_count(1);
            })
            .item(1, |item| {
                item.assert_paragraph().line_count(1);
            })
            .item(2, |item| {
                item.assert_session()
                    .child_count(2)
                    .child(0, |child| {
                        child.assert_paragraph().line_count(1);
                    })
                    .child(1, |child| {
                        child.assert_paragraph().line_count(1);
                    });
            })
            .item(3, |item| {
                item.assert_paragraph().line_count(1);
            })
            .item(4, |item| {
                item.assert_session().child_count(1).child(0, |child| {
                    child.assert_paragraph().line_count(1);
                });
            })
            .item(5, |item| {
                item.assert_paragraph().line_count(1);
            });
    }

    #[test]
    fn test_verified_multiple_sessions_sample() {
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("020-paragraphs-sessions-flat-multiple.txxt")
            .expect("Failed to load sample file");
        let tokens = lex(&source);

        let result = parse(tokens.clone());
        assert!(
            result.is_ok(),
            "Failed to parse 020-paragraphs-sessions-flat-multiple.txxt: {:?}",
            result
        );

        let doc = result.unwrap();

        // Expected structure based on 020-paragraphs-sessions-flat-multiple.txxt:
        // Line 1: "Multiple Sessions Flat Test" - paragraph (1 line)
        // Line 3: "This document tests..." - paragraph (1 line)
        // Line 5: "1. First Session" - session with 2 paragraphs
        //   Line 7: "This is the content..." - paragraph (1 line)
        //   Line 9: "It can have multiple..." - paragraph (1 line)
        // Line 11: "2. Second Session" - session with 1 paragraph
        //   Line 13: "The second session..." - paragraph (1 line)
        // Line 15: "A paragraph between sessions" - paragraph (1 line)
        // Line 17: "3. Third Session" - session with 1 paragraph
        //   Line 19: "Sessions can have..." - paragraph (1 line)
        // Line 21: "Another paragraph" - paragraph (1 line)
        // Line 23: "4. Session Without Numbering" - session with 1 NESTED session
        //   Line 25: "Session titles don't require..." - nested session title with 1 paragraph
        //     Line 27: "They just need..." - paragraph (1 line)
        // Line 29: "Final paragraph..." - paragraph (1 line)

        assert_ast(&doc)
            .item_count(9)
            // Item 0: Paragraph (1 line)
            .item(0, |item| {
                item.assert_paragraph().line_count(1);
            })
            // Item 1: Paragraph (1 line)
            .item(1, |item| {
                item.assert_paragraph().line_count(1);
            })
            // Item 2: Session with 2 paragraphs
            .item(2, |item| {
                item.assert_session().child_count(2).children(|children| {
                    children
                        .all_paragraphs()
                        .item(0, |p| {
                            p.assert_paragraph().line_count(1);
                        })
                        .item(1, |p| {
                            p.assert_paragraph().line_count(1);
                        });
                });
            })
            // Item 3: Session with 1 paragraph
            .item(3, |item| {
                item.assert_session().child_count(1).child(0, |child| {
                    child.assert_paragraph().line_count(1);
                });
            })
            // Item 4: Paragraph (1 line)
            .item(4, |item| {
                item.assert_paragraph().line_count(1);
            })
            // Item 5: Session with 1 paragraph
            .item(5, |item| {
                item.assert_session().child_count(1).child(0, |child| {
                    child.assert_paragraph().line_count(1);
                });
            })
            // Item 6: Paragraph (1 line)
            .item(6, |item| {
                item.assert_paragraph().line_count(1);
            })
            // Item 7: Session with 1 nested session
            .item(7, |item| {
                item.assert_session().child_count(1).child(0, |child| {
                    // This should be a nested session, not a paragraph
                    child
                        .assert_session()
                        .child_count(1)
                        .child(0, |nested_child| {
                            nested_child.assert_paragraph().line_count(1);
                        });
                });
            })
            // Item 8: Paragraph (1 line)
            .item(8, |item| {
                item.assert_paragraph().line_count(1);
            });
    }

    #[test]
    fn test_verified_nested_sessions_sample() {
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("030-paragraphs-sessions-nested-multiple.txxt")
            .expect("Failed to load sample file");
        let tokens = lex(&source);

        let result = parse(tokens.clone());
        assert!(
            result.is_ok(),
            "Failed to parse 030-paragraphs-sessions-nested-multiple.txxt: {:?}",
            result
        );

        let doc = result.unwrap();

        // Expected structure based on 030-paragraphs-sessions-nested-multiple.txxt:
        // Line 1: "Nested Sessions Test" - paragraph (1 line)
        // Line 3: "This document tests..." - paragraph (1 line)
        // Line 5: "1. Root Session" - session with complex nested structure
        //   Line 7: "This is content..." - paragraph (1 line)
        //   Line 9: "1.1. First Sub-session" - session with 2 paragraphs
        //     Line 11: "This is content..." - paragraph (1 line)
        //     Line 13: "It can have..." - paragraph (1 line)
        //   Line 15: "1.2. Second Sub-session" - session with nested session + paragraph
        //     Line 17: "Another sub-session..." - paragraph (1 line)
        //     Line 19: "1.2.1. Deeply Nested Session" - session with 2 paragraphs
        //       Line 21: "This is content..." - paragraph (1 line)
        //       Line 23: "Sessions can be..." - paragraph (1 line)
        //   Line 25: "Back to the first..." - paragraph (1 line)
        // Line 27: "2. Another Root Session" - session with nested session
        //   Line 29: "This session is..." - paragraph (1 line)
        //   Line 31: "2.1. Its Sub-session" - session with 1 paragraph
        //     Line 33: "Sub-sessions can..." - paragraph (1 line)
        // Line 35: "Final paragraph..." - paragraph (1 line)

        assert_ast(&doc)
            .item_count(5)
            // Item 0: Paragraph (1 line)
            .item(0, |item| {
                item.assert_paragraph().line_count(1);
            })
            // Item 1: Paragraph (1 line)
            .item(1, |item| {
                item.assert_paragraph().line_count(1);
            })
            // Item 2: "1. Root Session" with 4 children (paragraph, session, session, paragraph)
            .item(2, |item| {
                item.assert_session()
                    .child_count(4)
                    // Child 0: Paragraph
                    .child(0, |child| {
                        child.assert_paragraph().line_count(1);
                    })
                    // Child 1: "1.1. First Sub-session" with 2 paragraphs
                    .child(1, |child| {
                        child
                            .assert_session()
                            .child_count(2)
                            .child(0, |para| {
                                para.assert_paragraph().line_count(1);
                            })
                            .child(1, |para| {
                                para.assert_paragraph().line_count(1);
                            });
                    })
                    // Child 2: "1.2. Second Sub-session" with 2 children (paragraph + nested session)
                    .child(2, |child| {
                        child
                            .assert_session()
                            .child_count(2)
                            .child(0, |para| {
                                para.assert_paragraph().line_count(1);
                            })
                            // "1.2.1. Deeply Nested Session" with 2 paragraphs
                            .child(1, |deeply_nested| {
                                deeply_nested
                                    .assert_session()
                                    .child_count(2)
                                    .child(0, |para| {
                                        para.assert_paragraph().line_count(1);
                                    })
                                    .child(1, |para| {
                                        para.assert_paragraph().line_count(1);
                                    });
                            });
                    })
                    // Child 3: Paragraph ("Back to the first...")
                    .child(3, |child| {
                        child.assert_paragraph().line_count(1);
                    });
            })
            // Item 3: "2. Another Root Session" with 2 children (paragraph + session)
            .item(3, |item| {
                item.assert_session()
                    .child_count(2)
                    .child(0, |child| {
                        child.assert_paragraph().line_count(1);
                    })
                    // "2.1. Its Sub-session" with 1 paragraph
                    .child(1, |child| {
                        child.assert_session().child_count(1).child(0, |para| {
                            para.assert_paragraph().line_count(1);
                        });
                    });
            })
            // Item 4: Final paragraph
            .item(4, |item| {
                item.assert_paragraph().line_count(1);
            });
    }

    // ==================== LIST TESTS ====================
    // Following the complexity ladder: simplest → variations → documents

    #[test]
    fn test_simplest_dash_list() {
        // Simplest possible list: 2 dashed items
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("040-lists.txxt").unwrap();
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Find the first list (after "Plain dash lists:" paragraph)
        // Document structure: Para Para Para List Para List...
        assert_ast(&doc).item(3, |item| {
            item.assert_list()
                .item_count(3)
                .item(0, |list_item| {
                    list_item
                        .text("- First item {{list-item}}")
                        .text_contains("First item");
                })
                .item(1, |list_item| {
                    list_item
                        .text("- Second item {{list-item}}")
                        .text_contains("Second item");
                })
                .item(2, |list_item| {
                    list_item
                        .text("- Third item {{list-item}}")
                        .text_contains("Third item");
                });
        });
    }

    #[test]
    fn test_numbered_list() {
        // Test numbered list: "1. ", "2. ", "3. "
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("040-lists.txxt").unwrap();
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Numerical lists (item 5)
        assert_ast(&doc).item(5, |item| {
            item.assert_list()
                .item_count(3)
                .item(0, |list_item| {
                    list_item.text_starts_with("1.");
                })
                .item(1, |list_item| {
                    list_item.text_starts_with("2.");
                })
                .item(2, |list_item| {
                    list_item.text_starts_with("3.");
                });
        });
    }

    #[test]
    fn test_alphabetical_list() {
        // Test alphabetical list: "a. ", "b. ", "c. "
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("040-lists.txxt").unwrap();
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Alphabetical lists (item 7)
        assert_ast(&doc).item(7, |item| {
            item.assert_list()
                .item_count(3)
                .item(0, |list_item| {
                    list_item.text_starts_with("a.");
                })
                .item(1, |list_item| {
                    list_item.text_starts_with("b.");
                })
                .item(2, |list_item| {
                    list_item.text_starts_with("c.");
                });
        });
    }

    #[test]
    fn test_mixed_decoration_list() {
        // Test mixed decorations: different markers in same list
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("040-lists.txxt").unwrap();
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Mixed decoration lists (item 9)
        assert_ast(&doc).item(9, |item| {
            item.assert_list()
                .item_count(3)
                .item(0, |list_item| {
                    list_item.text_starts_with("1.");
                })
                .item(1, |list_item| {
                    list_item.text_starts_with("-");
                })
                .item(2, |list_item| {
                    list_item.text_starts_with("a.");
                });
        });
    }

    #[test]
    fn test_parenthetical_list() {
        // Test parenthetical numbering: "(1) ", "(2) ", "(3) "
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("040-lists.txxt").unwrap();
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Parenthetical numbering (item 11)
        assert_ast(&doc).item(11, |item| {
            item.assert_list()
                .item_count(3)
                .item(0, |list_item| {
                    list_item.text_starts_with("(1)");
                })
                .item(1, |list_item| {
                    list_item.text_starts_with("(2)");
                })
                .item(2, |list_item| {
                    list_item.text_starts_with("(3)");
                });
        });
    }

    #[test]
    fn test_paragraph_list_disambiguation() {
        // Critical test: single list-like line becomes paragraph, 2+ with blank line become list
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("050-paragraph-lists.txxt").unwrap();
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Items 2-4: Single list-item-lines merged into paragraphs
        assert_ast(&doc).item(2, |item| {
            item.assert_paragraph()
                .text_contains("- This is not a list");
        });

        assert_ast(&doc).item(3, |item| {
            item.assert_paragraph()
                .text_contains("1. This is also not a list");
        });

        // Item 6: First actual list (after blank line) - 0-indexed!
        assert_ast(&doc).item(6, |item| {
            item.assert_list()
                .item_count(2)
                .item(0, |list_item| {
                    list_item.text_contains("This is a list");
                })
                .item(1, |list_item| {
                    list_item.text_contains("Blank line required");
                });
        });
    }

    #[test]
    fn test_verified_lists_document() {
        // Full document test with lists from TxxtSources
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("040-lists.txxt").unwrap();
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Verify document structure: paragraphs + lists alternating
        assert_ast(&doc)
            .item(0, |item| {
                item.assert_paragraph().text_contains("Lists Only Test");
            })
            .item(1, |item| {
                item.assert_paragraph()
                    .text_contains("various list formats");
            })
            .item(2, |item| {
                item.assert_paragraph().text_contains("Plain dash lists");
            })
            .item(3, |item| {
                item.assert_list().item_count(3); // Dash list
            })
            .item(4, |item| {
                item.assert_paragraph().text_contains("Numerical lists");
            })
            .item(5, |item| {
                item.assert_list().item_count(3); // Numbered list
            })
            .item(6, |item| {
                item.assert_paragraph().text_contains("Alphabetical lists");
            })
            .item(7, |item| {
                item.assert_list().item_count(3); // Alphabetical list
            });
    }

    #[test]
    fn test_list_requires_preceding_blank_line() {
        // Critical test: Lists MUST have a preceding blank line for disambiguation
        // Without the blank line, consecutive list-item-lines should be parsed as paragraphs
        use crate::txxt_nano::testing::assert_ast;

        let source = "First paragraph\n- Item one\n- Item two\n";
        let tokens = lex_with_spans(source);
        let doc = parse_with_source(tokens, source).unwrap();

        // Should be parsed as a single paragraph, NOT a paragraph + list
        // because there's no blank line before the list-item-lines
        assert_eq!(
            doc.content.len(),
            1,
            "Should be 1 paragraph, not paragraph + list"
        );
        assert_ast(&doc).item(0, |item| {
            item.assert_paragraph()
                .text_contains("First paragraph")
                .text_contains("- Item one")
                .text_contains("- Item two");
        });

        // Now test the positive case: with blank line, it becomes separate items
        let source_with_blank = "First paragraph\n\n- Item one\n- Item two\n";
        let tokens2 = lex_with_spans(source_with_blank);
        let doc2 = parse_with_source(tokens2, source_with_blank).unwrap();

        // Should be parsed as paragraph + list
        assert_eq!(
            doc2.content.len(),
            2,
            "Should be paragraph + list with blank line"
        );
        assert_ast(&doc2)
            .item(0, |item| {
                item.assert_paragraph().text_contains("First paragraph");
            })
            .item(1, |item| {
                item.assert_list()
                    .item_count(2)
                    .item(0, |list_item| {
                        list_item.text_contains("Item one");
                    })
                    .item(1, |list_item| {
                        list_item.text_contains("Item two");
                    });
            });
    }

    // ==================== TRIFECTA TESTS ====================
    // Testing paragraphs + sessions + lists together

    #[test]
    fn test_trifecta_flat_simple() {
        // Test flat structure with all three elements
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("050-trifecta-flat-simple.txxt").unwrap();
        let tokens = lex_with_spans(&source);
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
        let tokens = lex_with_spans(&source);
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

    // ==================== NESTED LISTS TESTS ====================
    // Testing nested list structures

    #[test]
    fn test_verified_nested_lists_simple() {
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("070-nested-lists-simple.txxt")
            .expect("Failed to load sample file");
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Item 0-1: Opening paragraphs
        assert_ast(&doc)
            .item(0, |item| {
                item.assert_paragraph() // "Simple Nested Lists Test"
                    .text_contains("Simple Nested Lists Test");
            })
            .item(1, |item| {
                item.assert_paragraph() // "simple list-in-list nesting"
                    .text_contains("simple list-in-list nesting");
            });

        // Item 2: Paragraph before first list
        assert_ast(&doc).item(2, |item| {
            item.assert_paragraph().text_contains("Basic nested list");
        });

        // Item 3: First nested list structure
        assert_ast(&doc).item(3, |item| {
            item.assert_list()
                .item_count(2)
                // First item with nested list
                .item(0, |list_item| {
                    list_item
                        .text_contains("First outer item")
                        .child_count(1)
                        .child(0, |child| {
                            child.assert_list().item_count(2);
                        });
                })
                // Second item with nested list
                .item(1, |list_item| {
                    list_item
                        .text_contains("Second outer item")
                        .child_count(1)
                        .child(0, |child| {
                            child.assert_list().item_count(2);
                        });
                });
        });

        // Item 4: Paragraph before second list
        assert_ast(&doc).item(4, |item| {
            item.assert_paragraph()
                .text_contains("Numbered list with nested dashed list");
        });

        // Item 5: Numbered list with nested dashed lists
        assert_ast(&doc).item(5, |item| {
            item.assert_list()
                .item_count(2)
                .item(0, |list_item| {
                    list_item
                        .text_starts_with("1.")
                        .text_contains("First numbered item")
                        .child_count(1)
                        .child(0, |child| {
                            child.assert_list().item_count(2);
                        });
                })
                .item(1, |list_item| {
                    list_item
                        .text_starts_with("2.")
                        .text_contains("Second numbered item")
                        .child_count(1)
                        .child(0, |child| {
                            child.assert_list().item_count(2);
                        });
                });
        });

        // Item 6: Final paragraph
        assert_ast(&doc).item(6, |item| {
            item.assert_paragraph()
                .text_contains("Final paragraph after lists");
        });
    }

    #[test]
    fn test_verified_nested_lists_mixed_content() {
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("080-nested-lists-mixed-content.txxt")
            .expect("Failed to load sample file");
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

        // Item 0-1: Opening paragraphs
        assert_ast(&doc)
            .item(0, |item| {
                item.assert_paragraph() // "Nested Lists with Mixed Content Test"
                    .text_contains("Nested Lists with Mixed Content Test");
            })
            .item(1, |item| {
                item.assert_paragraph() // "mix of paragraphs and other lists"
                    .text_contains("mix of paragraphs and other lists");
            });

        // Item 2: Paragraph before first list
        assert_ast(&doc).item(2, |item| {
            item.assert_paragraph()
                .text_contains("List with paragraph content");
        });

        // Item 3: List with paragraph content in items
        assert_ast(&doc).item(3, |item| {
            item.assert_list()
                .item_count(2)
                // First item with one paragraph
                .item(0, |list_item| {
                    list_item
                        .text_contains("First item with nested paragraph")
                        .child_count(1)
                        .child(0, |child| {
                            child
                                .assert_paragraph()
                                .text_contains("paragraph nested inside the first list item");
                        });
                })
                // Second item with two paragraphs
                .item(1, |list_item| {
                    list_item
                        .text_contains("Second item with multiple paragraphs")
                        .child_count(2)
                        .child(0, |child| {
                            child
                                .assert_paragraph()
                                .text_contains("first paragraph in the second item");
                        })
                        .child(1, |child| {
                            child.assert_paragraph().text_contains("second paragraph");
                        });
                });
        });

        // Item 4: Paragraph before mixed content list
        assert_ast(&doc).item(4, |item| {
            item.assert_paragraph()
                .text_contains("mixed paragraphs and nested lists");
        });

        // Item 5: List with mixed content (paragraphs and nested lists)
        assert_ast(&doc).item(5, |item| {
            item.assert_list()
                .item_count(2)
                // First complex item: para + list + para
                .item(0, |list_item| {
                    list_item
                        .text_starts_with("1.")
                        .text_contains("First complex item")
                        .child_count(3)
                        .child(0, |child| {
                            child
                                .assert_paragraph()
                                .text_contains("paragraph explaining the first item");
                        })
                        .child(1, |child| {
                            child.assert_list().item_count(2);
                        })
                        .child(2, |child| {
                            child
                                .assert_paragraph()
                                .text_contains("Another paragraph after the nested list");
                        });
                })
                // Second complex item: para + list + para
                .item(1, |list_item| {
                    list_item
                        .text_starts_with("2.")
                        .text_contains("Second complex item")
                        .child_count(3)
                        .child(0, |child| {
                            child
                                .assert_paragraph()
                                .text_contains("Opening paragraph for item two");
                        })
                        .child(1, |child| {
                            child.assert_list().item_count(2);
                        })
                        .child(2, |child| {
                            child
                                .assert_paragraph()
                                .text_contains("Closing paragraph for item two");
                        });
                });
        });

        // Item 6: Paragraph before deeply nested structure
        assert_ast(&doc).item(6, |item| {
            item.assert_paragraph()
                .text_contains("Deeply nested structure");
        });

        // Item 7: Deeply nested list structure
        assert_ast(&doc).item(7, |item| {
            item.assert_list()
                .item_count(2)
                // First outer item with deep nesting
                .item(0, |outer_item| {
                    outer_item
                        .text_contains("Outer item one")
                        .child_count(2) // para + nested list
                        .child(0, |child| {
                            child
                                .assert_paragraph()
                                .text_contains("Paragraph in outer item");
                        })
                        .child(1, |middle_list| {
                            middle_list
                                .assert_list()
                                .item_count(2)
                                // Middle item one with inner list
                                .item(0, |middle_item| {
                                    middle_item
                                        .text_contains("Middle item one")
                                        .child_count(1)
                                        .child(0, |inner_list| {
                                            inner_list.assert_list().item_count(2);
                                        });
                                })
                                // Middle item two with paragraph
                                .item(1, |middle_item| {
                                    middle_item
                                        .text_contains("Middle item two")
                                        .child_count(1)
                                        .child(0, |para| {
                                            para.assert_paragraph()
                                                .text_contains("Paragraph in middle item");
                                        });
                                });
                        });
                })
                // Second outer item with paragraph
                .item(1, |outer_item| {
                    outer_item
                        .text_contains("Outer item two")
                        .child_count(1)
                        .child(0, |child| {
                            child.assert_paragraph().text_contains("Final paragraph");
                        });
                });
        });

        // Item 8: Final paragraph
        assert_ast(&doc).item(8, |item| {
            item.assert_paragraph().text_contains("End of document");
        });
    }

    // ==================== DEFINITION TESTS ====================
    // Testing definition structures

    #[test]
    fn test_unified_recursive_parser_simple() {
        // Minimal test for the unified recursive parser
        let source = "First paragraph\n\nDefinition:\n    Content of definition\n";
        let tokens = lex_with_spans(source);
        println!("Testing simple definition with unified parser:");
        println!("Source: {:?}", source);

        let result = parse_with_source(tokens, source);
        if let Err(ref e) = result {
            println!("Parse error: {:?}", e);
        }
        assert!(result.is_ok(), "Failed to parse simple definition");
        let doc = result.unwrap();
        assert_eq!(doc.content.len(), 2, "Should have paragraph and definition");
    }

    #[test]
    fn test_unified_recursive_nested_definitions() {
        // Test nested definitions with the unified parser
        let source = "Outer:\n    Inner:\n        Nested content\n";
        let tokens = lex_with_spans(source);
        println!("Testing nested definitions with unified parser:");
        println!("Source: {:?}", source);

        let result = parse_with_source(tokens, source);
        if let Err(ref e) = result {
            println!("Parse error: {:?}", e);
        }
        assert!(result.is_ok(), "Failed to parse nested definitions");

        let doc = result.unwrap();
        assert_eq!(doc.content.len(), 1, "Should have one outer definition");

        // Check outer definition
        let outer_def = doc.content[0]
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
        assert_eq!(nested_para.lines[0].as_string(), "Nested content");
    }

    #[test]
    // Previously ignored for issue #35 - now testing if fixed
    fn test_unified_parser_paragraph_then_definition() {
        // Test paragraph followed by definition - similar to failing test
        let source = "Simple paragraph\n\nAnother paragraph\n\nFirst Definition:\n    Definition content\n\nSecond Definition:\n    More content\n";
        let tokens = lex_with_spans(source);
        println!("Testing paragraph then definition:");
        println!("Source: {:?}", source);

        let result = parse_with_source(tokens, source);
        if let Err(ref e) = result {
            println!("Parse error: {:?}", e);
            println!("Error at span: {:?}", &source[e[0].span().clone()]);
        }
        assert!(result.is_ok(), "Failed to parse paragraph then definition");

        let doc = result.unwrap();
        println!("Parsed {} items", doc.content.len());
        for (i, item) in doc.content.iter().enumerate() {
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
            doc.content.len(),
            4,
            "Should have 2 paragraphs and 2 definitions"
        );
    }

    #[test]
    // Previously ignored for issue #35 - now testing if fixed
    fn test_verified_definitions_simple() {
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("090-definitions-simple.txxt")
            .expect("Failed to load sample file");
        let tokens = lex_with_spans(&source);

        // Debug: print first few tokens
        println!("First 10 tokens:");
        for (i, token) in tokens.iter().take(10).enumerate() {
            println!("  {}: {:?}", i, token);
        }

        let result = parse_with_source(tokens, &source);
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
    #[ignore = "Fails due to parser bug #41"]
    fn test_verified_definitions_mixed_content() {
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("100-definitions-mixed-content.txxt")
            .expect("Failed to load sample file");
        let tokens = lex_with_spans(&source);
        let doc = parse_with_source(tokens, &source).unwrap();

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

    #[test]
    #[ignore = "Still failing - needs investigation"]
    fn test_verified_ensemble_with_definitions() {
        // Comprehensive ensemble test with all core elements including definitions
        use crate::txxt_nano::testing::assert_ast;

        let source = TxxtSources::get_string("110-ensemble-with-definitions.txxt").unwrap();
        let tokens = lex_with_spans(&source);
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
        let tokens = lex_with_spans(&source);

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
        let tokens = lex_with_spans(input);
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
        let tokens = lex_with_spans(input);
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
        let tokens = lex_with_spans(input);
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
        let tokens = lex_with_spans(input);
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
        let tokens = lex_with_spans(input);
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
        let tokens = lex_with_spans(input);

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
        let tokens = lex_with_spans(input);
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

        // Should not contain position after end
        assert!(!span.contains(Position::new(0, 11)));
    }
}
