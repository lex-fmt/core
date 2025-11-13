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
//! use crate::lex::building::api;
//!
//! // In parser:
//! let paragraph = ast_builder::build_paragraph(&line_tokens, source);
//! let session_children: Vec<SessionContent> = vec![];
//! let session = ast_builder::build_session(&title_token, session_children, source);
//! ```

use crate::lex::ast::elements::typed_content::{ContentElement, SessionContent};
use crate::lex::ast::{Annotation, ListItem};
use crate::lex::lexing::tokens_linebased::LineToken;
use crate::lex::parsing::ContentItem;

use super::builders;
use super::extraction;
use super::normalization;

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
    let token_lines = normalization::normalize_line_tokens(line_tokens);

    // 2. Extract: normalized tokens → ParagraphData
    let data = extraction::extract_paragraph_data(token_lines, source);

    // 3. Create: ParagraphData → Paragraph AST node
    builders::create_paragraph(data, source)
}

// ============================================================================
// SESSION BUILDING
// ============================================================================

/// Build a Session AST node from a title token and content.
///
/// # Arguments
///
/// * `title_token` - LineToken for the session title
/// * `content` - Typed child content items (already constructed)
/// * `source` - Original source string
///
/// # Returns
///
/// A Session ContentItem
pub fn build_session(
    title_token: &LineToken,
    content: Vec<SessionContent>,
    source: &str,
) -> ContentItem {
    // 1. Normalize
    let tokens = normalization::normalize_line_token(title_token);

    // 2. Extract
    let data = extraction::extract_session_data(tokens, source);

    // 3. Create
    builders::create_session(data, content, source)
}

// ============================================================================
// DEFINITION BUILDING
// ============================================================================

/// Build a Definition AST node from a subject token and content.
///
/// # Arguments
///
/// * `subject_token` - LineToken for the definition subject
/// * `content` - Typed child content items (already constructed)
/// * `source` - Original source string
///
/// # Returns
///
/// A Definition ContentItem
pub fn build_definition(
    subject_token: &LineToken,
    content: Vec<ContentElement>,
    source: &str,
) -> ContentItem {
    // 1. Normalize
    let tokens = normalization::normalize_line_token(subject_token);

    // 2. Extract
    let data = extraction::extract_definition_data(tokens, source);

    // 3. Create
    builders::create_definition(data, content, source)
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
    builders::create_list(items)
}

// ============================================================================
// LIST ITEM BUILDING
// ============================================================================

/// Build a ListItem AST node from a marker token and content.
///
/// # Arguments
///
/// * `marker_token` - LineToken for the list item marker
/// * `content` - Typed child content items (already constructed)
/// * `source` - Original source string
///
/// # Returns
///
/// A ListItem node (not wrapped in ContentItem)
pub fn build_list_item(
    marker_token: &LineToken,
    content: Vec<ContentElement>,
    source: &str,
) -> ListItem {
    // 1. Normalize
    let tokens = normalization::normalize_line_token(marker_token);

    // 2. Extract
    let data = extraction::extract_list_item_data(tokens, source);

    // 3. Create
    builders::create_list_item(data, content, source)
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
/// * `content` - Typed child content items (already constructed)
/// * `source` - Original source string
///
/// # Returns
///
/// An Annotation ContentItem
pub fn build_annotation(
    label_token: &LineToken,
    content: Vec<ContentElement>,
    source: &str,
) -> ContentItem {
    // 1. Normalize
    let tokens = normalization::normalize_line_token(label_token);

    // 2. Extract (parses label AND parameters from tokens)
    let data = extraction::extract_annotation_data(tokens, source);

    // 3. Create
    builders::create_annotation(data, content, source)
}

// ============================================================================
// VERBATIM BLOCK BUILDING
// ============================================================================

/// Build a VerbatimBlock AST node from subject, content, and closing annotation.
///
/// This function implements the indentation wall stripping logic - content at
/// different nesting levels will have identical text after wall removal.
///
/// # Arguments
///
/// * `subject_token` - LineToken for the verbatim block subject
/// * `content_tokens` - LineTokens for each content line
/// * `closing_annotation` - The closing annotation node
/// * `source` - Original source string
///
/// # Returns
///
/// A VerbatimBlock ContentItem
///
/// # Example
///
/// ```rust,ignore
/// // Top-level: "Code:\n    line1\n    line2\n:: js ::"
/// // Nested:    "Session:\n    Code:\n        line1\n        line2\n    :: js ::"
/// //
/// // Both produce VerbatimBlock with content: "line1\nline2"
/// // The indentation wall (minimum indentation) is stripped.
/// ```
pub fn build_verbatim_block(
    subject_token: &LineToken,
    content_tokens: Vec<&LineToken>,
    closing_annotation: Annotation,
    source: &str,
) -> ContentItem {
    // 1. Normalize subject
    let subject_tokens = normalization::normalize_line_token(subject_token);

    // 2. Normalize content (preserving line boundaries for wall calculation)
    let content_token_lines: Vec<Vec<_>> = content_tokens
        .iter()
        .map(|lt| normalization::normalize_line_token(lt))
        .collect();

    // 3. Extract (includes indentation wall stripping)
    let group = extraction::VerbatimGroupTokenLines {
        subject_tokens,
        content_token_lines,
    };
    let data = extraction::extract_verbatim_block_data(vec![group], source);

    // 4. Create
    builders::create_verbatim_block(data, closing_annotation, source)
}

// ============================================================================
// NORMALIZED TOKEN API (tokens already normalized)
// ============================================================================
//
// Some callers (e.g., the linebased parser's unwrappers) already work with
// normalized Vec<(Token, Range)> sequences. These helpers skip the
// normalization pass and go straight to data extraction/AST creation.

use crate::lex::lexing::tokens_core::Token;
use std::ops::Range as ByteRange;

/// Build a Paragraph from already-normalized token lines.
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
    mut token_lines: Vec<Vec<(Token, ByteRange<usize>)>>,
    source: &str,
) -> ContentItem {
    if token_lines.len() == 1 {
        let mut new_token_lines = vec![];
        let mut current_line = vec![];
        for token_data in token_lines.remove(0) {
            if let Token::BlankLine(_) = token_data.0 {
                if !current_line.is_empty() {
                    new_token_lines.push(current_line);
                    current_line = vec![];
                }
            } else {
                current_line.push(token_data);
            }
        }
        if !current_line.is_empty() {
            new_token_lines.push(current_line);
        }
        token_lines = new_token_lines;
    }

    // 1. Extract
    let data = extraction::extract_paragraph_data(token_lines, source);

    // 2. Create
    builders::create_paragraph(data, source)
}

/// Build a Session from already-normalized title tokens.
///
/// # Arguments
///
/// * `title_tokens` - Normalized tokens for the session title
/// * `content` - Typed child content items (already constructed)
/// * `source` - Original source string
///
/// # Returns
///
/// A Session ContentItem
pub fn build_session_from_tokens(
    title_tokens: Vec<(Token, ByteRange<usize>)>,
    content: Vec<SessionContent>,
    source: &str,
) -> ContentItem {
    // Skip normalization, tokens already normalized
    // 1. Extract
    let data = extraction::extract_session_data(title_tokens, source);

    // 2. Create
    builders::create_session(data, content, source)
}

/// Build a Definition from already-normalized subject tokens.
///
/// # Arguments
///
/// * `subject_tokens` - Normalized tokens for the definition subject
/// * `content` - Typed child content items (already constructed)
/// * `source` - Original source string
///
/// # Returns
///
/// A Definition ContentItem
pub fn build_definition_from_tokens(
    subject_tokens: Vec<(Token, ByteRange<usize>)>,
    content: Vec<ContentElement>,
    source: &str,
) -> ContentItem {
    // Skip normalization, tokens already normalized
    // 1. Extract
    let data = extraction::extract_definition_data(subject_tokens, source);

    // 2. Create
    builders::create_definition(data, content, source)
}

/// Build a ListItem from already-normalized marker tokens.
///
/// # Arguments
///
/// * `marker_tokens` - Normalized tokens for the list item marker and text
/// * `content` - Typed child content items (already constructed)
/// * `source` - Original source string
///
/// # Returns
///
/// A ListItem node (not wrapped in ContentItem)
pub fn build_list_item_from_tokens(
    marker_tokens: Vec<(Token, ByteRange<usize>)>,
    content: Vec<ContentElement>,
    source: &str,
) -> ListItem {
    // Skip normalization, tokens already normalized
    // 1. Extract
    let data = extraction::extract_list_item_data(marker_tokens, source);

    // 2. Create
    builders::create_list_item(data, content, source)
}

/// Build an Annotation from already-normalized label tokens.
///
/// Skips normalization, goes through: extract (with label/param parsing) → create.
///
/// # Arguments
///
/// * `label_tokens` - Normalized tokens for the annotation label (includes label and parameters)
/// * `content` - Typed child content items (already constructed)
/// * `source` - Original source string
///
/// # Returns
///
/// An Annotation ContentItem
pub fn build_annotation_from_tokens(
    label_tokens: Vec<(Token, ByteRange<usize>)>,
    content: Vec<ContentElement>,
    source: &str,
) -> ContentItem {
    // Skip normalization, tokens already normalized
    // 1. Extract (parses label AND parameters from tokens)
    let data = extraction::extract_annotation_data(label_tokens, source);

    // 2. Create
    builders::create_annotation(data, content, source)
}

/// Build a VerbatimBlock from already-normalized tokens.
///
/// This implements indentation wall stripping - content at different nesting
/// levels will have identical text after wall removal.
///
/// # Arguments
///
/// * `subject_tokens` - Normalized tokens for the verbatim block subject
/// * `content_token_lines` - Normalized token vectors for each content line
/// * `closing_annotation` - The closing annotation node
/// * `source` - Original source string
///
/// # Returns
///
/// A VerbatimBlock ContentItem
///
/// # Example
///
/// ```rust,ignore
/// // Tokens already normalized upstream
/// let group = extraction::VerbatimGroupTokenLines {
///     subject_tokens: vec![(Token::Text("Code".into()), 0..4)],
///     content_token_lines: vec![
///         vec![(Token::Indentation, 6..10), (Token::Text("line1".into()), 10..15)],
///     ],
/// };
/// // After extraction, wall of 1 indent is stripped: "line1\nline2"
/// ```
pub fn build_verbatim_block_from_tokens(
    groups: Vec<extraction::VerbatimGroupTokenLines>,
    closing_annotation: Annotation,
    source: &str,
) -> ContentItem {
    let data = extraction::extract_verbatim_block_data(groups, source);
    builders::create_verbatim_block(data, closing_annotation, source)
}

// ============================================================================
// TEXT-BASED API (for pre-extracted inputs)
// ============================================================================
//
// These functions accept pre-extracted text and ast::Range locations for tests
// or any parser variant that wants to bypass token processing entirely.

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

/// Build a VerbatimBlock from pre-extracted text and locations.
///
/// NOTE: This does NOT perform indentation wall stripping.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::ast::elements::typed_content::SessionContent;
    use crate::lex::lexing::tokens_core::Token;
    use crate::lex::lexing::tokens_linebased::LineType;

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

        let result = build_session(&title_token, Vec::<SessionContent>::new(), source);

        match result {
            ContentItem::Session(session) => {
                assert_eq!(session.title.as_string(), "Session:");
            }
            _ => panic!("Expected Session"),
        }
    }
}
