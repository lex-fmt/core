//! Common AST Builders for All Parsers
//!
//! This module contains unified AST node builders that work with both the reference
//! and linebased parsers. These builders encapsulate the core logic for constructing
//! AST nodes from extracted text, labels, and location information.
//!
//! The builders are designed to:
//! 1. Accept pre-extracted text and location data
//! 2. Handle the construction of AST nodes uniformly
//! 3. Aggregate locations from children as needed
//! 4. Support both combinator-based (reference) and direct unwrapping (linebased) approaches

use crate::txxt::ast::traits::AstNode;
use crate::txxt::ast::{
    Annotation, Definition, ForeignBlock, Label, List, ListItem, Location, Paragraph, Parameter,
    Session, TextContent, TextLine,
};
use crate::txxt::parsers::ContentItem;

use super::location::{aggregate_locations, compute_location_from_locations};

// ============================================================================
// PARAGRAPH BUILDER
// ============================================================================

/// Build a Paragraph from text lines, each with their own text and location.
///
/// This creates a Paragraph node containing TextLines, with an overall location
/// computed from all child locations.
///
/// # Arguments
/// * `text_lines` - Vec of (text, location) tuples for each line
/// * `overall_location` - The combined location for the entire paragraph
///
/// # Returns
/// A Paragraph ContentItem
pub fn build_paragraph(
    text_lines: Vec<(String, Location)>,
    overall_location: Location,
) -> ContentItem {
    let lines: Vec<ContentItem> = text_lines
        .into_iter()
        .map(|(text, location)| {
            let text_content = TextContent::from_string(text, Some(location));
            let text_line = TextLine::new(text_content).at(location);
            ContentItem::TextLine(text_line)
        })
        .collect();

    ContentItem::Paragraph(Paragraph {
        lines,
        location: overall_location,
    })
}

// ============================================================================
// SESSION BUILDER
// ============================================================================

/// Build a Session from a title and child content.
///
/// The location is aggregated from the title location and all child content locations.
///
/// # Arguments
/// * `title_text` - The session title text
/// * `title_location` - The location of the session title
/// * `content` - The child content items
///
/// # Returns
/// A Session ContentItem
pub fn build_session(
    title_text: String,
    title_location: Location,
    content: Vec<ContentItem>,
) -> ContentItem {
    let title = TextContent::from_string(title_text, Some(title_location));
    let location = aggregate_locations(title_location, &content);

    let session = Session::new(title, content).at(location);
    ContentItem::Session(session)
}

// ============================================================================
// DEFINITION BUILDER
// ============================================================================

/// Build a Definition from a subject and child content.
///
/// The location is aggregated from the subject location and all child content locations.
///
/// # Arguments
/// * `subject_text` - The definition subject text
/// * `subject_location` - The location of the subject
/// * `content` - The child content items
///
/// # Returns
/// A Definition ContentItem
pub fn build_definition(
    subject_text: String,
    subject_location: Location,
    content: Vec<ContentItem>,
) -> ContentItem {
    let subject = TextContent::from_string(subject_text, Some(subject_location));
    let location = aggregate_locations(subject_location, &content);

    let definition = Definition::new(subject, content).at(location);
    ContentItem::Definition(definition)
}

// ============================================================================
// ANNOTATION BUILDER
// ============================================================================

/// Build an Annotation from label, parameters, and content.
///
/// The location is computed from the label location and all child content locations.
///
/// # Arguments
/// * `label_text` - The annotation label text
/// * `label_location` - The location of the label
/// * `parameters` - The annotation parameters
/// * `content` - The child content items
///
/// # Returns
/// An Annotation ContentItem
pub fn build_annotation(
    label_text: String,
    label_location: Location,
    parameters: Vec<Parameter>,
    content: Vec<ContentItem>,
) -> ContentItem {
    let label = Label::new(label_text).at(label_location);
    let location = aggregate_locations(label_location, &content);

    ContentItem::Annotation(Annotation {
        label,
        parameters,
        content,
        location,
    })
}

// ============================================================================
// LIST BUILDER
// ============================================================================

/// Build a List from list items.
///
/// The location is computed from all child item locations.
///
/// # Arguments
/// * `items` - The list items
///
/// # Returns
/// A List ContentItem
pub fn build_list(items: Vec<ContentItem>) -> ContentItem {
    use crate::txxt::ast::location::Position;

    if items.is_empty() {
        // Create an empty list with default location
        return ContentItem::List(List {
            content: vec![],
            location: Location::default(),
        });
    }

    // Compute location from all items
    let location = {
        let item_locations: Vec<Location> = items.iter().map(|item| item.location()).collect();
        if item_locations.is_empty() {
            Location::default()
        } else {
            // Find bounding box for all items
            let min_line = item_locations
                .iter()
                .map(|l| l.start.line)
                .min()
                .unwrap_or(0);
            let min_col = item_locations
                .iter()
                .filter(|l| l.start.line == min_line)
                .map(|l| l.start.column)
                .min()
                .unwrap_or(0);
            let max_line = item_locations.iter().map(|l| l.end.line).max().unwrap_or(0);
            let max_col = item_locations
                .iter()
                .filter(|l| l.end.line == max_line)
                .map(|l| l.end.column)
                .max()
                .unwrap_or(0);

            Location::new(
                Position::new(min_line, min_col),
                Position::new(max_line, max_col),
            )
        }
    };

    ContentItem::List(List {
        content: items,
        location,
    })
}

// ============================================================================
// LIST ITEM BUILDER
// ============================================================================

/// Build a ListItem from bullet/number and child content.
///
/// The location is aggregated from the bullet/number and all child content locations.
///
/// # Arguments
/// * `marker_text` - The list marker (e.g., "-", "1.", etc.)
/// * `marker_location` - The location of the marker
/// * `content` - The child content items
///
/// # Returns
/// A ListItem ContentItem
pub fn build_list_item(
    item_text: String,
    item_location: Location,
    content: Vec<ContentItem>,
) -> ContentItem {
    let location = if content.is_empty() {
        item_location
    } else {
        aggregate_locations(item_location, &content)
    };

    let item = if content.is_empty() {
        ListItem::new(item_text).at(location)
    } else {
        let text_content = TextContent::from_string(item_text, None);
        ListItem::with_text_content(text_content, content).at(location)
    };

    ContentItem::ListItem(item)
}

// ============================================================================
// FOREIGN BLOCK BUILDER
// ============================================================================

/// Build a ForeignBlock from subject, content, and closing annotation.
///
/// The location is aggregated from the subject, content, and closing annotation locations.
///
/// # Arguments
/// * `subject_text` - The foreign block subject (opening marker)
/// * `subject_location` - The location of the subject
/// * `content_text` - The content text of the foreign block
/// * `content_location` - The location of the content
/// * `closing_annotation` - The closing annotation with its location
///
/// # Returns
/// A ForeignBlock ContentItem
pub fn build_foreign_block(
    subject_text: String,
    subject_location: Location,
    content_text: String,
    content_location: Location,
    closing_annotation: Annotation,
) -> ContentItem {
    let subject = TextContent::from_string(subject_text, Some(subject_location));
    let content = TextContent::from_string(content_text, Some(content_location));

    let location_sources = vec![
        subject_location,
        content_location,
        closing_annotation.location,
    ];
    let location = compute_location_from_locations(&location_sources);

    let foreign_block = ForeignBlock {
        subject,
        content,
        closing_annotation,
        location,
    };

    ContentItem::ForeignBlock(foreign_block)
}

// ============================================================================
// TEXT EXTRACTION HELPERS
// ============================================================================

/// Extract text from a byte range in the source.
///
/// This is the unified, common approach for extracting text from source byte ranges.
/// Both parsers have access to byte ranges, so both use this single implementation.
pub fn extract_text_from_span(source: &str, span: &std::ops::Range<usize>) -> String {
    if span.start >= span.end {
        return String::new();
    }
    // Clamp the span to the source length
    let end = span.end.min(source.len());
    if span.start >= end {
        return String::new();
    }
    source[span.start..end].trim().to_string()
}
