//! Session element parsing
//!
//! This module handles parsing of session elements in the txxt format.
//! Sessions are hierarchical containers with a title and nested content.

use chumsky::prelude::*;
use std::ops::Range;
use std::sync::Arc;

use crate::txxt_nano::ast::position::SourceLocation;
use crate::txxt_nano::ast::{ContentItem, Location, Session, TextContent};
use crate::txxt_nano::lexer::Token;
use crate::txxt_nano::parser::combinators::{session_title, token};

/// Type alias for token with span
type TokenSpan = (Token, Range<usize>);

/// Type alias for parser error
type ParserError = Simple<TokenSpan>;

/// Helper: convert a byte range to a Span using source location
fn byte_range_to_location(source: &str, range: &Range<usize>) -> Option<Location> {
    if range.start > range.end {
        return None;
    }
    let source_loc = SourceLocation::new(source);
    Some(source_loc.range_to_location(range))
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
        .map(move |((title_text, title_location), content)| {
            let span = byte_range_to_location(&source_for_session, &title_location);
            ContentItem::Session(Session {
                title: TextContent::from_string(title_text, None),
                content,
                span,
            })
        })
}

#[cfg(test)]
mod tests {
    use crate::txxt_nano::ast::Container;
    use crate::txxt_nano::ast::ContentItem;
    use crate::txxt_nano::lexer::{lex, Token};
    use crate::txxt_nano::parser::parser::parse;
    use crate::txxt_nano::processor::txxt_sources::TxxtSources;
    use crate::txxt_nano::testing::assert_ast;

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
    fn test_verified_single_session_sample() {
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
}
