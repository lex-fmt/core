use crate::txxt::ast::{AstNode, Container, ContentItem};
use crate::txxt::lexer::{lex, lex_with_locations, Token};
use crate::txxt::parser::api::{parse, parse_with_source};
use crate::txxt::parser::combinators::paragraph;
use crate::txxt::processor::txxt_sources::TxxtSources;
use chumsky::Parser;
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
                println!(
                    "  Error at location {:?}: {:?}",
                    error.span(),
                    error.reason()
                );
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
                println!(
                    "  Error at location {:?}: {:?}",
                    error.span(),
                    error.reason()
                );
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
                for list_item in &list.items {
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
    let doc = parse_with_source(tokens, &source).expect("Failed to parse foreign blocks sample");

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
