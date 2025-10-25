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

use super::ast::{ContentItem, Document, List, ListItem, Paragraph, Session};
use crate::txxt_nano::lexer::Token;

/// Type alias for token with span
type TokenSpan = (Token, Range<usize>);

/// Type alias for parser error
type ParserError = Simple<TokenSpan>;

/// Intermediate AST structures that hold spans instead of extracted text
/// These are converted to final AST structures after parsing completes

#[derive(Debug, Clone)]
#[allow(dead_code)] // Used internally in parser, may not be directly constructed elsewhere
pub(crate) struct ParagraphWithSpans {
    line_spans: Vec<Vec<Range<usize>>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct SessionWithSpans {
    title_spans: Vec<Range<usize>>,
    content: Vec<ContentItemWithSpans>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ListWithSpans {
    item_spans: Vec<Vec<Range<usize>>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) enum ContentItemWithSpans {
    Paragraph(ParagraphWithSpans),
    Session(SessionWithSpans),
    List(ListWithSpans),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct DocumentWithSpans {
    items: Vec<ContentItemWithSpans>,
}

/// Helper to extract text from source using a span
#[allow(dead_code)] // Reserved for future use
fn extract_text(source: &str, span: &Range<usize>) -> String {
    if span.start >= span.end || span.end > source.len() {
        // Empty or synthetic span (like for IndentLevel/DedentLevel)
        return String::new();
    }
    source[span.start..span.end].to_string()
}

/// Helper to extract and concatenate text from multiple spans
fn extract_line_text(source: &str, spans: &[Range<usize>]) -> String {
    if spans.is_empty() {
        return String::new();
    }

    // Find the overall span from first to last
    let start = spans.first().map(|s| s.start).unwrap_or(0);
    let end = spans.last().map(|s| s.end).unwrap_or(0);

    if start >= end || end > source.len() {
        return String::new();
    }

    source[start..end].trim().to_string()
}

/// Convert intermediate AST with spans to final AST with extracted text
fn convert_document(source: &str, doc_with_spans: DocumentWithSpans) -> Document {
    Document {
        items: doc_with_spans
            .items
            .into_iter()
            .map(|item| convert_content_item(source, item))
            .collect(),
    }
}

fn convert_content_item(source: &str, item: ContentItemWithSpans) -> ContentItem {
    match item {
        ContentItemWithSpans::Paragraph(p) => ContentItem::Paragraph(convert_paragraph(source, p)),
        ContentItemWithSpans::Session(s) => ContentItem::Session(convert_session(source, s)),
        ContentItemWithSpans::List(l) => ContentItem::List(convert_list(source, l)),
    }
}

fn convert_paragraph(source: &str, para: ParagraphWithSpans) -> Paragraph {
    Paragraph {
        lines: para
            .line_spans
            .iter()
            .map(|spans| extract_line_text(source, spans))
            .collect(),
    }
}

fn convert_session(source: &str, sess: SessionWithSpans) -> Session {
    Session {
        title: extract_line_text(source, &sess.title_spans),
        content: sess
            .content
            .into_iter()
            .map(|item| convert_content_item(source, item))
            .collect(),
    }
}

fn convert_list(source: &str, list: ListWithSpans) -> List {
    List {
        items: list
            .item_spans
            .iter()
            .map(|spans| ListItem::new(extract_line_text(source, spans)))
            .collect(),
    }
}

/// Parse a text line (sequence of text and whitespace tokens)
/// Returns the collected spans for this line
fn text_line() -> impl Parser<TokenSpan, Vec<Range<usize>>, Error = ParserError> + Clone {
    filter(|(t, _span): &TokenSpan| {
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
    .map(|tokens_with_spans: Vec<TokenSpan>| {
        // Collect all spans for this line
        tokens_with_spans.into_iter().map(|(_, s)| s).collect()
    })
}

/// Helper: match a specific token type, ignoring the span
fn token(t: Token) -> impl Parser<TokenSpan, (), Error = ParserError> + Clone {
    filter(move |(tok, _)| tok == &t).ignored()
}

/// Parse a list item line - a line that starts with a list marker
/// Grammar: <list-item-line> = <plain-marker> <text>+ | <ordered-marker> <text>+
/// Where: <plain-marker> = "-" " "
///        <ordered-marker> = (<number> | <letter> | <roman>) ("." | ")") " "
fn list_item_line() -> impl Parser<TokenSpan, Vec<Range<usize>>, Error = ParserError> + Clone {
    // Just check that the line starts with a valid list marker, then collect all tokens
    // We validate the marker and collect the full line content
    let rest_of_line = filter(|(t, _span): &TokenSpan| {
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
    .repeated();

    // Pattern 1: Dash + whitespace + rest
    let dash_pattern = filter(|(t, _): &TokenSpan| matches!(t, Token::Dash))
        .then(filter(|(t, _): &TokenSpan| matches!(t, Token::Whitespace)))
        .chain(rest_of_line);

    // Pattern 2: Number/Text + Period/CloseParen + whitespace + rest
    let ordered_pattern = filter(|(t, _): &TokenSpan| matches!(t, Token::Number | Token::Text))
        .then(filter(|(t, _): &TokenSpan| {
            matches!(t, Token::Period | Token::CloseParen)
        }))
        .then(filter(|(t, _): &TokenSpan| matches!(t, Token::Whitespace)))
        .chain(rest_of_line);

    // Pattern 3: OpenParen + Number + CloseParen + whitespace + rest
    let paren_pattern = filter(|(t, _): &TokenSpan| matches!(t, Token::OpenParen))
        .then(filter(|(t, _): &TokenSpan| matches!(t, Token::Number)))
        .then(filter(|(t, _): &TokenSpan| matches!(t, Token::CloseParen)))
        .then(filter(|(t, _): &TokenSpan| matches!(t, Token::Whitespace)))
        .chain(rest_of_line);

    // Try each pattern and collect all spans
    dash_pattern
        .or(ordered_pattern)
        .or(paren_pattern)
        .map(|tokens_with_spans: Vec<TokenSpan>| {
            tokens_with_spans.into_iter().map(|(_, s)| s).collect()
        })
}

/// Parse a list - two or more consecutive list-item-lines
/// Grammar: <list> = <blank-line> <list-item-line>{2,*}
///
/// IMPORTANT: Lists require a preceding blank line for disambiguation.
/// The blank line has already been consumed by the previous element's ending newline.
/// Blank lines between list items are NOT allowed (would terminate the list).
fn list() -> impl Parser<TokenSpan, ListWithSpans, Error = ParserError> + Clone {
    // Expect to start right at first list-item-line (blank line already consumed)
    list_item_line()
        .then_ignore(token(Token::Newline))
        .repeated()
        .at_least(2) // Lists require at least 2 items
        .then_ignore(token(Token::Newline).or_not()) // Optional blank line at end
        .map(|item_spans| ListWithSpans { item_spans })
}

/// Parse a paragraph - one or more lines of text separated by newlines, ending with a blank line
/// A paragraph is a catch-all that matches when nothing else does.
///
/// Simplified rule: Paragraphs can contain ANYTHING (including single list-item-lines).
/// Lists require a blank line before them, so disambiguation is handled by parse order:
/// 1. Try list first (needs blank line + 2+ items)
/// 2. Try session (needs title + blank + indent)
/// 3. Try paragraph (catches everything else)
fn paragraph() -> impl Parser<TokenSpan, ParagraphWithSpans, Error = ParserError> {
    // Match lines that are NOT session titles (not followed by blank line + IndentLevel)
    let non_session_line = text_line().then_ignore(token(Token::Newline)).then_ignore(
        token(Token::Newline)
            .then(token(Token::IndentLevel))
            .not()
            .rewind(),
    );

    non_session_line
        .repeated()
        .at_least(1)
        .then_ignore(token(Token::Newline).or_not()) // Optional blank line at end
        .map(|line_spans| ParagraphWithSpans { line_spans })
}

/// Parse a session title - a line of text followed by a newline and blank line
fn session_title() -> impl Parser<TokenSpan, Vec<Range<usize>>, Error = ParserError> + Clone {
    text_line()
        .then_ignore(token(Token::Newline))
        .then_ignore(token(Token::Newline))
}

/// Parse a session - a title followed by indented content
/// IMPORTANT: Once we match a session title, we MUST see IndentLevel or fail
/// This prevents backtracking to paragraph parser when session content is malformed
fn session() -> impl Parser<TokenSpan, SessionWithSpans, Error = ParserError> + Clone {
    recursive(|session_parser| {
        // Parse order (from docs/tips-tricks.txxt):
        // 1. List first (requires 2+ list-item-lines)
        // 2. Session second (requires title + blank + indent)
        // 3. Paragraph last (catch-all, including single list-item-lines)
        let content_item = list()
            .map(ContentItemWithSpans::List)
            .or(session_parser.map(ContentItemWithSpans::Session))
            .or(paragraph().map(ContentItemWithSpans::Paragraph));

        session_title()
            .then(
                // Once we have a session title, we're committed - must see IndentLevel
                token(Token::IndentLevel)
                    .ignore_then(content_item.repeated().at_least(1)) // Sessions must have content
                    .then_ignore(token(Token::DedentLevel)),
            )
            .map(|(title_spans, content)| SessionWithSpans {
                title_spans,
                content,
            })
    })
}

/// Parse a document - a sequence of paragraphs, lists, and sessions
/// Returns intermediate AST with spans
///
/// Parse order (from docs/tips-tricks.txxt):
/// 1. List first (requires 2+ list-item-lines)
/// 2. Session second (requires title + blank + indent)
/// 3. Paragraph last (catch-all, including single list-item-lines)
#[allow(private_interfaces)] // DocumentWithSpans is internal implementation detail
pub fn document() -> impl Parser<TokenSpan, DocumentWithSpans, Error = ParserError> {
    let content_item = list()
        .map(ContentItemWithSpans::List)
        .or(session().map(ContentItemWithSpans::Session))
        .or(paragraph().map(ContentItemWithSpans::Paragraph));

    content_item
        .repeated()
        .then_ignore(token(Token::DedentLevel)) // Consume final document-closing dedent
        .then_ignore(end())
        .map(|items| DocumentWithSpans { items })
}

/// Parse with source text - extracts actual content from spans
pub fn parse_with_source(
    tokens_with_spans: Vec<TokenSpan>,
    source: &str,
) -> Result<Document, Vec<ParserError>> {
    let doc_with_spans = document().parse(tokens_with_spans)?;
    Ok(convert_document(source, doc_with_spans))
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
    use crate::txxt_nano::lexer::{lex, lex_with_spans};
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
                                ContentItem::List(l) =>
                                    format!("  {}: List ({} items)", i, l.items.len()),
                            })
                            .collect::<Vec<_>>()
                            .join("\n")
                    );
                }
            }
            (ContentItem::List(l), "List") => {
                // expected_details should be item count
                if let Ok(expected_items) = expected_details.parse::<usize>() {
                    assert_eq!(
                        l.items.len(),
                        expected_items,
                        "{}: Expected List with {} items, got {} items",
                        path,
                        expected_items,
                        l.items.len()
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
            (ContentItem::List(l), expected) => {
                panic!(
                    "{}: Expected {}, got List with {} items",
                    path,
                    expected,
                    l.items.len()
                );
            }
        }
    }

    #[test]
    fn test_simple_paragraph() {
        let input = "Hello world\n\n";
        let tokens_with_spans = lex_with_spans(input);

        let result = paragraph().parse(tokens_with_spans);
        assert!(result.is_ok(), "Failed to parse paragraph: {:?}", result);

        let para_with_spans = result.unwrap();
        assert_eq!(para_with_spans.line_spans.len(), 1);

        // Verify actual content is preserved
        let para = convert_paragraph(input, para_with_spans);
        assert_eq!(para.lines.len(), 1);
        assert_eq!(para.lines[0], "Hello world");
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
                println!("Document has {} items:", doc.items.len());
                for (i, item) in doc.items.iter().enumerate() {
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
            Token::Text,
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
                println!("Document has {} items:", doc.items.len());
                for (i, item) in doc.items.iter().enumerate() {
                    match item {
                        ContentItem::Paragraph(p) => {
                            println!("  {}: Paragraph with {} lines", i, p.lines.len());
                        }
                        ContentItem::Session(s) => {
                            println!(
                                "  {}: Session '{}' with {} children",
                                i,
                                s.title,
                                s.content.len()
                            );
                        }
                        ContentItem::List(l) => {
                            println!("  {}: List with {} items", i, l.items.len());
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
        // THIS is the greedy paragraph bug from the code review:
        // Text Newline Newline IndentLevel [something that's not a valid content item]
        //
        // When session parser fails to parse content after IndentLevel,
        // it backtracks and paragraph parser gets a chance
        // Paragraph parser matches "Text Newline" leaving "Newline IndentLevel ..."
        // This causes a confusing error
        //
        // With the fix: Paragraph parser uses `.not()` to reject patterns followed by IndentLevel
        // So it won't consume the session title, and the error will be about the malformed session

        let tokens = vec![
            Token::Text, // "title"
            Token::Newline,
            Token::Newline,
            Token::IndentLevel,
            Token::Colon, // This is not valid content (can't start a paragraph or session)
            Token::DedentLevel,
            Token::DedentLevel,
        ];

        println!("\n=== Test: Greedy paragraph bug - session title + IndentLevel + unparseable content ===");
        println!("Tokens: {:?}", tokens);

        let result = parse(tokens.clone());

        match &result {
            Ok(doc) => {
                println!("\n✓ Parsed successfully (shouldn't happen!):");
                for (i, item) in doc.items.iter().enumerate() {
                    match item {
                        ContentItem::Paragraph(p) => {
                            println!("  {}: Paragraph with {} lines", i, p.lines.len());
                        }
                        ContentItem::Session(s) => {
                            println!(
                                "  {}: Session '{}' with {} children",
                                i,
                                s.title,
                                s.content.len()
                            );
                        }
                        ContentItem::List(l) => {
                            println!("  {}: List with {} items", i, l.items.len());
                        }
                    }
                }
                panic!("Should have failed to parse!");
            }
            Err(errors) => {
                println!("\n✗ Parse failed with {} error(s):", errors.len());
                for (i, error) in errors.iter().enumerate() {
                    println!("  Error {}: at span {:?}", i, error.span());
                    println!("    Reason: {:?}", error.reason());
                    println!("    Found: {:?}", error.found());
                }

                // With the bug: error says "unexpected IndentLevel at position 3"
                //   because paragraph consumed "Text Newline", left "Newline IndentLevel Colon..."
                //
                // With the fix: error is NOT at position 3 (IndentLevel)
                //   It could be at position 4 (Colon - can't start content item)
                //   Or at position 5+ (trailing tokens after session attempts to match)

                assert_eq!(errors.len(), 1, "Should have exactly one error");
                let error = &errors[0];

                // The critical check: error should NOT be at position 3 (IndentLevel)
                // If it is, that means paragraph parser consumed the title
                assert_ne!(
                    error.span().start, 3,
                    "BUG STILL PRESENT: Paragraph parser consumed session title, error is at IndentLevel (position 3)"
                );

                println!(
                    "\n✓ Fix verified: Error is at position {}, not at IndentLevel (position 3)",
                    error.span().start
                );
                println!("  This means the paragraph parser correctly rejected the session title pattern");
            }
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
                println!("Document has {} items:", doc.items.len());
                assert_eq!(doc.items.len(), 2, "Should have 2 paragraphs");
            }
            Err(e) => {
                panic!("Should have parsed successfully: {:?}", e);
            }
        }
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
                    ContentItem::List(l) => format!("  {}: List with {} items", i, l.items.len()),
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
                ContentItem::List(l) => {
                    panic!(
                        "Item {} should be a Paragraph with {} lines, but found List with {} items",
                        i,
                        expected_lines,
                        l.items.len()
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
                    ContentItem::List(l) => format!("  {}: List with {} items", i, l.items.len()),
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
            ContentItem::List(l) => {
                panic!(
                    "Item 0: Expected Paragraph, got List with {} items",
                    l.items.len()
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
            ContentItem::List(l) => {
                panic!(
                    "Item 1: Expected Paragraph, got List with {} items",
                    l.items.len()
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
                            ContentItem::List(l) => format!("  {}: List with {} items", i, l.items.len()),
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
                    ContentItem::List(l) => {
                        panic!(
                            "Item 2, child 0: Expected Paragraph, got List with {} items",
                            l.items.len()
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
                    ContentItem::List(l) => {
                        panic!(
                            "Item 2, child 1: Expected Paragraph, got List with {} items",
                            l.items.len()
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
            ContentItem::List(l) => {
                panic!(
                    "Item 2: Expected Session, got List with {} items",
                    l.items.len()
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
            ContentItem::List(l) => {
                panic!(
                    "Item 3: Expected Paragraph, got List with {} items",
                    l.items.len()
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
                            ContentItem::List(l) => format!("  {}: List with {} items", i, l.items.len()),
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
                    ContentItem::List(l) => {
                        panic!(
                            "Item 4, child 0: Expected Paragraph, got List with {} items",
                            l.items.len()
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
            ContentItem::List(l) => {
                panic!(
                    "Item 4: Expected Session, got List with {} items",
                    l.items.len()
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
            ContentItem::List(l) => {
                panic!(
                    "Item 5: Expected Paragraph, got List with {} items",
                    l.items.len()
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
                    ContentItem::List(l) => format!("  {}: List ({} items)", i, l.items.len()),
                })
                .collect::<Vec<_>>()
                .join("\n")
        );

        use crate::txxt_nano::testing::assert_ast;

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
                    ContentItem::List(l) => format!("  {}: List ({} items)", i, l.items.len()),
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
            doc.items.len(),
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
            doc2.items.len(),
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
                        .assert_paragraph()
                        .text_contains("starts with a paragraph");
                })
                .child(1, |child| {
                    child
                        .assert_paragraph()
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
                item.assert_paragraph()
                    .text_contains("Trifecta Nesting Test");
            })
            .item(1, |item| {
                item.assert_paragraph()
                    .text_contains("various levels of nesting");
            });

        // Item 2: Root session with nested sessions and mixed content
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
}
