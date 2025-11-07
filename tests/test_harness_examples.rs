//! Example tests demonstrating the test harness API
//!
//! These tests show how to use the new per-element testing infrastructure

use lex::lex::ast::traits::Container;
use lex::lex::testing::lexplore::*;

#[test]
fn test_load_paragraph_by_number() {
    // Load paragraph variation #1
    let source = Lexplore::get_source_for(ElementType::Paragraph, 1).unwrap();
    assert!(source.contains("simple paragraph"));
}

#[test]
fn test_parse_with_reference_parser() {
    // Load and parse with the reference (stable) parser
    let source = Lexplore::get_source_for(ElementType::Paragraph, 1).unwrap();
    let doc = parse_with_parser(&source, Parser::Reference).unwrap();

    // Get the first paragraph
    let paragraph = get_first_paragraph(&doc).unwrap();
    assert!(paragraph_text_starts_with(paragraph, "This is a simple"));
}

#[test]
fn test_parse_with_linebased_parser() {
    // Load and parse with the linebased (experimental) parser
    let source = Lexplore::get_source_for(ElementType::Paragraph, 2).unwrap();

    // Note: linebased parser may have issues, this just demonstrates the API
    let result = parse_with_parser(&source, Parser::Linebased);

    // We don't assert success because parsers may have bugs
    // The infrastructure itself is being tested
    println!("Linebased parser result: {:?}", result.is_ok());
}

#[test]
fn test_compare_parsers() {
    // Parse the same source with both parsers
    let source = Lexplore::get_source_for(ElementType::Paragraph, 1).unwrap();

    let parsers = vec![Parser::Reference, Parser::Linebased];
    let results = parse_with_multiple_parsers(&source, &parsers);

    // We got results from both parsers (may or may not match due to parser bugs)
    println!(
        "Got results from {} parsers",
        results.as_ref().map(|r| r.len()).unwrap_or(0)
    );

    if let Ok(results) = results {
        // Try comparing them (may fail if parsers produce different ASTs)
        let comparison = compare_parser_results(&results);
        println!("Parser comparison result: {:?}", comparison.is_ok());
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
    let source = Lexplore::get_source_for(ElementType::Session, 1).unwrap();
    let doc = parse_with_parser(&source, Parser::Reference).unwrap();

    // Get the first session
    if let Some(session) = get_first_session(&doc) {
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
    let source = Lexplore::get_source_for(ElementType::Definition, 1).unwrap();
    let doc = parse_with_parser(&source, Parser::Reference).unwrap();

    // Get the first definition
    if let Some(definition) = get_first_definition(&doc) {
        // Check it has a label (the defined term)
        assert!(!definition.label().is_empty());

        println!("Definition term: {}", definition.label());
        println!("Definition has {} children", definition.children().len());
    }
}

#[test]
fn test_list_structure() {
    // Load a list
    let source = Lexplore::get_source_for(ElementType::List, 1).unwrap();
    let doc = parse_with_parser(&source, Parser::Reference).unwrap();

    // Get the first list
    if let Some(list) = get_first_list(&doc) {
        // Check it has items
        assert!(!list.items.is_empty());

        println!("List has {} items", list.items.len());
    }
}

#[test]
fn test_nested_list() {
    // Load a nested list variation
    let source = Lexplore::get_source_for(ElementType::List, 7).unwrap(); // list-07-nested-simple
    let doc = parse_with_parser(&source, Parser::Reference).unwrap();

    println!(
        "Parsed nested list document with {} top-level items",
        doc.root.children.len()
    );
}

#[test]
fn test_annotation_structure() {
    // Load an annotation
    let source = Lexplore::get_source_for(ElementType::Annotation, 1).unwrap();
    let doc = parse_with_parser(&source, Parser::Reference).unwrap();

    // Get the first annotation
    if let Some(annotation) = get_first_annotation(&doc) {
        println!("Annotation has {} children", annotation.children().len());
    }
}

#[test]
fn test_verbatim_structure() {
    // Load a verbatim block
    let source = Lexplore::get_source_for(ElementType::Verbatim, 1).unwrap();
    let doc = parse_with_parser(&source, Parser::Reference).unwrap();

    // Get the first verbatim block
    if let Some(verbatim) = get_first_verbatim(&doc) {
        println!("Verbatim block has {} lines", verbatim.children.len());
    }
}

#[test]
fn test_element_source_for_api_examples() {
    // This demonstrates the desired API from the task:
    // ast = LexSource.getSourceFor(Paragraph, 1)

    // Get source
    let source = Lexplore::get_source_for(ElementType::Paragraph, 1).unwrap();

    // Parse with chosen parser
    let doc = parse_with_parser(&source, Parser::Reference).unwrap();

    // Get first element
    let paragraph = get_first_paragraph(&doc).unwrap();

    // Assertions
    assert!(paragraph_text_starts_with(paragraph, "This is a simple"));
    assert!(paragraph_text_contains(paragraph, "paragraph"));

    // This API matches what was requested:
    // paragraph.text_starts_with("Hello there")
}
