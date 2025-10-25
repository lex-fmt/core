//! Parser implementation for the txxt format using chumsky
//!
//! This module implements a parser combinator-based parser for txxt documents.
//! It builds on the token stream from the lexer and produces an AST.

use chumsky::prelude::*;

use super::ast::{ContentItem, Document, Paragraph, Session};
use crate::txxt_nano::lexer::Token;

/// Parse a text line (sequence of text and whitespace tokens)
fn text_line() -> impl Parser<Token, String, Error = Simple<Token>> {
    filter(|t: &Token| {
        matches!(
            t,
            Token::Text
                | Token::Whitespace
                | Token::Number
                | Token::Dash
                | Token::Period
                | Token::OpenParen
                | Token::CloseParen
                | Token::Colon
        )
    })
    .repeated()
    .at_least(1)
    .map(|tokens| {
        tokens
            .iter()
            .map(|t| match t {
                Token::Text => "text",
                Token::Whitespace => " ",
                Token::Number => "num",
                Token::Dash => "-",
                Token::Period => ".",
                Token::OpenParen => "(",
                Token::CloseParen => ")",
                Token::Colon => ":",
                _ => "",
            })
            .collect::<String>()
            .trim()
            .to_string()
    })
}

/// Parse a paragraph - one or more lines of text separated by newlines, ending with a blank line
fn paragraph() -> impl Parser<Token, Paragraph, Error = Simple<Token>> {
    text_line()
        .then_ignore(just(Token::Newline))
        .repeated()
        .at_least(1)
        .then_ignore(just(Token::Newline).or_not()) // Optional blank line at end
        .map(Paragraph::new)
}

/// Parse a session title - a line of text followed by a newline and blank line
fn session_title() -> impl Parser<Token, String, Error = Simple<Token>> {
    text_line()
        .then_ignore(just(Token::Newline))
        .then_ignore(just(Token::Newline))
}

/// Parse a session - a title followed by indented content
fn session() -> impl Parser<Token, Session, Error = Simple<Token>> + Clone {
    recursive(|session_parser| {
        // Try session first (before paragraph) to handle nested sessions
        let content_item = session_parser
            .map(ContentItem::Session)
            .or(paragraph().map(ContentItem::Paragraph));

        session_title()
            .then(
                just(Token::IndentLevel)
                    .ignore_then(content_item.repeated())
                    .then_ignore(just(Token::DedentLevel)),
            )
            .map(|(title, content)| Session::new(title, content))
    })
}

/// Parse a document - a sequence of paragraphs and sessions
pub fn document() -> impl Parser<Token, Document, Error = Simple<Token>> {
    // Try session first, as a session title looks like a paragraph
    let content_item = session()
        .map(ContentItem::Session)
        .or(paragraph().map(ContentItem::Paragraph));

    content_item
        .repeated()
        .then_ignore(just(Token::DedentLevel).repeated()) // Consume any trailing dedents
        .then_ignore(end())
        .map(Document::with_items)
}

/// Parse a txxt document from a token stream
pub fn parse(tokens: Vec<Token>) -> Result<Document, Vec<Simple<Token>>> {
    document().parse(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::txxt_nano::lexer::lex;
    use crate::txxt_nano::processor::txxt_sources::TxxtSources;

    /// Helper to verify a content item matches expected type and structure
    fn verify_item(item: &ContentItem, path: &str, expected_type: &str, expected_details: &str) {
        match (item, expected_type) {
            (ContentItem::Paragraph(p), "Paragraph") => {
                // expected_details should be line count like "1" or "3"
                if let Ok(expected_lines) = expected_details.parse::<usize>() {
                    assert_eq!(
                        p.lines.len(),
                        expected_lines,
                        "{}: Expected Paragraph with {} lines, got {} lines.\nLines: {:?}",
                        path,
                        expected_lines,
                        p.lines.len(),
                        p.lines
                    );
                } else {
                    // Just verify it's a paragraph
                    // expected_details might be descriptive text
                }
            }
            (ContentItem::Session(s), "Session") => {
                // expected_details should be child count like "2" or "nested structure"
                if let Ok(expected_children) = expected_details.parse::<usize>() {
                    assert_eq!(
                        s.content.len(),
                        expected_children,
                        "{}: Expected Session with {} children, got {} children.\nChildren: {:?}",
                        path,
                        expected_children,
                        s.content.len(),
                        s.content
                            .iter()
                            .enumerate()
                            .map(|(i, item)| match item {
                                ContentItem::Paragraph(p) =>
                                    format!("  {}: Paragraph ({} lines)", i, p.lines.len()),
                                ContentItem::Session(s) =>
                                    format!("  {}: Session ({} children)", i, s.content.len()),
                            })
                            .collect::<Vec<_>>()
                            .join("\n")
                    );
                }
            }
            (ContentItem::Paragraph(p), expected) => {
                panic!(
                    "{}: Expected {}, got Paragraph with {} lines",
                    path,
                    expected,
                    p.lines.len()
                );
            }
            (ContentItem::Session(s), expected) => {
                panic!(
                    "{}: Expected {}, got Session with {} children",
                    path,
                    expected,
                    s.content.len()
                );
            }
        }
    }

    #[test]
    fn test_simple_paragraph() {
        let input = "Hello world\n\n";
        let tokens = lex(input);

        let result = paragraph().parse(tokens);
        assert!(result.is_ok(), "Failed to parse paragraph: {:?}", result);

        let para = result.unwrap();
        assert_eq!(para.lines.len(), 1);
    }

    #[test]
    fn test_verified_paragraphs_sample() {
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
        let expected_structure = [
            ("Paragraph", 1), // "Simple Paragraphs Test"
            ("Paragraph", 1), // "This is a simple paragraph with just one line."
            ("Paragraph", 3), // Multi-line paragraph
            ("Paragraph", 1), // "Another paragraph follows..."
            ("Paragraph", 1), // Paragraph with special chars
            ("Paragraph", 1), // Paragraph with numbers
            ("Paragraph", 1), // Paragraph with mixed content
        ];

        assert_eq!(
            doc.items.len(),
            expected_structure.len(),
            "Expected {} paragraphs, got {}.\n\nExpected structure:\n{}\n\nActual structure:\n{}",
            expected_structure.len(),
            doc.items.len(),
            expected_structure
                .iter()
                .enumerate()
                .map(|(i, (t, lines))| format!("  {}: {} with {} lines", i, t, lines))
                .collect::<Vec<_>>()
                .join("\n"),
            doc.items
                .iter()
                .enumerate()
                .map(|(i, item)| match item {
                    ContentItem::Paragraph(p) =>
                        format!("  {}: Paragraph with {} lines", i, p.lines.len()),
                    ContentItem::Session(s) => format!(
                        "  {}: Session '{}' with {} items",
                        i,
                        s.title,
                        s.content.len()
                    ),
                })
                .collect::<Vec<_>>()
                .join("\n")
        );

        // Verify each item matches expected structure
        for (i, (item, (expected_type, expected_lines))) in
            doc.items.iter().zip(expected_structure.iter()).enumerate()
        {
            match item {
                ContentItem::Paragraph(p) => {
                    assert_eq!(
                        expected_type, &"Paragraph",
                        "Item {} should be a {}, but found Paragraph",
                        i, expected_type
                    );
                    assert_eq!(
                        p.lines.len(),
                        *expected_lines,
                        "Item {} (Paragraph) should have {} lines, but has {}.\nLines: {:?}",
                        i,
                        expected_lines,
                        p.lines.len(),
                        p.lines
                    );
                }
                ContentItem::Session(s) => {
                    panic!(
                        "Item {} should be a Paragraph with {} lines, but found Session '{}' with {} items",
                        i, expected_lines, s.title, s.content.len()
                    );
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

        let expected_doc_items = 6;
        assert_eq!(
            doc.items.len(),
            expected_doc_items,
            "Expected {} root-level items, got {}.\n\nExpected:\n  0: Paragraph (1 line)\n  1: Paragraph (1 line)\n  2: Session with 2 paragraphs\n  3: Paragraph (1 line)\n  4: Session with 1 paragraph\n  5: Paragraph (1 line)\n\nActual:\n{}",
            expected_doc_items,
            doc.items.len(),
            doc.items.iter().enumerate()
                .map(|(i, item)| match item {
                    ContentItem::Paragraph(p) => format!("  {}: Paragraph ({} lines)", i, p.lines.len()),
                    ContentItem::Session(s) => format!("  {}: Session with {} items", i, s.content.len()),
                })
                .collect::<Vec<_>>()
                .join("\n")
        );

        // Verify item 0: Paragraph with 1 line
        match &doc.items[0] {
            ContentItem::Paragraph(p) => {
                assert_eq!(
                    p.lines.len(),
                    1,
                    "Item 0: Expected 1 line, got {}. Lines: {:?}",
                    p.lines.len(),
                    p.lines
                );
            }
            ContentItem::Session(s) => {
                panic!(
                    "Item 0: Expected Paragraph, got Session '{}' with {} items",
                    s.title,
                    s.content.len()
                );
            }
        }

        // Verify item 1: Paragraph with 1 line
        match &doc.items[1] {
            ContentItem::Paragraph(p) => {
                assert_eq!(
                    p.lines.len(),
                    1,
                    "Item 1: Expected 1 line, got {}. Lines: {:?}",
                    p.lines.len(),
                    p.lines
                );
            }
            ContentItem::Session(s) => {
                panic!(
                    "Item 1: Expected Paragraph, got Session '{}' with {} items",
                    s.title,
                    s.content.len()
                );
            }
        }

        // Verify item 2: Session with 2 paragraphs
        match &doc.items[2] {
            ContentItem::Session(session) => {
                assert_eq!(
                    session.content.len(), 2,
                    "Item 2: Session should have 2 items, got {}.\n\nExpected:\n  0: Paragraph (1 line)\n  1: Paragraph (1 line)\n\nActual:\n{}",
                    session.content.len(),
                    session.content.iter().enumerate()
                        .map(|(i, item)| match item {
                            ContentItem::Paragraph(p) => format!("  {}: Paragraph ({} lines)", i, p.lines.len()),
                            ContentItem::Session(s) => format!("  {}: Session with {} items", i, s.content.len()),
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                );

                // Verify session's first paragraph
                match &session.content[0] {
                    ContentItem::Paragraph(p) => {
                        assert_eq!(
                            p.lines.len(),
                            1,
                            "Item 2, child 0: Expected 1 line, got {}. Lines: {:?}",
                            p.lines.len(),
                            p.lines
                        );
                    }
                    ContentItem::Session(s) => {
                        panic!(
                            "Item 2, child 0: Expected Paragraph, got Session '{}' with {} items",
                            s.title,
                            s.content.len()
                        );
                    }
                }

                // Verify session's second paragraph
                match &session.content[1] {
                    ContentItem::Paragraph(p) => {
                        assert_eq!(
                            p.lines.len(),
                            1,
                            "Item 2, child 1: Expected 1 line, got {}. Lines: {:?}",
                            p.lines.len(),
                            p.lines
                        );
                    }
                    ContentItem::Session(s) => {
                        panic!(
                            "Item 2, child 1: Expected Paragraph, got Session '{}' with {} items",
                            s.title,
                            s.content.len()
                        );
                    }
                }
            }
            ContentItem::Paragraph(p) => {
                panic!(
                    "Item 2: Expected Session with 2 items, got Paragraph with {} lines",
                    p.lines.len()
                );
            }
        }

        // Verify item 3: Paragraph with 1 line
        match &doc.items[3] {
            ContentItem::Paragraph(p) => {
                assert_eq!(
                    p.lines.len(),
                    1,
                    "Item 3: Expected 1 line, got {}. Lines: {:?}",
                    p.lines.len(),
                    p.lines
                );
            }
            ContentItem::Session(s) => {
                panic!(
                    "Item 3: Expected Paragraph, got Session '{}' with {} items",
                    s.title,
                    s.content.len()
                );
            }
        }

        // Verify item 4: Session with 1 paragraph
        match &doc.items[4] {
            ContentItem::Session(session) => {
                assert_eq!(
                    session.content.len(), 1,
                    "Item 4: Session should have 1 item, got {}.\n\nExpected:\n  0: Paragraph (1 line)\n\nActual:\n{}",
                    session.content.len(),
                    session.content.iter().enumerate()
                        .map(|(i, item)| match item {
                            ContentItem::Paragraph(p) => format!("  {}: Paragraph ({} lines)", i, p.lines.len()),
                            ContentItem::Session(s) => format!("  {}: Session with {} items", i, s.content.len()),
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                );

                // Verify session's paragraph
                match &session.content[0] {
                    ContentItem::Paragraph(p) => {
                        assert_eq!(
                            p.lines.len(),
                            1,
                            "Item 4, child 0: Expected 1 line, got {}. Lines: {:?}",
                            p.lines.len(),
                            p.lines
                        );
                    }
                    ContentItem::Session(s) => {
                        panic!(
                            "Item 4, child 0: Expected Paragraph, got Session '{}' with {} items",
                            s.title,
                            s.content.len()
                        );
                    }
                }
            }
            ContentItem::Paragraph(p) => {
                panic!(
                    "Item 4: Expected Session with 1 item, got Paragraph with {} lines",
                    p.lines.len()
                );
            }
        }

        // Verify item 5: Paragraph with 1 line
        match &doc.items[5] {
            ContentItem::Paragraph(p) => {
                assert_eq!(
                    p.lines.len(),
                    1,
                    "Item 5: Expected 1 line, got {}. Lines: {:?}",
                    p.lines.len(),
                    p.lines
                );
            }
            ContentItem::Session(s) => {
                panic!(
                    "Item 5: Expected Paragraph, got Session '{}' with {} items",
                    s.title,
                    s.content.len()
                );
            }
        }
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

        let expected_items = 9;
        assert_eq!(
            doc.items.len(),
            expected_items,
            "Expected {} root-level items, got {}.\n\nExpected:\n  0: Paragraph (1 line)\n  1: Paragraph (1 line)\n  2: Session (2 paragraphs)\n  3: Session (1 paragraph)\n  4: Paragraph (1 line)\n  5: Session (1 paragraph)\n  6: Paragraph (1 line)\n  7: Session (1 nested session)\n  8: Paragraph (1 line)\n\nActual:\n{}",
            expected_items,
            doc.items.len(),
            doc.items.iter().enumerate()
                .map(|(i, item)| match item {
                    ContentItem::Paragraph(p) => format!("  {}: Paragraph ({} lines)", i, p.lines.len()),
                    ContentItem::Session(s) => format!("  {}: Session ({} items)", i, s.content.len()),
                })
                .collect::<Vec<_>>()
                .join("\n")
        );

        // Item 0: Paragraph (1 line)
        verify_item(&doc.items[0], "Item[0]", "Paragraph", "1");

        // Item 1: Paragraph (1 line)
        verify_item(&doc.items[1], "Item[1]", "Paragraph", "1");

        // Item 2: Session with 2 paragraphs
        verify_item(&doc.items[2], "Item[2]", "Session", "2");
        if let ContentItem::Session(s) = &doc.items[2] {
            verify_item(&s.content[0], "Item[2].content[0]", "Paragraph", "1");
            verify_item(&s.content[1], "Item[2].content[1]", "Paragraph", "1");
        }

        // Item 3: Session with 1 paragraph
        verify_item(&doc.items[3], "Item[3]", "Session", "1");
        if let ContentItem::Session(s) = &doc.items[3] {
            verify_item(&s.content[0], "Item[3].content[0]", "Paragraph", "1");
        }

        // Item 4: Paragraph (1 line)
        verify_item(&doc.items[4], "Item[4]", "Paragraph", "1");

        // Item 5: Session with 1 paragraph
        verify_item(&doc.items[5], "Item[5]", "Session", "1");
        if let ContentItem::Session(s) = &doc.items[5] {
            verify_item(&s.content[0], "Item[5].content[0]", "Paragraph", "1");
        }

        // Item 6: Paragraph (1 line)
        verify_item(&doc.items[6], "Item[6]", "Paragraph", "1");

        // Item 7: Session with 1 nested session
        verify_item(&doc.items[7], "Item[7]", "Session", "1");
        if let ContentItem::Session(s) = &doc.items[7] {
            // This should be a nested session, not a paragraph
            verify_item(&s.content[0], "Item[7].content[0]", "Session", "1");

            // Verify the nested session's content
            if let ContentItem::Session(nested) = &s.content[0] {
                verify_item(
                    &nested.content[0],
                    "Item[7].content[0].content[0]",
                    "Paragraph",
                    "1",
                );
            }
        }

        // Item 8: Paragraph (1 line)
        verify_item(&doc.items[8], "Item[8]", "Paragraph", "1");
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

        let expected_items = 5;
        assert_eq!(
            doc.items.len(),
            expected_items,
            "Expected {} root-level items, got {}.\n\nExpected:\n  0: Paragraph\n  1: Paragraph\n  2: Session (deeply nested)\n  3: Session (with sub-session)\n  4: Paragraph\n\nActual:\n{}",
            expected_items,
            doc.items.len(),
            doc.items.iter().enumerate()
                .map(|(i, item)| match item {
                    ContentItem::Paragraph(p) => format!("  {}: Paragraph ({} lines)", i, p.lines.len()),
                    ContentItem::Session(s) => format!("  {}: Session ({} items)", i, s.content.len()),
                })
                .collect::<Vec<_>>()
                .join("\n")
        );

        // Item 0: Paragraph (1 line)
        verify_item(&doc.items[0], "Item[0]", "Paragraph", "1");

        // Item 1: Paragraph (1 line)
        verify_item(&doc.items[1], "Item[1]", "Paragraph", "1");

        // Item 2: "1. Root Session" with 4 children
        // (paragraph, session, session, paragraph)
        verify_item(&doc.items[2], "Item[2]", "Session", "4");
        if let ContentItem::Session(root_session) = &doc.items[2] {
            // Child 0: Paragraph
            verify_item(
                &root_session.content[0],
                "Item[2].content[0]",
                "Paragraph",
                "1",
            );

            // Child 1: "1.1. First Sub-session" with 2 paragraphs
            verify_item(
                &root_session.content[1],
                "Item[2].content[1]",
                "Session",
                "2",
            );
            if let ContentItem::Session(sub1) = &root_session.content[1] {
                verify_item(
                    &sub1.content[0],
                    "Item[2].content[1].content[0]",
                    "Paragraph",
                    "1",
                );
                verify_item(
                    &sub1.content[1],
                    "Item[2].content[1].content[1]",
                    "Paragraph",
                    "1",
                );
            }

            // Child 2: "1.2. Second Sub-session" with 2 children (paragraph + nested session)
            verify_item(
                &root_session.content[2],
                "Item[2].content[2]",
                "Session",
                "2",
            );
            if let ContentItem::Session(sub2) = &root_session.content[2] {
                // First child: paragraph
                verify_item(
                    &sub2.content[0],
                    "Item[2].content[2].content[0]",
                    "Paragraph",
                    "1",
                );

                // Second child: "1.2.1. Deeply Nested Session" with 2 paragraphs
                verify_item(
                    &sub2.content[1],
                    "Item[2].content[2].content[1]",
                    "Session",
                    "2",
                );
                if let ContentItem::Session(deeply_nested) = &sub2.content[1] {
                    verify_item(
                        &deeply_nested.content[0],
                        "Item[2].content[2].content[1].content[0]",
                        "Paragraph",
                        "1",
                    );
                    verify_item(
                        &deeply_nested.content[1],
                        "Item[2].content[2].content[1].content[1]",
                        "Paragraph",
                        "1",
                    );
                }
            }

            // Child 3: Paragraph ("Back to the first...")
            verify_item(
                &root_session.content[3],
                "Item[2].content[3]",
                "Paragraph",
                "1",
            );
        }

        // Item 3: "2. Another Root Session" with 2 children (paragraph + session)
        verify_item(&doc.items[3], "Item[3]", "Session", "2");
        if let ContentItem::Session(root2) = &doc.items[3] {
            // Child 0: Paragraph
            verify_item(&root2.content[0], "Item[3].content[0]", "Paragraph", "1");

            // Child 1: "2.1. Its Sub-session" with 1 paragraph
            verify_item(&root2.content[1], "Item[3].content[1]", "Session", "1");
            if let ContentItem::Session(sub) = &root2.content[1] {
                verify_item(
                    &sub.content[0],
                    "Item[3].content[1].content[0]",
                    "Paragraph",
                    "1",
                );
            }
        }

        // Item 4: Final paragraph
        verify_item(&doc.items[4], "Item[4]", "Paragraph", "1");
    }
}
