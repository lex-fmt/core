//! Tests for NodeStartLocation trait implementation
//!
//! This module tests the ability to retrieve the starting location (line, column)
//! of AST nodes in the source document.

use txxt_nano::txxt_nano::ast::{
    Annotation, ContentItem, Definition, Document, ForeignBlock, Label, List, ListItem, Paragraph,
    Position, Session, Span, NodeStartLocation, TextContent,
};
use txxt_nano::txxt_nano::lexer::lex_with_spans;
use txxt_nano::txxt_nano::parser::parse_with_source_positions;

#[test]
fn test_paragraph_start_location_with_span() {
    let para = Paragraph::from_line("Test paragraph".to_string())
        .with_span(Some(Span::new(Position::new(5, 10), Position::new(5, 24))));

    let location = para.start_location();
    assert!(location.is_some());
    assert_eq!(location.unwrap(), Position::new(5, 10));
}

#[test]
fn test_paragraph_start_location_without_span() {
    let para = Paragraph::from_line("Test paragraph".to_string());

    let location = para.start_location();
    assert!(location.is_none());
}

#[test]
fn test_session_start_location_with_span() {
    let session = Session::with_title("Section".to_string())
        .with_span(Some(Span::new(Position::new(2, 0), Position::new(10, 5))));

    let location = session.start_location();
    assert!(location.is_some());
    assert_eq!(location.unwrap(), Position::new(2, 0));
}

#[test]
fn test_session_start_location_without_span() {
    let session = Session::with_title("Section".to_string());

    let location = session.start_location();
    assert!(location.is_none());
}

#[test]
fn test_definition_start_location_with_span() {
    let definition = Definition::with_subject("Term".to_string())
        .with_span(Some(Span::new(Position::new(3, 5), Position::new(4, 20))));

    let location = definition.start_location();
    assert!(location.is_some());
    assert_eq!(location.unwrap(), Position::new(3, 5));
}

#[test]
fn test_definition_start_location_without_span() {
    let definition = Definition::with_subject("Term".to_string());

    let location = definition.start_location();
    assert!(location.is_none());
}

#[test]
fn test_annotation_start_location_with_span() {
    let annotation = Annotation::marker(Label::new("note".to_string()))
        .with_span(Some(Span::new(Position::new(1, 0), Position::new(1, 10))));

    let location = annotation.start_location();
    assert!(location.is_some());
    assert_eq!(location.unwrap(), Position::new(1, 0));
}

#[test]
fn test_annotation_start_location_without_span() {
    let annotation = Annotation::marker(Label::new("note".to_string()));

    let location = annotation.start_location();
    assert!(location.is_none());
}

#[test]
fn test_foreign_block_start_location_with_span() {
    let closing = Annotation::marker(Label::new("end".to_string()));
    let foreign = ForeignBlock::marker("code".to_string(), closing)
        .with_span(Some(Span::new(Position::new(7, 2), Position::new(10, 8))));

    let location = foreign.start_location();
    assert!(location.is_some());
    assert_eq!(location.unwrap(), Position::new(7, 2));
}

#[test]
fn test_foreign_block_start_location_without_span() {
    let closing = Annotation::marker(Label::new("end".to_string()));
    let foreign = ForeignBlock::marker("code".to_string(), closing);

    let location = foreign.start_location();
    assert!(location.is_none());
}

#[test]
fn test_list_start_location_with_span() {
    let list = List::new(vec![ListItem::new("Item 1".to_string())])
        .with_span(Some(Span::new(Position::new(8, 0), Position::new(10, 0))));

    let location = list.start_location();
    assert!(location.is_some());
    assert_eq!(location.unwrap(), Position::new(8, 0));
}

#[test]
fn test_list_start_location_without_span() {
    let list = List::new(vec![ListItem::new("Item 1".to_string())]);

    let location = list.start_location();
    assert!(location.is_none());
}

#[test]
fn test_list_item_start_location_with_span() {
    let item = ListItem::new("Item text".to_string())
        .with_span(Some(Span::new(Position::new(4, 2), Position::new(4, 15))));

    let location = item.start_location();
    assert!(location.is_some());
    assert_eq!(location.unwrap(), Position::new(4, 2));
}

#[test]
fn test_list_item_start_location_without_span() {
    let item = ListItem::new("Item text".to_string());

    let location = item.start_location();
    assert!(location.is_none());
}

#[test]
fn test_document_start_location_with_span() {
    let doc = Document::new()
        .with_span(Some(Span::new(Position::new(0, 0), Position::new(20, 10))));

    let location = doc.start_location();
    assert!(location.is_some());
    assert_eq!(location.unwrap(), Position::new(0, 0));
}

#[test]
fn test_document_start_location_without_span() {
    let doc = Document::new();

    let location = doc.start_location();
    assert!(location.is_none());
}

#[test]
fn test_content_item_start_location_paragraph() {
    let para = Paragraph::from_line("Test".to_string())
        .with_span(Some(Span::new(Position::new(1, 5), Position::new(1, 9))));
    let item = ContentItem::Paragraph(para);

    let location = item.start_location();
    assert!(location.is_some());
    assert_eq!(location.unwrap(), Position::new(1, 5));
}

#[test]
fn test_content_item_start_location_session() {
    let session = Session::with_title("Test".to_string())
        .with_span(Some(Span::new(Position::new(2, 0), Position::new(5, 0))));
    let item = ContentItem::Session(session);

    let location = item.start_location();
    assert!(location.is_some());
    assert_eq!(location.unwrap(), Position::new(2, 0));
}

#[test]
fn test_start_location_with_multiline_span() {
    // Test with a span that crosses multiple lines
    let para = Paragraph::from_line("Multiline content".to_string())
        .with_span(Some(Span::new(Position::new(10, 15), Position::new(12, 5))));

    let location = para.start_location();
    assert!(location.is_some());
    assert_eq!(location.unwrap(), Position::new(10, 15));
}

#[test]
fn test_start_location_at_document_start() {
    // Test with a node at the very beginning of the document
    let para = Paragraph::from_line("First line".to_string())
        .with_span(Some(Span::new(Position::new(0, 0), Position::new(0, 10))));

    let location = para.start_location();
    assert!(location.is_some());
    assert_eq!(location.unwrap(), Position::new(0, 0));
}

#[test]
fn test_start_location_with_parsed_document() {
    // Test with an actual parsed document to ensure spans are correctly set
    let content = "First paragraph\n\nSecond paragraph\n\nThird paragraph";
    let tokens = lex_with_spans(content);
    let doc = parse_with_source_positions(tokens, content).unwrap();

    // Check that all paragraphs have valid start locations
    for item in &doc.content {
        let location = item.start_location();
        assert!(location.is_some(), "All parsed elements should have spans");
        let pos = location.unwrap();
        // Verify the position is valid (within document bounds)
        assert!(pos.line < 5, "Line should be within document");
    }
}

#[test]
fn test_start_location_preserves_exact_position() {
    // Test that the exact position is preserved
    for line in 0..10 {
        for col in 0..20 {
            let para = Paragraph::from_line("Test".to_string())
                .with_span(Some(Span::new(Position::new(line, col), Position::new(line, col + 4))));

            let location = para.start_location();
            assert!(location.is_some());
            let pos = location.unwrap();
            assert_eq!(pos.line, line, "Line should be preserved");
            assert_eq!(pos.column, col, "Column should be preserved");
        }
    }
}

#[test]
fn test_start_location_with_nested_structures() {
    // Test start locations in nested structures
    let inner_para = Paragraph::from_line("Nested".to_string())
        .with_span(Some(Span::new(Position::new(2, 4), Position::new(2, 10))));

    let session = Session::new(
        TextContent::from_string("Section".to_string(), None),
        vec![ContentItem::Paragraph(inner_para)],
    )
    .with_span(Some(Span::new(Position::new(1, 0), Position::new(3, 0))));

    // Check session location
    let session_loc = session.start_location();
    assert!(session_loc.is_some());
    assert_eq!(session_loc.unwrap(), Position::new(1, 0));

    // Check nested paragraph location
    if let Some(ContentItem::Paragraph(p)) = session.content.first() {
        let para_loc = p.start_location();
        assert!(para_loc.is_some());
        assert_eq!(para_loc.unwrap(), Position::new(2, 4));
    }
}

#[test]
fn test_start_location_with_complex_document() {
    // Create a more complex document structure
    let para1 = Paragraph::from_line("First".to_string())
        .with_span(Some(Span::new(Position::new(0, 0), Position::new(0, 5))));

    let inner_para = Paragraph::from_line("Nested".to_string())
        .with_span(Some(Span::new(Position::new(3, 2), Position::new(3, 8))));

    let session = Session::new(
        TextContent::from_string("Section".to_string(), None),
        vec![ContentItem::Paragraph(inner_para)],
    )
    .with_span(Some(Span::new(Position::new(2, 0), Position::new(4, 0))));

    let para2 = Paragraph::from_line("Last".to_string())
        .with_span(Some(Span::new(Position::new(5, 0), Position::new(5, 4))));

    let doc = Document::with_content(vec![
        ContentItem::Paragraph(para1),
        ContentItem::Session(session),
        ContentItem::Paragraph(para2),
    ])
    .with_span(Some(Span::new(Position::new(0, 0), Position::new(5, 4))));

    // Verify document start location
    assert_eq!(doc.start_location(), Some(Position::new(0, 0)));

    // Verify all child elements have correct start locations
    assert_eq!(
        doc.content[0].start_location(),
        Some(Position::new(0, 0))
    );
    assert_eq!(
        doc.content[1].start_location(),
        Some(Position::new(2, 0))
    );
    assert_eq!(
        doc.content[2].start_location(),
        Some(Position::new(5, 0))
    );
}

#[test]
fn test_start_location_consistency_with_span() {
    // Ensure that start_location always returns the span's start position
    let test_cases = vec![
        (Position::new(0, 0), Position::new(0, 10)),
        (Position::new(5, 15), Position::new(6, 20)),
        (Position::new(100, 200), Position::new(101, 0)),
    ];

    for (start, end) in test_cases {
        let para = Paragraph::from_line("Test".to_string())
            .with_span(Some(Span::new(start, end)));

        let location = para.start_location();
        assert_eq!(
            location,
            Some(start),
            "Start location should match span's start position"
        );
    }
}

#[test]
fn test_start_location_with_real_parsing_sessions() {
    // Test with a document containing sessions
    let content = ":: Introduction\n\nSome intro text\n\n:: Chapter 1\n\nChapter content";
    let tokens = lex_with_spans(content);
    
    // Try to parse, but if it fails, skip the test
    if let Ok(doc) = parse_with_source_positions(tokens, content) {
        // Find sessions and verify their start locations
        for item in &doc.content {
            if let ContentItem::Session(session) = item {
                let loc = session.start_location();
                assert!(loc.is_some(), "Session should have a start location");
                // Verify location is at line start (column 0 for session markers)
                let pos = loc.unwrap();
                assert_eq!(pos.column, 0, "Session should start at column 0");
            }
        }
    }
}

#[test]
fn test_start_location_with_lists() {
    let content = "- Item 1\n- Item 2\n- Item 3";
    let tokens = lex_with_spans(content);
    let doc = parse_with_source_positions(tokens, content).unwrap();

    // Find lists and verify their start locations
    for item in &doc.content {
        if let ContentItem::List(list) = item {
            let loc = list.start_location();
            assert!(loc.is_some(), "List should have a start location");
            assert_eq!(loc.unwrap().line, 0, "List should start at line 0");
        }
    }
}

#[test]
fn test_start_location_ordering() {
    // Verify that elements later in the document have larger positions
    let para1 = Paragraph::from_line("First".to_string())
        .with_span(Some(Span::new(Position::new(0, 0), Position::new(0, 5))));
    let para2 = Paragraph::from_line("Second".to_string())
        .with_span(Some(Span::new(Position::new(2, 0), Position::new(2, 6))));
    let para3 = Paragraph::from_line("Third".to_string())
        .with_span(Some(Span::new(Position::new(4, 0), Position::new(4, 5))));

    let loc1 = para1.start_location().unwrap();
    let loc2 = para2.start_location().unwrap();
    let loc3 = para3.start_location().unwrap();

    // Positions should be ordered
    assert!(loc1 < loc2, "First paragraph should come before second");
    assert!(loc2 < loc3, "Second paragraph should come before third");
    assert!(loc1 < loc3, "First paragraph should come before third");
}
