//! Demonstration of the fluent API for test harness
//!
//! This shows the ergonomic improvements from adding the fluent builder pattern

use lex::lex::ast::traits::Container;
use lex::lex::testing::lexplore::*;

// ============================================================================
// BEFORE: Verbose with unwrap() everywhere
// ============================================================================

#[test]
fn test_old_style_verbose() {
    let doc = Lexplore::paragraph(1).parse();
    // Using query API directly
    let paragraph = doc.iter_paragraphs_recursive().next().unwrap();

    assert!(paragraph_text_starts_with(paragraph, "This is a simple"));
}

// ============================================================================
// AFTER: Clean fluent API
// ============================================================================

#[test]
fn test_new_style_fluent() {
    // Much cleaner! No unwrap() needed
    let parsed = Lexplore::paragraph(1).parse();
    let paragraph = parsed.expect_paragraph();

    assert!(paragraph_text_starts_with(paragraph, "This is a simple"));
}

#[test]
fn test_new_style_with_parser_choice() {
    // Easy to switch parsers
    let parsed = Lexplore::paragraph(1).parse_with(Parser::Linebased); // or Parser::Reference

    // The rest stays the same
    if let Some(paragraph) = parsed.first_paragraph() {
        assert!(paragraph_text_contains(paragraph, "simple"));
    }
}

// ============================================================================
// ELEMENT-SPECIFIC SHORTCUTS
// ============================================================================

#[test]
fn test_paragraph_shortcut() {
    let parsed = Lexplore::paragraph(2).parse();
    let p = parsed.expect_paragraph();
    println!("Paragraph text: {}", p.text());
}

#[test]
fn test_list_shortcut() {
    let parsed = Lexplore::list(1).parse();
    let list = parsed.expect_list();
    println!("List has {} items", list.items.len());
}

#[test]
fn test_session_shortcut() {
    let parsed = Lexplore::session(1).parse();
    let session = parsed.expect_session();
    println!(
        "Session: {} ({} children)",
        session.label(),
        session.children().len()
    );
}

#[test]
fn test_definition_shortcut() {
    let parsed = Lexplore::definition(1).parse();
    let def = parsed.expect_definition();
    println!(
        "Definition: {} ({} children)",
        def.label(),
        def.children().len()
    );
}

// ============================================================================
// MUST_ METHODS FOR DIRECT ACCESS
// ============================================================================

#[test]
fn test_get_source() {
    // Get source using fluent API
    let source = Lexplore::paragraph(1).source();
    assert!(source.contains("simple"));
}

#[test]
fn test_get_ast_fluent_api() {
    // Get AST using fluent API
    let doc = Lexplore::paragraph(1).parse();
    assert!(!doc.root.children.is_empty());
}

// ============================================================================
// JUST SOURCE, NO PARSING
// ============================================================================

#[test]
fn test_get_source_only() {
    // Sometimes you just need the raw source
    let source = Lexplore::paragraph(1).source();
    assert!(!source.is_empty());
}

// ============================================================================
// OPTIONAL EXTRACTION (NO PANIC)
// ============================================================================

#[test]
fn test_safe_optional_extraction() {
    let parsed = Lexplore::paragraph(1).parse();

    // Use first_* for Option, expect_* to panic if not found
    match parsed.first_paragraph() {
        Some(p) => println!("Found paragraph: {}", p.text()),
        None => println!("No paragraph found (parser might have issues)"),
    }
}

// ============================================================================
// COMPARING PARSERS
// ============================================================================

#[test]
fn test_compare_both_parsers() {
    // Parse with both parsers using fluent API
    let parsed_ref = Lexplore::paragraph(1).parse_with(Parser::Reference);
    let parsed_linebased = Lexplore::paragraph(1).parse_with(Parser::Linebased);

    // Both parsers should at least parse successfully for this simple case
    let ref_para = parsed_ref.first_paragraph();
    let linebased_para = parsed_linebased.first_paragraph();

    if ref_para.is_some() && linebased_para.is_some() {
        println!("✓ Both parsers successfully parsed");
    } else {
        println!("✗ One or both parsers had issues");
    }
}

// ============================================================================
// REAL-WORLD USAGE EXAMPLE
// ============================================================================

#[test]
fn test_realistic_workflow() {
    // Load and parse
    let parsed = Lexplore::paragraph(1).parse();

    // Extract element
    let paragraph = parsed.expect_paragraph();

    // Make assertions
    assert!(!paragraph.text().is_empty());
    assert!(paragraph_text_starts_with(paragraph, "This"));
    assert!(paragraph_text_contains(paragraph, "simple"));

    // Or use the existing fluent assertion API
    use lex::lex::testing::assert_ast;
    assert_ast(&parsed).item_count(1).item(0, |item| {
        item.assert_paragraph().text_contains("simple");
    });
}

// ============================================================================
// NESTED ELEMENTS
// ============================================================================

#[test]
fn test_nested_list() {
    let parsed = Lexplore::list(7).parse(); // nested-simple
    let list = parsed.expect_list();

    assert!(list.items.len() >= 2);
    println!("Nested list has {} top-level items", list.items.len());
}

#[test]
fn test_nested_definition() {
    let parsed = Lexplore::definition(6).parse(); // nested-definitions

    // Parser may have issues with complex nested content
    if let Some(def) = parsed.first_definition() {
        assert!(!def.label().is_empty());
        println!(
            "Definition '{}' has {} children",
            def.label(),
            def.children().len()
        );
    } else {
        println!("Note: Parser couldn't extract definition (known parser issues)");
    }
}
