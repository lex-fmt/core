use crate::txxt_nano::ast::{Container, ContentItem};
use crate::txxt_nano::lexer::{lex, lex_with_spans, Token};
use crate::txxt_nano::parser::api::parse;
use crate::txxt_nano::parser::combinators::paragraph;
use chumsky::Parser;

#[test]
fn test_simple_paragraph() {
    let input = "Hello world\n\n";
    let mut tokens_with_spans = lex_with_spans(input);

    // Skip DocStart and DocEnd tokens for direct paragraph test
    tokens_with_spans.retain(|(t, _)| !matches!(t, Token::DocStart | Token::DocEnd));

    let result = paragraph(input).parse(tokens_with_spans);
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
        }
        Err(errors) => {
            println!("\n✗ Parse failed with errors:");
            for error in errors {
                println!("  Error at span {:?}: {:?}", error.span(), error.reason());
                println!("  Found: {:?}", error.found());
            }
        }
    }
}

#[test]
fn test_session_title_followed_by_bare_indent_level() {
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
                    _ => {}
                }
            }
        }
        Err(errors) => {
            println!("\n✗ Parse failed:");
            for error in errors {
                println!("  Error at span {:?}: {:?}", error.span(), error.reason());
            }
        }
    }
}
