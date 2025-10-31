//! List element parsing
//!
//! This module handles parsing of list elements in the txxt format.
//! Lists are sequences of list items with optional nested content.

use chumsky::prelude::*;
use chumsky::primitive::filter;
use std::ops::Range;
use std::sync::Arc;

use crate::txxt::ast::location::SourceLocation;
use crate::txxt::ast::{AstNode, ContentItem, List, ListItem, Location, TextContent};
use crate::txxt::lexer::Token;
use crate::txxt::parser::combinators::{
    compute_location_from_optional_locations, extract_tokens_to_text_and_location, is_text_token,
    token,
};

/// Type alias for token with location
type TokenLocation = (Token, Range<usize>);

/// Type alias for parser error
type ParserError = Simple<TokenLocation>;

/// Parse a list item line - a line that starts with a list marker
/// Phase 5: Now returns extracted text with location information
pub(crate) fn list_item_line(
    source: Arc<String>,
) -> impl Parser<TokenLocation, (String, Range<usize>), Error = ParserError> + Clone {
    let rest_of_line = filter(|(t, _location): &TokenLocation| is_text_token(t)).repeated();

    let dash_pattern = filter(|(t, _): &TokenLocation| matches!(t, Token::Dash))
        .then(filter(|(t, _): &TokenLocation| {
            matches!(t, Token::Whitespace)
        }))
        .chain(rest_of_line);

    let ordered_pattern =
        filter(|(t, _): &TokenLocation| matches!(t, Token::Number(_) | Token::Text(_)))
            .then(filter(|(t, _): &TokenLocation| {
                matches!(t, Token::Period | Token::CloseParen)
            }))
            .then(filter(|(t, _): &TokenLocation| {
                matches!(t, Token::Whitespace)
            }))
            .chain(rest_of_line);

    let paren_pattern = filter(|(t, _): &TokenLocation| matches!(t, Token::OpenParen))
        .then(filter(|(t, _): &TokenLocation| {
            matches!(t, Token::Number(_))
        }))
        .then(filter(|(t, _): &TokenLocation| {
            matches!(t, Token::CloseParen)
        }))
        .then(filter(|(t, _): &TokenLocation| {
            matches!(t, Token::Whitespace)
        }))
        .chain(rest_of_line);

    dash_pattern
        .or(ordered_pattern)
        .or(paren_pattern)
        .map(move |tokens_with_locations| {
            extract_tokens_to_text_and_location(&source, tokens_with_locations)
        })
}

/// Helper: convert a byte range to a location using source location
fn byte_range_to_location(source: &str, range: &Range<usize>) -> Option<Location> {
    if range.start > range.end {
        return None;
    }
    let source_loc = SourceLocation::new(source);
    Some(source_loc.range_to_location(range))
}

/// Build a list parser
pub(crate) fn build_list_parser<P>(
    source: Arc<String>,
    items: P,
) -> impl Parser<TokenLocation, ContentItem, Error = ParserError> + Clone
where
    P: Parser<TokenLocation, Vec<ContentItem>, Error = ParserError> + Clone + 'static,
{
    let source_for_list = source.clone();
    let single_list_item = list_item_line(source.clone())
        .then_ignore(token(Token::Newline))
        .then(
            token(Token::IndentLevel)
                .ignore_then(items)
                .then_ignore(token(Token::DedentLevel))
                .or_not(),
        )
        .map(move |((text, text_location), maybe_content)| {
            let content = maybe_content.unwrap_or_default();
            let line_location = byte_range_to_location(&source_for_list, &text_location);
            let text_content = TextContent::from_string(text, line_location);

            let mut location_sources = vec![line_location];
            location_sources.extend(content.iter().map(|item| item.location()));
            let location = compute_location_from_optional_locations(&location_sources);

            ListItem::with_text_content(text_content, content).with_location(location)
        });

    single_list_item.repeated().at_least(2).map(|items| {
        let locations: Vec<Option<Location>> = items.iter().map(|item| item.location).collect();
        let location = compute_location_from_optional_locations(&locations);
        let content_items: Vec<ContentItem> =
            items.into_iter().map(ContentItem::ListItem).collect();
        ContentItem::List(List {
            content: content_items,
            location,
        })
    })
}

#[cfg(test)]
mod tests {
    use crate::txxt::lexer::lex_with_locations;
    use crate::txxt::parser::api::parse_with_source;
    use crate::txxt::processor::txxt_sources::TxxtSources;
    use crate::txxt::testing::assert_ast;

    #[test]
    fn test_simplest_dash_list() {
        // Simplest possible list: 2 dashed items
        let source = TxxtSources::get_string("040-lists.txxt").unwrap();
        let tokens = lex_with_locations(&source);
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
        let source = TxxtSources::get_string("040-lists.txxt").unwrap();
        let tokens = lex_with_locations(&source);
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
        let source = TxxtSources::get_string("040-lists.txxt").unwrap();
        let tokens = lex_with_locations(&source);
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
        let source = TxxtSources::get_string("040-lists.txxt").unwrap();
        let tokens = lex_with_locations(&source);
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
        let source = TxxtSources::get_string("040-lists.txxt").unwrap();
        let tokens = lex_with_locations(&source);
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
        let source = TxxtSources::get_string("050-paragraph-lists.txxt").unwrap();
        let tokens = lex_with_locations(&source);
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
        let source = TxxtSources::get_string("040-lists.txxt").unwrap();
        let tokens = lex_with_locations(&source);
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
        let source = "First paragraph\n- Item one\n- Item two\n";
        let tokens = lex_with_locations(source);
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
        let tokens2 = lex_with_locations(source_with_blank);
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

    #[test]
    fn test_verified_nested_lists_simple() {
        let source = TxxtSources::get_string("070-nested-lists-simple.txxt")
            .expect("Failed to load sample file");
        let tokens = lex_with_locations(&source);
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
        let source = TxxtSources::get_string("080-nested-lists-mixed-content.txxt")
            .expect("Failed to load sample file");
        let tokens = lex_with_locations(&source);
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
}
