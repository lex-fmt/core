//! AST Node Creation from Extracted Data
//!
//! This module creates AST nodes from primitive data structures returned by
//! the data_extraction layer. It handles the conversion from byte ranges to
//! AST Range (line/column positions) and constructs the final AST structures.
//!
//! # Architecture
//!
//! ```text
//! Data Structs (primitives) → AST Creation → AST Nodes
//! { text: String,               ↓
//!   byte_range: Range<usize> }  - Convert byte ranges → ast::Range
//!                                - Create TextContent, TextLine, etc.
//!                                - Build complete AST nodes
//!                                ↓
//!                                ContentItem (with ast::Range)
//! ```
//!
//! # Responsibilities
//!
//! - Convert `Range<usize>` (byte offsets) → `ast::Range` (line/column)
//! - Create AST structures (TextContent, TextLine, Paragraph, etc.)
//! - Pure AST construction - no token processing
//! - Aggregate locations from children where needed
//!
//! # Key Design Principle
//!
//! This layer receives **primitives** and produces **AST types**. The byte→line/column
//! conversion happens here using `byte_range_to_ast_range()`.

use super::extraction::{
    AnnotationData, DefinitionData, ForeignBlockData, ListItemData, ParagraphData, SessionData,
};
use super::location::{
    aggregate_locations, byte_range_to_ast_range, compute_location_from_locations,
};
use crate::lex::ast::{
    Annotation, Definition, ForeignBlock, Label, List, ListItem, Paragraph, Range, Session,
    TextContent, TextLine,
};
use crate::lex::parsing::ContentItem;

// ============================================================================
// VALIDATION FUNCTIONS
// ============================================================================

/// Validates that a container does not contain any Session nodes.
///
/// Used for GeneralContainer types (Definition, Annotation, ListItem) where
/// Session nesting is prohibited by the Lex specification.
///
/// # Panics
///
/// Panics if any Session is found in the content with a descriptive error message.
fn validate_no_sessions(content: &[ContentItem], container_name: &str) {
    for item in content {
        if item.is_session() {
            panic!(
                "Invalid nesting: {} cannot contain Session elements. \
                 Sessions can only be nested within Document or other Sessions.",
                container_name
            );
        }
    }
}

/// Validates that a container only contains ListItem nodes.
///
/// Used for ListContainer types where only ListItem variants are allowed.
///
/// # Panics
///
/// Panics if any non-ListItem is found in the content.
fn validate_only_list_items(content: &[ContentItem]) {
    for item in content {
        if !item.is_list_item() {
            panic!(
                "Invalid List content: Lists can only contain ListItem elements, found {}",
                item.node_type()
            );
        }
    }
}

/// Validates that a container only contains ForeignLine nodes.
///
/// Used for ForeignContainer types where only ForeignLine nodes are allowed.
///
/// # Panics
///
/// Panics if any non-ForeignLine is found in the content.
fn validate_only_foreign_lines(content: &[ContentItem]) {
    for item in content {
        if !item.is_foreign_line() {
            panic!(
                "Invalid ForeignBlock content: ForeignBlocks can only contain ForeignLine elements, found {}",
                item.node_type()
            );
        }
    }
}

// ============================================================================
// PARAGRAPH CREATION
// ============================================================================

/// Create a Paragraph AST node from extracted paragraph data.
///
/// Converts byte ranges to AST Ranges and builds the Paragraph structure
/// with TextLines.
///
/// # Arguments
///
/// * `data` - Extracted paragraph data with text and byte ranges
/// * `source` - Original source string (needed for byte→line/column conversion)
///
/// # Returns
///
/// A Paragraph ContentItem with proper ast::Range locations
pub(super) fn create_paragraph(data: ParagraphData, source: &str) -> ContentItem {
    // Convert byte ranges to AST ranges and build TextLines
    let lines: Vec<ContentItem> = data
        .text_lines
        .into_iter()
        .map(|(text, byte_range)| {
            let location = byte_range_to_ast_range(byte_range, source);
            let text_content = TextContent::from_string(text, Some(location.clone()));
            let text_line = TextLine::new(text_content).at(location);
            ContentItem::TextLine(text_line)
        })
        .collect();

    // Convert overall byte range to AST range
    let overall_location = byte_range_to_ast_range(data.overall_byte_range, source);

    #[allow(deprecated)]
    {
        ContentItem::Paragraph(Paragraph {
            lines: crate::lex::ast::elements::container::Container::new(lines),
            location: overall_location,
        })
    }
}

// ============================================================================
// SESSION CREATION
// ============================================================================

/// Create a Session AST node from extracted session data.
///
/// Converts byte range to AST Range, creates TextContent for title,
/// and aggregates location from title and children.
///
/// # Arguments
///
/// * `data` - Extracted session data with title text and byte range
/// * `content` - Child content items
/// * `source` - Original source string
///
/// # Returns
///
/// A Session ContentItem
pub(super) fn create_session(
    data: SessionData,
    content: Vec<ContentItem>,
    source: &str,
) -> ContentItem {
    let title_location = byte_range_to_ast_range(data.title_byte_range, source);
    let title = TextContent::from_string(data.title_text, Some(title_location.clone()));
    let location = aggregate_locations(title_location, &content);

    let session = Session::new(title, content).at(location);
    ContentItem::Session(session)
}

// ============================================================================
// DEFINITION CREATION
// ============================================================================

/// Create a Definition AST node from extracted definition data.
///
/// Converts byte range to AST Range, creates TextContent for subject,
/// and aggregates location from subject and children.
///
/// # Arguments
///
/// * `data` - Extracted definition data with subject text and byte range
/// * `content` - Child content items
/// * `source` - Original source string
///
/// # Returns
///
/// A Definition ContentItem
pub(super) fn create_definition(
    data: DefinitionData,
    content: Vec<ContentItem>,
    source: &str,
) -> ContentItem {
    // Validate that content doesn't contain Sessions
    validate_no_sessions(&content, "Definition");

    let subject_location = byte_range_to_ast_range(data.subject_byte_range, source);
    let subject = TextContent::from_string(data.subject_text, Some(subject_location.clone()));
    let location = aggregate_locations(subject_location, &content);

    let definition = Definition::new(subject, content).at(location);
    ContentItem::Definition(definition)
}

// ============================================================================
// LIST CREATION
// ============================================================================

/// Create a List AST node from list items.
///
/// Aggregates location from all list items.
///
/// # Arguments
///
/// * `items` - Vector of ListItem nodes
///
/// # Returns
///
/// A List ContentItem
pub(super) fn create_list(items: Vec<ListItem>) -> ContentItem {
    // Convert ListItems to ContentItems
    let content: Vec<ContentItem> = items.into_iter().map(ContentItem::ListItem).collect();

    // Validate that all items are ListItems (should always be true, but verify)
    validate_only_list_items(&content);

    // Get locations from all items for aggregation
    let item_locations: Vec<Range> = content
        .iter()
        .map(|item| match item {
            ContentItem::ListItem(li) => li.location.clone(),
            _ => unreachable!(),
        })
        .collect();

    let location = if item_locations.is_empty() {
        Range::default()
    } else {
        compute_location_from_locations(&item_locations)
    };

    ContentItem::List(List {
        items: crate::lex::ast::elements::container::ListContainer::new(content),
        location,
    })
}

// ============================================================================
// LIST ITEM CREATION
// ============================================================================

/// Create a ListItem AST node from extracted list item data.
///
/// Converts byte range to AST Range, creates TextContent for marker,
/// and aggregates location from marker and children.
///
/// # Arguments
///
/// * `data` - Extracted list item data with marker text and byte range
/// * `content` - Child content items
/// * `source` - Original source string
///
/// # Returns
///
/// A ListItem node (not wrapped in ContentItem)
pub(super) fn create_list_item(
    data: ListItemData,
    content: Vec<ContentItem>,
    source: &str,
) -> ListItem {
    // Validate that content doesn't contain Sessions
    validate_no_sessions(&content, "ListItem");

    let marker_location = byte_range_to_ast_range(data.marker_byte_range, source);
    let marker = TextContent::from_string(data.marker_text, Some(marker_location.clone()));
    let location = aggregate_locations(marker_location, &content);

    ListItem::with_text_content(marker, content).at(location)
}

// ============================================================================
// ANNOTATION CREATION
// ============================================================================

/// Create an Annotation AST node from extracted annotation data.
///
/// Converts byte ranges to AST Ranges, creates Label and Parameters from extracted data,
/// and aggregates location from label and children.
///
/// # Arguments
///
/// * `data` - Extracted annotation data with label text, parameters, and byte ranges
/// * `content` - Child content items
/// * `source` - Original source string
///
/// # Returns
///
/// An Annotation ContentItem
pub(super) fn create_annotation(
    data: AnnotationData,
    content: Vec<ContentItem>,
    source: &str,
) -> ContentItem {
    use crate::lex::ast::Parameter;

    // Validate that content doesn't contain Sessions
    validate_no_sessions(&content, "Annotation");

    let label_location = byte_range_to_ast_range(data.label_byte_range, source);
    let label = Label::new(data.label_text).at(label_location.clone());

    // Convert ParameterData to Parameter AST nodes
    let parameters: Vec<Parameter> = data
        .parameters
        .into_iter()
        .map(|param_data| {
            let location = byte_range_to_ast_range(param_data.overall_byte_range, source);
            Parameter {
                key: param_data.key_text,
                value: param_data.value_text.unwrap_or_default(),
                location,
            }
        })
        .collect();

    // Aggregate location from label and content
    let location = aggregate_locations(label_location, &content);

    let annotation = Annotation {
        label,
        parameters,
        children: crate::lex::ast::elements::container::GeneralContainer::new(content),
        location,
    };

    ContentItem::Annotation(annotation)
}

// ============================================================================
// FOREIGN BLOCK CREATION
// ============================================================================

/// Create a ForeignBlock AST node from extracted foreign block data.
///
/// Converts byte ranges to AST Ranges, creates TextContent for subject and content,
/// and aggregates location from all components.
///
/// # Arguments
///
/// * `data` - Extracted foreign block data (with indentation wall already stripped)
/// * `closing_annotation` - The closing annotation node
/// * `source` - Original source string
///
/// # Returns
///
/// A ForeignBlock ContentItem
pub(super) fn create_foreign_block(
    data: ForeignBlockData,
    closing_annotation: Annotation,
    source: &str,
) -> ContentItem {
    use crate::lex::ast::elements::ForeignLine;

    let subject_location = byte_range_to_ast_range(data.subject_byte_range, source);
    let subject = TextContent::from_string(data.subject_text, Some(subject_location.clone()));

    // Create ForeignLine children from content lines
    let mut children: Vec<ContentItem> = Vec::new();
    let mut line_locations: Vec<Range> = Vec::new();

    for (line_text, line_byte_range) in data.content_lines {
        let line_location = byte_range_to_ast_range(line_byte_range, source);
        line_locations.push(line_location.clone());

        let line_content = TextContent::from_string(line_text, Some(line_location.clone()));
        let foreign_line = ForeignLine::from_text_content(line_content).at(line_location);
        children.push(ContentItem::ForeignLine(foreign_line));
    }

    // Validate that all children are ForeignLines
    validate_only_foreign_lines(&children);

    // Aggregate location from subject, all lines, and closing annotation
    let mut location_sources = vec![subject_location];
    location_sources.extend(line_locations);
    location_sources.push(closing_annotation.location.clone());
    let location = compute_location_from_locations(&location_sources);

    let foreign_block = ForeignBlock::new(subject, children, closing_annotation).at(location);

    ContentItem::ForeignBlock(Box::new(foreign_block))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::ast::Position;

    #[test]
    fn test_create_paragraph() {
        let source = "hello";
        let data = ParagraphData {
            text_lines: vec![("hello".to_string(), 0..5)],
            overall_byte_range: 0..5,
        };

        let result = create_paragraph(data, source);

        match result {
            ContentItem::Paragraph(para) => {
                assert_eq!(para.lines.len(), 1);
                assert_eq!(para.location.start, Position::new(0, 0));
                assert_eq!(para.location.end, Position::new(0, 5));
            }
            _ => panic!("Expected Paragraph"),
        }
    }

    #[test]
    fn test_create_session() {
        let source = "Session";
        let data = SessionData {
            title_text: "Session".to_string(),
            title_byte_range: 0..7,
        };

        let result = create_session(data, vec![], source);

        match result {
            ContentItem::Session(session) => {
                assert_eq!(session.title.as_string(), "Session");
                assert_eq!(session.location.start, Position::new(0, 0));
                assert_eq!(session.location.end, Position::new(0, 7));
            }
            _ => panic!("Expected Session"),
        }
    }

    // ============================================================================
    // VALIDATION TESTS
    // ============================================================================

    #[test]
    #[should_panic(expected = "Definition cannot contain Session elements")]
    fn test_definition_rejects_session_child() {
        use crate::lex::ast::elements::Session;

        let source = "Test Subject:\n    Nested Session\n";
        let session = Session::with_title("Nested Session".to_string());
        let content = vec![ContentItem::Session(session)];

        let data = DefinitionData {
            subject_text: "Test Subject".to_string(),
            subject_byte_range: 0..12,
        };

        // This should panic during validation
        let _definition = create_definition(data, content, source);
    }

    #[test]
    #[should_panic(expected = "Annotation cannot contain Session elements")]
    fn test_annotation_rejects_session_child() {
        use crate::lex::ast::elements::Session;

        let source = ":: test ::\n    Nested Session\n";
        let session = Session::with_title("Nested Session".to_string());
        let content = vec![ContentItem::Session(session)];

        let data = AnnotationData {
            label_text: "test".to_string(),
            label_byte_range: 0..4,
            parameters: vec![],
        };

        // This should panic during validation
        let _annotation = create_annotation(data, content, source);
    }

    #[test]
    #[should_panic(expected = "ListItem cannot contain Session elements")]
    fn test_list_item_rejects_session_child() {
        use crate::lex::ast::elements::Session;

        let source = "- Item\n    Nested Session\n";
        let session = Session::with_title("Nested Session".to_string());
        let content = vec![ContentItem::Session(session)];

        let data = ListItemData {
            marker_text: "- ".to_string(),
            marker_byte_range: 0..2,
        };

        // This should panic during validation
        let _list_item = create_list_item(data, content, source);
    }

    #[test]
    fn test_session_allows_session_child() {
        use crate::lex::ast::elements::Session;

        let source = "Parent Session\n    Nested Session\n";
        let nested_session = Session::with_title("Nested Session".to_string());
        let content = vec![ContentItem::Session(nested_session)];

        let data = SessionData {
            title_text: "Parent Session".to_string(),
            title_byte_range: 0..14,
        };

        // This should succeed - Sessions can contain Sessions
        let result = create_session(data, content, source);

        match result {
            ContentItem::Session(session) => {
                assert_eq!(session.children.len(), 1);
                assert_eq!(session.title.as_string(), "Parent Session");
            }
            _ => panic!("Expected Session"),
        }
    }

    #[test]
    fn test_definition_allows_non_session_children() {
        use crate::lex::ast::elements::Paragraph;

        let source = "Test Subject:\n    Some content\n";
        let para = Paragraph::from_line("Some content".to_string());
        let content = vec![ContentItem::Paragraph(para)];

        let data = DefinitionData {
            subject_text: "Test Subject".to_string(),
            subject_byte_range: 0..12,
        };

        // This should succeed - Definitions can contain Paragraphs
        let result = create_definition(data, content, source);

        match result {
            ContentItem::Definition(def) => {
                assert_eq!(def.children.len(), 1);
                assert_eq!(def.subject.as_string(), "Test Subject");
            }
            _ => panic!("Expected Definition"),
        }
    }

    #[test]
    fn test_annotation_allows_non_session_children() {
        use crate::lex::ast::elements::Paragraph;

        let source = ":: note ::\n    Some content\n";
        let para = Paragraph::from_line("Some content".to_string());
        let content = vec![ContentItem::Paragraph(para)];

        let data = AnnotationData {
            label_text: "note".to_string(),
            label_byte_range: 0..4,
            parameters: vec![],
        };

        // This should succeed - Annotations can contain Paragraphs
        let result = create_annotation(data, content, source);

        match result {
            ContentItem::Annotation(ann) => {
                assert_eq!(ann.children.len(), 1);
                assert_eq!(ann.label.value, "note");
            }
            _ => panic!("Expected Annotation"),
        }
    }

    #[test]
    fn test_list_item_allows_non_session_children() {
        use crate::lex::ast::elements::Paragraph;

        let source = "- Item\n    Some content\n";
        let para = Paragraph::from_line("Item content".to_string());
        let content = vec![ContentItem::Paragraph(para)];

        let data = ListItemData {
            marker_text: "- ".to_string(),
            marker_byte_range: 0..2,
        };

        // This should succeed - ListItems can contain Paragraphs
        let result = create_list_item(data, content, source);
        assert_eq!(result.children.len(), 1);
        assert_eq!(result.text(), "- ");
    }
}
