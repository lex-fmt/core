//! AST Construction Facade
//!
//! This module provides the facade layer that sits between parsers and base builders.
//! It handles the conversion from parser-specific token structures to the data format
//! expected by the common AST builders.
//!
//! # Architecture
//!
//! ```text
//! Parser (linebased/reference)
//!     ↓ sends: LineTokens + source string
//!     ↓
//! Facade Layer (this module)
//!     ↓ does 3 things:
//!     1. Unroll tokens → Vec<(Token, Range<usize>)>
//!     2. Convert Range<usize> → Location
//!     3. Extract text
//!     ↓ calls base builders with correct data
//!     ↓
//! Base Builders (builders.rs)
//!     ↓ creates AST nodes
//! ```
//!
//! # Usage
//!
//! Parsers should call these facade functions instead of the base builders directly:
//!
//! ```rust,ignore
//! use crate::txxt::parsers::common::ast_construction;
//!
//! // In linebased parser:
//! let paragraph = ast_construction::build_paragraph_from_line_tokens(
//!     &line_tokens,
//!     source
//! );
//!
//! // Facade handles unrolling, location conversion, and calls base builder
//! ```

use crate::txxt::lexers::linebased::tokens::LineToken;
use crate::txxt::lexers::tokens::Token;
use crate::txxt::parsers::ContentItem;
use std::ops::Range;

use super::builders;
use super::token_processing::{
    compute_bounding_box, extract_text, flatten_token_vecs, range_to_location,
};

// ============================================================================
// PARAGRAPH CONSTRUCTION
// ============================================================================

/// Build a Paragraph from LineTokens.
///
/// This facade function:
/// 1. Extracts source tokens from each LineToken
/// 2. Computes location for each line from its tokens
/// 3. Extracts text for each line
/// 4. Calls the base paragraph builder
///
/// # Arguments
///
/// * `line_tokens` - The LineTokens representing paragraph lines
/// * `source` - The original source string (needed for text extraction and location conversion)
///
/// # Returns
///
/// A Paragraph ContentItem with accurate locations
///
/// # Example
///
/// ```rust,ignore
/// let paragraph_tokens: Vec<LineToken> = /* ... from parser ... */;
/// let paragraph = build_paragraph_from_line_tokens(&paragraph_tokens, source);
/// ```
pub fn build_paragraph_from_line_tokens(line_tokens: &[LineToken], source: &str) -> ContentItem {
    // Extract (text, location) for each line
    let text_lines: Vec<(String, crate::txxt::ast::Location)> = line_tokens
        .iter()
        .map(|line_token| {
            // Get source tokens for this line
            let tokens = line_token.source_token_pairs();

            // Compute bounding box
            let range = compute_bounding_box(&tokens);

            // Convert to location and extract text
            let location = range_to_location(range.clone(), source);
            let text = extract_text(range, source);

            (text, location)
        })
        .collect();

    // Compute overall location from all line tokens
    let all_token_vecs: Vec<Vec<(Token, Range<usize>)>> = line_tokens
        .iter()
        .map(|lt| lt.source_token_pairs())
        .collect();
    let all_tokens = flatten_token_vecs(&all_token_vecs);
    let overall_range = compute_bounding_box(&all_tokens);
    let overall_location = range_to_location(overall_range, source);

    // Call base builder
    builders::build_paragraph(text_lines, overall_location)
}

// ============================================================================
// SESSION CONSTRUCTION
// ============================================================================

/// Build a Session from a title LineToken and content items.
///
/// # Arguments
///
/// * `title_token` - The LineToken for the session title
/// * `content` - The child content items (already constructed)
/// * `source` - The original source string
///
/// # Returns
///
/// A Session ContentItem with accurate location
pub fn build_session_from_line_token(
    title_token: &LineToken,
    content: Vec<ContentItem>,
    source: &str,
) -> ContentItem {
    // Extract title text and location
    let title_tokens = title_token.source_token_pairs();
    let title_range = compute_bounding_box(&title_tokens);
    let title_location = range_to_location(title_range.clone(), source);
    let title_text = extract_text(title_range, source);

    // Call base builder
    builders::build_session(title_text, title_location, content)
}

// ============================================================================
// DEFINITION CONSTRUCTION
// ============================================================================

/// Build a Definition from a subject LineToken and content items.
///
/// # Arguments
///
/// * `subject_token` - The LineToken for the definition subject
/// * `content` - The child content items (already constructed)
/// * `source` - The original source string
///
/// # Returns
///
/// A Definition ContentItem with accurate location
pub fn build_definition_from_line_token(
    subject_token: &LineToken,
    content: Vec<ContentItem>,
    source: &str,
) -> ContentItem {
    // Extract subject text and location
    let subject_tokens = subject_token.source_token_pairs();
    let subject_range = compute_bounding_box(&subject_tokens);
    let subject_location = range_to_location(subject_range.clone(), source);
    let subject_text = extract_text(subject_range, source);

    // Call base builder
    builders::build_definition(subject_text, subject_location, content)
}

// ============================================================================
// LIST CONSTRUCTION
// ============================================================================

/// Build a List from ListItems.
///
/// # Arguments
///
/// * `items` - The list items (already constructed)
/// * `source` - The original source string
///
/// # Returns
///
/// A List ContentItem with location computed from all items
pub fn build_list_from_items(items: Vec<ContentItem>, _source: &str) -> ContentItem {
    // The base builder will compute location from items
    // We just pass through since items already have locations
    builders::build_list(items)
}

/// Build a ListItem from a marker LineToken and content items.
///
/// # Arguments
///
/// * `marker_token` - The LineToken containing the list marker and item text
/// * `content` - The child content items (already constructed)
/// * `source` - The original source string
///
/// # Returns
///
/// A ListItem ContentItem with accurate location
pub fn build_list_item_from_line_token(
    marker_token: &LineToken,
    content: Vec<ContentItem>,
    source: &str,
) -> ContentItem {
    // Extract marker tokens
    let marker_tokens = marker_token.source_token_pairs();
    let marker_range = compute_bounding_box(&marker_tokens);
    let marker_location = range_to_location(marker_range.clone(), source);
    let item_text = extract_text(marker_range, source);

    // Call base builder
    builders::build_list_item(item_text, marker_location, content)
}

// ============================================================================
// ANNOTATION CONSTRUCTION
// ============================================================================

/// Build an Annotation from LineTokens.
///
/// # Arguments
///
/// * `label_token` - LineToken for the annotation label
/// * `parameters` - The annotation parameters (already parsed)
/// * `content` - The child content items (already constructed)
/// * `source` - The original source string
///
/// # Returns
///
/// An Annotation ContentItem with accurate location
pub fn build_annotation_from_line_token(
    label_token: &LineToken,
    parameters: Vec<crate::txxt::ast::Parameter>,
    content: Vec<ContentItem>,
    source: &str,
) -> ContentItem {
    // Extract label
    let label_tokens = label_token.source_token_pairs();
    let label_range = compute_bounding_box(&label_tokens);
    let label_location = range_to_location(label_range.clone(), source);
    let label_text = extract_text(label_range, source);

    // Call base builder
    builders::build_annotation(label_text, label_location, parameters, content)
}

// ============================================================================
// FOREIGN BLOCK CONSTRUCTION
// ============================================================================

/// Build a ForeignBlock from a title LineToken and raw content.
///
/// # Arguments
///
/// * `subject_token` - The LineToken for the foreign block subject/title
/// * `content_text` - The raw string content of the block
/// * `content_location` - The location of the content
/// * `closing_annotation` - The closing annotation marker
/// * `source` - The original source string
///
/// # Returns
///
/// A ForeignBlock ContentItem with accurate location
pub fn build_foreign_block_from_line_token(
    subject_token: &LineToken,
    content_text: String,
    content_location: crate::txxt::ast::Location,
    closing_annotation: crate::txxt::ast::Annotation,
    source: &str,
) -> ContentItem {
    // Extract subject
    let subject_tokens = subject_token.source_token_pairs();
    let subject_range = compute_bounding_box(&subject_tokens);
    let subject_location = range_to_location(subject_range.clone(), source);
    let subject_text = extract_text(subject_range, source);

    // Call base builder
    builders::build_foreign_block(
        subject_text,
        subject_location,
        content_text,
        content_location,
        closing_annotation,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::txxt::lexers::linebased::tokens::LineTokenType;
    use crate::txxt::lexers::tokens::Token;

    #[test]
    fn test_build_paragraph_from_line_tokens() {
        // Create mock line tokens
        #[allow(clippy::single_range_in_vec_init)]
        let line_tokens = vec![
            LineToken {
                source_tokens: vec![Token::Text("Hello".to_string())],
                token_spans: vec![0..5],
                line_type: LineTokenType::ParagraphLine,
                source_span: Some(0..5),
            },
            LineToken {
                source_tokens: vec![Token::Text("World".to_string())],
                token_spans: vec![6..11],
                line_type: LineTokenType::ParagraphLine,
                source_span: Some(6..11),
            },
        ];

        let source = "Hello\nWorld";
        let paragraph = build_paragraph_from_line_tokens(&line_tokens, source);

        // Verify it's a paragraph
        match paragraph {
            ContentItem::Paragraph(p) => {
                assert_eq!(p.lines.len(), 2);
                // Verify location spans the entire paragraph
                assert_eq!(p.location.start.line, 0);
                assert_eq!(p.location.end.line, 1);
            }
            _ => panic!("Expected Paragraph"),
        }
    }

    #[test]
    fn test_build_session_from_line_token() {
        let title_token = LineToken {
            source_tokens: vec![Token::Text("Title".to_string()), Token::Colon],
            token_spans: vec![0..5, 5..6],
            line_type: LineTokenType::SubjectLine,
            source_span: Some(0..6),
        };

        let source = "Title:";
        let content = vec![]; // Empty content for test

        let session = build_session_from_line_token(&title_token, content, source);

        match session {
            ContentItem::Session(s) => {
                assert_eq!(s.title.as_ref(), "Title:");
                assert_eq!(s.location.start.line, 0);
            }
            _ => panic!("Expected Session"),
        }
    }

    #[test]
    fn test_build_definition_from_line_token() {
        let subject_token = LineToken {
            source_tokens: vec![Token::Text("Subject".to_string()), Token::Colon],
            token_spans: vec![0..7, 7..8],
            line_type: LineTokenType::SubjectLine,
            source_span: Some(0..8),
        };

        let source = "Subject:";
        let content = vec![];

        let definition = build_definition_from_line_token(&subject_token, content, source);

        match definition {
            ContentItem::Definition(d) => {
                assert_eq!(d.subject.as_ref(), "Subject:");
                assert_eq!(d.location.start.line, 0);
            }
            _ => panic!("Expected Definition"),
        }
    }
}
