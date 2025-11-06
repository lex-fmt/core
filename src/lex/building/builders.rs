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

    ContentItem::Paragraph(Paragraph {
        lines: crate::lex::ast::elements::container::Container::new(lines),
        location: overall_location,
    })
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
        items: crate::lex::ast::elements::container::Container::new(content),
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
        children: crate::lex::ast::elements::container::Container::new(content),
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
    let subject_location = byte_range_to_ast_range(data.subject_byte_range, source);
    let subject = TextContent::from_string(data.subject_text, Some(subject_location.clone()));

    let content_location = byte_range_to_ast_range(data.content_byte_range, source);
    let content = TextContent::from_string(data.content_text, Some(content_location.clone()));

    // Aggregate location from subject, content, and closing annotation
    let location_sources = vec![
        subject_location,
        content_location,
        closing_annotation.location.clone(),
    ];
    let location = compute_location_from_locations(&location_sources);

    let foreign_block = ForeignBlock {
        subject,
        content,
        closing_annotation,
        location,
    };

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
}
