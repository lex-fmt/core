//! Public AST Builder API
//!
//! This module provides the public API for building AST nodes from tokens.
//! It coordinates the three-layer architecture:
//!
//! 1. Token Normalization - Convert various token formats to standard vectors
//! 2. Data Extraction - Extract primitive data (text, byte ranges) from tokens
//! 3. AST Creation - Convert primitives to AST nodes with ast::Range
//!
//! # Architecture
//!
//! ```text
//! Tokens → normalize → extract → create → AST Nodes
//!   ↓          ↓          ↓          ↓
//! Parser  token_norm  data_ext  ast_create
//! ```
//!
//! # Usage
//!
//! Parsers should **only** use functions from this module. They should never:
//! - Extract text manually
//! - Call data_extraction functions directly
//! - Call ast_creation functions directly
//!
//! ```rust,ignore
//! use crate::lex::parsers::common::ast_builder;
//!
//! // In parser:
//! let paragraph = ast_builder::build_paragraph(&line_tokens, source);
//! let session = ast_builder::build_session(&title_token, content, source);
//! ```

use crate::lex::ast::traits::AstNode;
use crate::lex::ast::{Annotation, ListItem, Parameter};
use crate::lex::lexers::linebased::tokens_linebased::LineToken;
use crate::lex::parsers::ContentItem;

use super::ast_creation;
use super::data_extraction;
use super::token_normalization;

// ============================================================================
// PARAGRAPH BUILDING
// ============================================================================

/// Build a Paragraph AST node from line tokens.
///
/// This is the complete pipeline: normalize → extract → create.
///
/// # Arguments
///
/// * `line_tokens` - LineTokens representing paragraph lines
/// * `source` - Original source string
///
/// # Returns
///
/// A Paragraph ContentItem
///
/// # Example
///
/// ```rust,ignore
/// let line_tokens: Vec<LineToken> = /* ... from parser ... */;
/// let paragraph = build_paragraph(&line_tokens, source);
/// ```
pub fn build_paragraph(line_tokens: &[LineToken], source: &str) -> ContentItem {
    // 1. Normalize: LineTokens → Vec<Vec<(Token, Range<usize>)>>
    let token_lines = token_normalization::normalize_line_tokens(line_tokens);

    // 2. Extract: normalized tokens → ParagraphData
    let data = data_extraction::extract_paragraph_data(token_lines, source);

    // 3. Create: ParagraphData → Paragraph AST node
    ast_creation::create_paragraph(data, source)
}

// ============================================================================
// SESSION BUILDING
// ============================================================================

/// Build a Session AST node from a title token and content.
///
/// # Arguments
///
/// * `title_token` - LineToken for the session title
/// * `content` - Child content items (already constructed)
/// * `source` - Original source string
///
/// # Returns
///
/// A Session ContentItem
pub fn build_session(
    title_token: &LineToken,
    content: Vec<ContentItem>,
    source: &str,
) -> ContentItem {
    // 1. Normalize
    let tokens = token_normalization::normalize_line_token(title_token);

    // 2. Extract
    let data = data_extraction::extract_session_data(tokens, source);

    // 3. Create
    ast_creation::create_session(data, content, source)
}

// ============================================================================
// DEFINITION BUILDING
// ============================================================================

/// Build a Definition AST node from a subject token and content.
///
/// # Arguments
///
/// * `subject_token` - LineToken for the definition subject
/// * `content` - Child content items (already constructed)
/// * `source` - Original source string
///
/// # Returns
///
/// A Definition ContentItem
pub fn build_definition(
    subject_token: &LineToken,
    content: Vec<ContentItem>,
    source: &str,
) -> ContentItem {
    // 1. Normalize
    let tokens = token_normalization::normalize_line_token(subject_token);

    // 2. Extract
    let data = data_extraction::extract_definition_data(tokens, source);

    // 3. Create
    ast_creation::create_definition(data, content, source)
}

// ============================================================================
// LIST BUILDING
// ============================================================================

/// Build a List AST node from list items.
///
/// # Arguments
///
/// * `items` - Vector of ListItem nodes
///
/// # Returns
///
/// A List ContentItem
pub fn build_list(items: Vec<ListItem>) -> ContentItem {
    // No normalization/extraction needed - items already constructed
    ast_creation::create_list(items)
}

// ============================================================================
// LIST ITEM BUILDING
// ============================================================================

/// Build a ListItem AST node from a marker token and content.
///
/// # Arguments
///
/// * `marker_token` - LineToken for the list item marker
/// * `content` - Child content items (already constructed)
/// * `source` - Original source string
///
/// # Returns
///
/// A ListItem node (not wrapped in ContentItem)
pub fn build_list_item(
    marker_token: &LineToken,
    content: Vec<ContentItem>,
    source: &str,
) -> ListItem {
    // 1. Normalize
    let tokens = token_normalization::normalize_line_token(marker_token);

    // 2. Extract
    let data = data_extraction::extract_list_item_data(tokens, source);

    // 3. Create
    ast_creation::create_list_item(data, content, source)
}

// ============================================================================
// ANNOTATION BUILDING
// ============================================================================

/// Build an Annotation AST node from a label token and content.
///
/// Goes through the full pipeline: normalize → extract (with label/param parsing) → create.
///
/// # Arguments
///
/// * `label_token` - LineToken for the annotation label (includes label and parameters between :: markers)
/// * `content` - Child content items (already constructed)
/// * `source` - Original source string
///
/// # Returns
///
/// An Annotation ContentItem
pub fn build_annotation(
    label_token: &LineToken,
    content: Vec<ContentItem>,
    source: &str,
) -> ContentItem {
    // 1. Normalize
    let tokens = token_normalization::normalize_line_token(label_token);

    // 2. Extract (parses label AND parameters from tokens)
    let data = data_extraction::extract_annotation_data(tokens, source);

    // 3. Create
    ast_creation::create_annotation(data, content, source)
}

// ============================================================================
// FOREIGN BLOCK BUILDING
// ============================================================================

/// Build a ForeignBlock AST node from subject, content, and closing annotation.
///
/// This function implements the indentation wall stripping logic - content at
/// different nesting levels will have identical text after wall removal.
///
/// # Arguments
///
/// * `subject_token` - LineToken for the foreign block subject
/// * `content_tokens` - LineTokens for each content line
/// * `closing_annotation` - The closing annotation node
/// * `source` - Original source string
///
/// # Returns
///
/// A ForeignBlock ContentItem
///
/// # Example
///
/// ```rust,ignore
/// // Top-level: "Code:\n    line1\n    line2\n:: js ::"
/// // Nested:    "Session:\n    Code:\n        line1\n        line2\n    :: js ::"
/// //
/// // Both produce ForeignBlock with content: "line1\nline2"
/// // The indentation wall (minimum indentation) is stripped.
/// ```
pub fn build_foreign_block(
    subject_token: &LineToken,
    content_tokens: Vec<&LineToken>,
    closing_annotation: Annotation,
    source: &str,
) -> ContentItem {
    // 1. Normalize subject
    let subject_tokens = token_normalization::normalize_line_token(subject_token);

    // 2. Normalize content (preserving line boundaries for wall calculation)
    let content_token_lines: Vec<Vec<_>> = content_tokens
        .iter()
        .map(|lt| token_normalization::normalize_line_token(lt))
        .collect();

    // 3. Extract (includes indentation wall stripping)
    let data =
        data_extraction::extract_foreign_block_data(subject_tokens, content_token_lines, source);

    // 4. Create
    ast_creation::create_foreign_block(data, closing_annotation, source)
}

// ============================================================================
// REFERENCE PARSER API (tokens already normalized)
// ============================================================================
//
// The reference parser works with flat token streams (Token, ByteRange) that
// are already in normalized format. These functions skip the normalization
// step and go straight to data extraction.

use crate::lex::lexers::tokens_core::Token;
use std::ops::Range as ByteRange;

/// Build a Paragraph from already-normalized token lines (for reference parser).
///
/// # Arguments
///
/// * `token_lines` - Normalized token vectors, one per line
/// * `source` - Original source string
///
/// # Returns
///
/// A Paragraph ContentItem
pub fn build_paragraph_from_tokens(
    token_lines: Vec<Vec<(Token, ByteRange<usize>)>>,
    source: &str,
) -> ContentItem {
    // Skip normalization, tokens already normalized
    // 1. Extract
    let data = data_extraction::extract_paragraph_data(token_lines, source);

    // 2. Create
    ast_creation::create_paragraph(data, source)
}

/// Build a Session from already-normalized title tokens (for reference parser).
///
/// # Arguments
///
/// * `title_tokens` - Normalized tokens for the session title
/// * `content` - Child content items (already constructed)
/// * `source` - Original source string
///
/// # Returns
///
/// A Session ContentItem
pub fn build_session_from_tokens(
    title_tokens: Vec<(Token, ByteRange<usize>)>,
    content: Vec<ContentItem>,
    source: &str,
) -> ContentItem {
    // Skip normalization, tokens already normalized
    // 1. Extract
    let data = data_extraction::extract_session_data(title_tokens, source);

    // 2. Create
    ast_creation::create_session(data, content, source)
}

/// Build a Definition from already-normalized subject tokens (for reference parser).
///
/// # Arguments
///
/// * `subject_tokens` - Normalized tokens for the definition subject
/// * `content` - Child content items (already constructed)
/// * `source` - Original source string
///
/// # Returns
///
/// A Definition ContentItem
pub fn build_definition_from_tokens(
    subject_tokens: Vec<(Token, ByteRange<usize>)>,
    content: Vec<ContentItem>,
    source: &str,
) -> ContentItem {
    // Skip normalization, tokens already normalized
    // 1. Extract
    let data = data_extraction::extract_definition_data(subject_tokens, source);

    // 2. Create
    ast_creation::create_definition(data, content, source)
}

/// Build a ListItem from already-normalized marker tokens (for reference parser).
///
/// # Arguments
///
/// * `marker_tokens` - Normalized tokens for the list item marker and text
/// * `content` - Child content items (already constructed)
/// * `source` - Original source string
///
/// # Returns
///
/// A ListItem node (not wrapped in ContentItem)
pub fn build_list_item_from_tokens(
    marker_tokens: Vec<(Token, ByteRange<usize>)>,
    content: Vec<ContentItem>,
    source: &str,
) -> ListItem {
    // Skip normalization, tokens already normalized
    // 1. Extract
    let data = data_extraction::extract_list_item_data(marker_tokens, source);

    // 2. Create
    ast_creation::create_list_item(data, content, source)
}

/// Build an Annotation from already-normalized label tokens (for reference parser).
///
/// Skips normalization, goes through: extract (with label/param parsing) → create.
///
/// # Arguments
///
/// * `label_tokens` - Normalized tokens for the annotation label (includes label and parameters)
/// * `content` - Child content items (already constructed)
/// * `source` - Original source string
///
/// # Returns
///
/// An Annotation ContentItem
pub fn build_annotation_from_tokens(
    label_tokens: Vec<(Token, ByteRange<usize>)>,
    content: Vec<ContentItem>,
    source: &str,
) -> ContentItem {
    // Skip normalization, tokens already normalized
    // 1. Extract (parses label AND parameters from tokens)
    let data = data_extraction::extract_annotation_data(label_tokens, source);

    // 2. Create
    ast_creation::create_annotation(data, content, source)
}

/// Build a ForeignBlock from already-normalized tokens (for reference parser).
///
/// This implements indentation wall stripping - content at different nesting
/// levels will have identical text after wall removal.
///
/// # Arguments
///
/// * `subject_tokens` - Normalized tokens for the foreign block subject
/// * `content_token_lines` - Normalized token vectors for each content line
/// * `closing_annotation` - The closing annotation node
/// * `source` - Original source string
///
/// # Returns
///
/// A ForeignBlock ContentItem
///
/// # Example
///
/// ```rust,ignore
/// // Tokens already normalized from reference parser
/// let subject_tokens = vec![(Token::Text("Code".into()), 0..4)];
/// let content_lines = vec![
///     vec![(Token::Indentation, 6..10), (Token::Text("line1".into()), 10..15)],
///     vec![(Token::Indentation, 16..20), (Token::Indentation, 20..24), (Token::Text("line2".into()), 24..29)],
/// ];
/// // After extraction, wall of 1 indent is stripped: "line1\n    line2"
/// ```
pub fn build_foreign_block_from_tokens(
    subject_tokens: Vec<(Token, ByteRange<usize>)>,
    content_token_lines: Vec<Vec<(Token, ByteRange<usize>)>>,
    closing_annotation: Annotation,
    source: &str,
) -> ContentItem {
    // Skip normalization, tokens already normalized
    // 1. Extract (includes indentation wall stripping)
    let data =
        data_extraction::extract_foreign_block_data(subject_tokens, content_token_lines, source);

    // 2. Create
    ast_creation::create_foreign_block(data, closing_annotation, source)
}

// ============================================================================
// TEXT-BASED API (for reference parser and simple cases)
// ============================================================================
//
// These functions accept pre-extracted text and ast::Range locations.
// They are used by the reference parser which extracts text during parsing,
// and for simple cases where text has already been extracted.

/// Build a Paragraph from pre-extracted text lines with locations.
///
/// # Arguments
///
/// * `text_lines` - Vec of (text, location) tuples for each line
/// * `overall_location` - The combined location for the entire paragraph
///
/// # Returns
///
/// A Paragraph ContentItem
pub fn build_paragraph_from_text(
    text_lines: Vec<(String, crate::lex::ast::Range)>,
    overall_location: crate::lex::ast::Range,
) -> ContentItem {
    use crate::lex::ast::{Paragraph, TextContent, TextLine};

    let lines: Vec<ContentItem> = text_lines
        .into_iter()
        .map(|(text, location)| {
            let text_content = TextContent::from_string(text, Some(location.clone()));
            let text_line = TextLine::new(text_content).at(location);
            ContentItem::TextLine(text_line)
        })
        .collect();

    ContentItem::Paragraph(Paragraph {
        lines,
        location: overall_location,
    })
}

/// Build a Session from pre-extracted title text and location.
///
/// # Arguments
///
/// * `title_text` - The session title text
/// * `title_location` - The location of the session title
/// * `content` - The child content items
///
/// # Returns
///
/// A Session ContentItem
pub fn build_session_from_text(
    title_text: String,
    title_location: crate::lex::ast::Range,
    content: Vec<ContentItem>,
) -> ContentItem {
    use crate::lex::ast::{Session, TextContent};
    use crate::lex::parsers::common::location::aggregate_locations;

    let title = TextContent::from_string(title_text, Some(title_location.clone()));
    let location = aggregate_locations(title_location, &content);

    let session = Session::new(title, content).at(location);
    ContentItem::Session(session)
}

/// Build a Definition from pre-extracted subject text and location.
///
/// # Arguments
///
/// * `subject_text` - The definition subject text
/// * `subject_location` - The location of the subject
/// * `content` - The child content items
///
/// # Returns
///
/// A Definition ContentItem
pub fn build_definition_from_text(
    subject_text: String,
    subject_location: crate::lex::ast::Range,
    content: Vec<ContentItem>,
) -> ContentItem {
    use crate::lex::ast::{Definition, TextContent};
    use crate::lex::parsers::common::location::aggregate_locations;

    let subject = TextContent::from_string(subject_text, Some(subject_location.clone()));
    let location = aggregate_locations(subject_location, &content);

    let definition = Definition::new(subject, content).at(location);
    ContentItem::Definition(definition)
}

/// Build an Annotation from pre-extracted label text and location.
///
/// # Arguments
///
/// * `label_text` - The annotation label text
/// * `label_location` - The location of the label
/// * `parameters` - The annotation parameters
/// * `content` - The child content items
///
/// # Returns
///
/// An Annotation ContentItem
pub fn build_annotation_from_text(
    label_text: String,
    label_location: crate::lex::ast::Range,
    parameters: Vec<Parameter>,
    content: Vec<ContentItem>,
) -> ContentItem {
    use crate::lex::ast::Label;
    use crate::lex::parsers::common::location::aggregate_locations;

    let label = Label::new(label_text).at(label_location.clone());
    let location = aggregate_locations(label_location, &content);

    ContentItem::Annotation(Annotation {
        label,
        parameters,
        content,
        location,
    })
}

/// Build a List from content items.
///
/// # Arguments
///
/// * `items` - The list items as ContentItems
///
/// # Returns
///
/// A List ContentItem
pub fn build_list_from_items(items: Vec<ContentItem>) -> ContentItem {
    use crate::lex::ast::range::Position;
    use crate::lex::ast::List;

    if items.is_empty() {
        return ContentItem::List(List {
            content: vec![],
            location: crate::lex::ast::Range::default(),
        });
    }

    // Compute location from all items
    let item_locations: Vec<crate::lex::ast::Range> =
        items.iter().map(|item| item.range().clone()).collect();

    let location = if item_locations.is_empty() {
        crate::lex::ast::Range::default()
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

        crate::lex::ast::Range::new(
            0..0,
            Position::new(min_line, min_col),
            Position::new(max_line, max_col),
        )
    };

    ContentItem::List(List {
        content: items,
        location,
    })
}

/// Build a ForeignBlock from pre-extracted text and locations.
///
/// NOTE: This does NOT perform indentation wall stripping.
/// Use build_foreign_block_from_tokens for proper indentation handling.
///
/// # Arguments
///
/// * `subject_text` - The foreign block subject text
/// * `subject_location` - The location of the subject
/// * `content_text` - The content text
/// * `content_location` - The location of the content
/// * `closing_annotation` - The closing annotation
///
/// # Returns
///
/// A ForeignBlock ContentItem
pub fn build_foreign_block_from_text(
    subject_text: String,
    subject_location: crate::lex::ast::Range,
    content_text: String,
    content_location: crate::lex::ast::Range,
    closing_annotation: Annotation,
) -> ContentItem {
    use crate::lex::ast::{ForeignBlock, TextContent};
    use crate::lex::parsers::common::location::compute_location_from_locations;

    let subject = TextContent::from_string(subject_text, Some(subject_location.clone()));
    let content = TextContent::from_string(content_text, Some(content_location.clone()));

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
    use crate::lex::lexers::linebased::tokens_linebased::LineType;
    use crate::lex::lexers::tokens_core::Token;

    fn make_line_token(tokens: Vec<Token>, spans: Vec<std::ops::Range<usize>>) -> LineToken {
        LineToken {
            source_tokens: tokens,
            token_spans: spans,
            line_type: LineType::ParagraphLine,
        }
    }

    #[test]
    fn test_build_paragraph() {
        let source = "hello world";
        let line_tokens = vec![make_line_token(
            vec![
                Token::Text("hello".to_string()),
                Token::Whitespace,
                Token::Text("world".to_string()),
            ],
            vec![0..5, 5..6, 6..11],
        )];

        let result = build_paragraph(&line_tokens, source);

        match result {
            ContentItem::Paragraph(para) => {
                assert_eq!(para.lines.len(), 1);
            }
            _ => panic!("Expected Paragraph"),
        }
    }

    #[test]
    fn test_build_session() {
        let source = "Session:";
        let title_token = make_line_token(
            vec![Token::Text("Session".to_string()), Token::Colon],
            vec![0..7, 7..8],
        );

        let result = build_session(&title_token, vec![], source);

        match result {
            ContentItem::Session(session) => {
                assert_eq!(session.title.as_string(), "Session:");
            }
            _ => panic!("Expected Session"),
        }
    }
}
