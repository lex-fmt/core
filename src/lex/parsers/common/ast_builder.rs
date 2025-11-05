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

/// Build an Annotation AST node from a label token, parameters, and content.
///
/// # Arguments
///
/// * `label_token` - LineToken for the annotation label
/// * `parameters` - Annotation parameters (already parsed)
/// * `content` - Child content items (already constructed)
/// * `source` - Original source string
///
/// # Returns
///
/// An Annotation ContentItem
pub fn build_annotation(
    label_token: &LineToken,
    parameters: Vec<Parameter>,
    content: Vec<ContentItem>,
    source: &str,
) -> ContentItem {
    // 1. Normalize
    let tokens = token_normalization::normalize_line_token(label_token);

    // 2. Extract
    let data = data_extraction::extract_annotation_data(tokens, source);

    // 3. Create
    ast_creation::create_annotation(data, parameters, content, source)
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
/// # Arguments
///
/// * `label_tokens` - Normalized tokens for the annotation label
/// * `parameters` - Annotation parameters (already parsed)
/// * `content` - Child content items (already constructed)
/// * `source` - Original source string
///
/// # Returns
///
/// An Annotation ContentItem
pub fn build_annotation_from_tokens(
    label_tokens: Vec<(Token, ByteRange<usize>)>,
    parameters: Vec<Parameter>,
    content: Vec<ContentItem>,
    source: &str,
) -> ContentItem {
    // Skip normalization, tokens already normalized
    // 1. Extract
    let data = data_extraction::extract_annotation_data(label_tokens, source);

    // 2. Create
    ast_creation::create_annotation(data, parameters, content, source)
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
