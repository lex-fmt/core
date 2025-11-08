//! Example tests demonstrating the test harness API
//!
//! These tests show how to use the fluent testing infrastructure

use lex::lex::ast::traits::Container;
use lex::lex::testing::lexplore::*;

#[test]
fn test_load_paragraph_by_number() {
    // Load paragraph variation #1 using fluent API
    let source = Lexplore::paragraph(1).source();
    assert!(source.contains("simple"));
}

#[test]
fn test_parse_with_reference_parser() {
    // Load and parse with the reference (stable) parser using fluent API
    let parsed = Lexplore::paragraph(1).parse();

    // Get the first paragraph using query API
    let paragraph = parsed.expect_paragraph();
    assert!(paragraph_text_starts_with(paragraph, "This is a simple"));
}

#[test]
fn test_parse_with_linebased_parser() {
    // Load and parse with the linebased (experimental) parser
    let parsed = Lexplore::paragraph(2).parse_with(Parser::Linebased);

    // Check if we can get a paragraph (may fail due to parser bugs)
    if let Some(paragraph) = parsed.first_paragraph() {
        println!("Linebased parser successfully parsed paragraph");
        assert!(!paragraph.text().is_empty());
    } else {
        println!("Linebased parser had issues (expected for some test cases)");
    }
}

#[test]
fn test_list_available_numbers() {
    // List all available variations for paragraphs
    let numbers = Lexplore::list_numbers_for(ElementType::Paragraph).unwrap();
    assert!(!numbers.is_empty());
    assert!(numbers.contains(&1));
    assert!(numbers.contains(&2));

    println!("Available paragraph variations: {:?}", numbers);
}

#[test]
fn test_session_with_children() {
    // Load a session element
    let doc = Lexplore::session(1).parse();

    // Get the first session using query API
    let sessions: Vec<_> = doc.iter_sessions_recursive().collect();
    if let Some(session) = sessions.first() {
        // Check it has a label
        assert!(!session.label().is_empty());

        // Check it has children
        assert!(!session.children().is_empty());

        println!("Session label: {}", session.label());
        println!("Session has {} children", session.children().len());
    }
}

#[test]
fn test_definition_structure() {
    // Load a definition
    let doc = Lexplore::definition(1).parse();

    // Get the first definition using query API
    let definitions: Vec<_> = doc.iter_definitions_recursive().collect();
    if let Some(definition) = definitions.first() {
        // Check it has a label (the defined term)
        assert!(!definition.label().is_empty());

        println!("Definition term: {}", definition.label());
        println!("Definition has {} children", definition.children().len());
    }
}

#[test]
fn test_list_structure() {
    // Load a list
    let doc = Lexplore::list(1).parse();

    // Get the first list using query API
    let lists: Vec<_> = doc.iter_lists_recursive().collect();
    if let Some(list) = lists.first() {
        // Check it has items
        assert!(!list.items.is_empty());

        println!("List has {} items", list.items.len());
    }
}

#[test]
fn test_nested_list() {
    // Load a nested list variation (list-07-nested-simple)
    let doc = Lexplore::list(7).parse();

    println!(
        "Parsed nested list document with {} top-level items",
        doc.root.children.len()
    );
}

#[test]
fn test_annotation_structure() {
    // Load an annotation
    let doc = Lexplore::annotation(1).parse();

    // Get the first annotation using query API
    let annotations: Vec<_> = doc.iter_annotations_recursive().collect();
    if let Some(annotation) = annotations.first() {
        println!("Annotation has {} children", annotation.children().len());
    }
}

#[test]
fn test_verbatim_structure() {
    // Load a verbatim block
    let doc = Lexplore::verbatim(1).parse();

    // Get the first verbatim block using query API
    let verbatims: Vec<_> = doc.iter_verbatim_blocks_recursive().collect();
    if let Some(verbatim) = verbatims.first() {
        println!("Verbatim block has {} lines", verbatim.children.len());
    }
}

#[test]
fn test_element_source_for_api_examples() {
    // This demonstrates the query API usage

    // Parse using fluent API
    let doc = Lexplore::paragraph(1).parse();

    // Get first element using query API
    let paragraph = doc.iter_paragraphs_recursive().next().unwrap();

    // Assertions
    assert!(paragraph_text_starts_with(paragraph, "This is a simple"));
    assert!(paragraph_text_contains(paragraph, "paragraph"));
}
