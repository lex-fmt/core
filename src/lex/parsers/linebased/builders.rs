//! Linebased Parser Unwrapper - Pattern to AST Conversion
//!
//! This module handles converting matched patterns and tokens into AST nodes.
//! It uses the AST construction facade from common::ast_construction which properly
//! handles token unrolling, location conversion, and AST construction.
//!
//! The unwrapper is responsible for:
//! 1. Taking matched pattern data + tokens
//! 2. Calling facade functions with tokens + source string
//! 3. Facade handles: unrolling, location conversion, calling base builders
//! 4. Handling recursive content from nested blocks

use crate::lex::ast::range::SourceLocation;
use crate::lex::ast::Range;
use crate::lex::lexers::LineToken;
use crate::lex::parsers::common::ast_construction;
use crate::lex::parsers::common::{
    build_annotation, build_paragraph, extract_text_from_span,
    location::{compute_location_from_locations, default_location},
};
use crate::lex::parsers::ContentItem;

// ============================================================================
// TEXT AND LOCATION EXTRACTION
// ============================================================================

/// Extract text from a LineToken by computing bounding box from token_spans.
fn extract_text_from_line_token(token: &LineToken, source: &str) -> Result<String, String> {
    if token.token_spans.is_empty() {
        return Err("LineToken has no token_spans".to_string());
    }

    // Compute bounding box from token_spans
    let min_start = token.token_spans.iter().map(|r| r.start).min().unwrap();
    let max_end = token.token_spans.iter().map(|r| r.end).max().unwrap();
    let span = min_start..max_end;

    Ok(extract_text_from_span(source, &span))
}

/// Extract text from a subset of token slice using byte range extraction.
/// This is the unified approach - use the same byte-range logic as reference parser.
fn extract_text_from_token_slice(
    token: &LineToken,
    start_idx: usize,
    end_idx: usize,
    source: &str,
) -> Result<String, String> {
    if start_idx > token.token_spans.len() || end_idx > token.token_spans.len() {
        return Err(format!(
            "Token slice indices out of bounds: start_idx={}, end_idx={}, len={}",
            start_idx,
            end_idx,
            token.token_spans.len()
        ));
    }
    if start_idx > end_idx {
        return Err("Invalid token slice: start > end".to_string());
    }

    // Get the byte ranges for the slice
    let spans = &token.token_spans[start_idx..end_idx];
    if spans.is_empty() {
        return Ok(String::new());
    }

    let start = spans.first().map(|s| s.start).unwrap_or(0);
    let end = spans.last().map(|s| s.end).unwrap_or(0);

    if start >= end || end > source.len() {
        return Ok(String::new());
    }

    Ok(source[start..end].trim().to_string())
}

// Location utilities are now provided by crate::lex::parsers::common::location
// See that module for compute_location_from_locations, aggregate_locations, etc.

/// Convert a line token to a Paragraph ContentItem.
///
/// Uses the AST construction facade which handles token unrolling,
/// location conversion, and calls the base builder.
pub fn unwrap_token_to_paragraph(token: &LineToken, source: &str) -> Result<ContentItem, String> {
    // Use facade to build paragraph from single token
    let paragraph =
        ast_construction::build_paragraph_from_line_tokens(std::slice::from_ref(token), source);
    Ok(paragraph)
}

/// Convert multiple line tokens to a single Paragraph ContentItem with multiple lines.
///
/// Uses the AST construction facade which handles token unrolling,
/// location conversion, and calls the base builder.
pub fn unwrap_tokens_to_paragraph(
    tokens: Vec<LineToken>,
    source: &str,
) -> Result<ContentItem, String> {
    if tokens.is_empty() {
        return Err("Cannot create paragraph from empty token list".to_string());
    }

    // Use facade to build paragraph from multiple tokens
    let paragraph = ast_construction::build_paragraph_from_line_tokens(&tokens, source);
    Ok(paragraph)
}

/// Convert an annotation line token to an Annotation ContentItem.
///
/// Annotations are lines with :: markers, format: :: label [params] :: [optional text]
/// This builds an Annotation element from the source tokens, extracting:
/// - Label and parameters between :: markers
/// - Optional trailing text after closing :: as a child paragraph
pub fn unwrap_annotation(token: &LineToken, source: &str) -> Result<ContentItem, String> {
    use crate::lex::lexers::tokens::Token;

    // Extract location from the token
    let location = extract_location_from_token(token, source);

    // Find the structure: :: [tokens]* :: [tokens]*
    // We need to count how many :: markers we have
    let mut dcolon_count = 0;
    let mut first_dcolon_idx = None;
    let mut second_dcolon_start = None;

    // Scan through source tokens looking for :: markers
    let mut i = 0;
    while i < token.source_tokens.len() {
        if matches!(&token.source_tokens[i], Token::LexMarker) {
            // Token::LexMarker represents the :: marker
            dcolon_count += 1;
            if dcolon_count == 1 {
                first_dcolon_idx = Some(i);
            } else if dcolon_count == 2 {
                second_dcolon_start = Some(i);
                break;
            }
        }
        i += 1;
    }

    // Parse based on what we found
    if dcolon_count >= 2 {
        // We have :: label :: [text]
        // Extract label tokens between the two :: markers using byte-range extraction
        let first_dcolon = first_dcolon_idx.unwrap();
        let second_dcolon = second_dcolon_start.unwrap();

        let label_text =
            extract_text_from_token_slice(token, first_dcolon + 1, second_dcolon, source)?;

        // Extract text after second :: using byte-range extraction
        let trailing_text = extract_text_from_token_slice(
            token,
            second_dcolon + 1,
            token.source_tokens.len(),
            source,
        )?;

        // Build content with optional trailing text
        // Note: We use build_paragraph directly here because we've already extracted the text
        // and can't easily create a proper LineToken for the facade
        let content = if !trailing_text.is_empty() {
            vec![build_paragraph(
                vec![(trailing_text, location.clone())],
                location.clone(),
            )]
        } else {
            vec![]
        };

        // Use build_annotation directly because we've already parsed the label structure
        // The annotation facade expects unparsed LineToken, but we've done custom parsing
        let annotation = build_annotation(label_text, location, vec![], content);
        return Ok(annotation);
    }

    // Fallback: single-line annotation without trailing text
    // Use facade to create simple annotation
    let annotation =
        ast_construction::build_annotation_from_line_token(token, vec![], vec![], source);
    Ok(annotation)
}

/// Create an annotation with block content from an opening annotation token and parsed content
pub fn unwrap_annotation_with_content(
    opening_token: &LineToken,
    content: Vec<ContentItem>,
    source: &str,
) -> Result<ContentItem, String> {
    // Use facade to create annotation with content
    let annotation =
        ast_construction::build_annotation_from_line_token(opening_token, vec![], content, source);
    Ok(annotation)
}

/// Extract location information from a LineToken by computing bounding box from token_spans
///
/// This computes the bounding box from the token_spans to determine the exact
/// line and column positions in the source code. If no token spans are available,
/// returns a default location.
fn extract_location_from_token(token: &LineToken, source: &str) -> Range {
    if token.token_spans.is_empty() {
        return default_location();
    }

    // Compute bounding box from token_spans
    let min_start = token.token_spans.iter().map(|r| r.start).min().unwrap();
    let max_end = token.token_spans.iter().map(|r| r.end).max().unwrap();
    let span = min_start..max_end;

    let source_location = SourceLocation::new(source);
    source_location.byte_range_to_ast_range(&span)
}

/// Extract a combined location that spans multiple tokens
///
/// Uses Location-level aggregation (matching reference parser approach):
/// 1. Convert each token's byte range to a Location
/// 2. Collect all Locations
/// 3. Aggregate using Location-level min/max (line/column coordinates)
///
/// This approach is semantically correct for hierarchical/non-contiguous children
/// and avoids issues with assuming contiguous byte ranges.
fn extract_location_from_tokens(tokens: &[LineToken], source: &str) -> Range {
    if tokens.is_empty() {
        return default_location();
    }

    // Convert each token's bounding box to a Location
    let locations: Vec<Range> = tokens
        .iter()
        .filter_map(|token| {
            if token.token_spans.is_empty() {
                None
            } else {
                // Compute bounding box from token_spans
                let min_start = token.token_spans.iter().map(|r| r.start).min().unwrap();
                let max_end = token.token_spans.iter().map(|r| r.end).max().unwrap();
                let span = min_start..max_end;

                let source_location = SourceLocation::new(source);
                Some(source_location.byte_range_to_ast_range(&span))
            }
        })
        .collect();

    // Aggregate all locations using Location-level bounds
    if locations.is_empty() {
        default_location()
    } else {
        compute_location_from_locations(&locations)
    }
}

/// Create a Session AST node from a subject line token and content
///
/// Used by the parser when it matches: SUBJECT_LINE + BLANK_LINE + INDENT
///
/// Uses the AST construction facade which handles token unrolling,
/// location conversion, and calls the base builder.
pub fn unwrap_session(
    subject_token: &LineToken,
    content: Vec<ContentItem>,
    source: &str,
) -> Result<ContentItem, String> {
    // Use facade to build session
    let session = ast_construction::build_session_from_line_token(subject_token, content, source);
    Ok(session)
}

/// Create a Definition AST node from a subject token and content
///
/// Used by the parser when it matches: SUBJECT_LINE + INDENT (no blank line)
///
/// Uses the AST construction facade which handles token unrolling,
/// location conversion, and calls the base builder.
pub fn unwrap_definition(
    subject_token: &LineToken,
    content: Vec<ContentItem>,
    source: &str,
) -> Result<ContentItem, String> {
    // Use facade to build definition
    let definition =
        ast_construction::build_definition_from_line_token(subject_token, content, source);
    Ok(definition)
}

/// Create a List AST node from multiple list item tokens
///
/// Used by the parser when it matches: BLANK_LINE + 2+ list items
///
/// Uses the AST construction facade which handles location computation
/// from all child items.
pub fn unwrap_list(list_items: Vec<ContentItem>, source: &str) -> Result<ContentItem, String> {
    if list_items.is_empty() {
        return Err("Cannot create list with no items".to_string());
    }

    // Use facade to build list
    let list = ast_construction::build_list_from_items(list_items, source);
    Ok(list)
}

/// Create a ListItem AST node from a list line token and optional nested content
///
/// Called for each item in a list
///
/// Uses the AST construction facade which handles token unrolling,
/// location conversion, and calls the base builder.
pub fn unwrap_list_item(
    item_token: &LineToken,
    content: Vec<ContentItem>,
    source: &str,
) -> Result<ContentItem, String> {
    // Use facade to build list item
    let list_item = ast_construction::build_list_item_from_line_token(item_token, content, source);
    Ok(list_item)
}

/// Create a ForeignBlock AST node from subject, content, and closing annotation
///
/// Used by the parser when it matches: SUBJECT_LINE + INDENT...DEDENT + ANNOTATION_LINE
///
/// Uses the AST construction facade for building the foreign block and annotation.
/// The caller must combine content lines, as foreign blocks have special content handling.
pub fn unwrap_foreign_block(
    subject_token: &LineToken,
    content_lines: Vec<&LineToken>,
    closing_annotation_token: &LineToken,
    source: &str,
) -> Result<ContentItem, String> {
    // Combine all content lines into a single text block
    let mut content_text = String::new();
    for (idx, token) in content_lines.iter().enumerate() {
        if idx > 0 {
            content_text.push('\n');
        }
        content_text.push_str(&extract_text_from_line_token(token, source)?);
    }

    // Compute content location from all content lines
    let content_location = if content_lines.is_empty() {
        Range::default()
    } else {
        // Convert Vec<&LineToken> to Vec<LineToken> for the function
        let tokens: Vec<LineToken> = content_lines.iter().map(|t| (*t).clone()).collect();
        extract_location_from_tokens(&tokens, source)
    };

    // Use facade to build the closing annotation
    let closing_annotation = match ast_construction::build_annotation_from_line_token(
        closing_annotation_token,
        vec![],
        vec![],
        source,
    ) {
        ContentItem::Annotation(annotation) => annotation,
        _ => unreachable!("build_annotation_from_line_token always returns Annotation"),
    };

    // Use facade to create foreign block
    let foreign_block = ast_construction::build_foreign_block_from_line_token(
        subject_token,
        content_text,
        content_location,
        closing_annotation,
        source,
    );

    Ok(foreign_block)
}
