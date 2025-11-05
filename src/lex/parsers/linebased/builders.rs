//! Linebased Parser Unwrapper - Pattern to AST Conversion
//!
//! This module handles converting matched patterns and tokens into AST nodes.
//! It uses the new AST builder API which coordinates:
//! 1. Token normalization (various formats → standard vectors)
//! 2. Data extraction (tokens → primitives with byte ranges)
//! 3. AST creation (primitives → AST nodes with ast::Range)
//!
//! The unwrapper is responsible for:
//! 1. Taking matched pattern data + tokens from the parser
//! 2. Calling ast_builder public API with tokens + source string
//! 3. Handling recursive content from nested blocks

use crate::lex::ast::range::SourceLocation;
use crate::lex::ast::Range;
use crate::lex::lexers::LineToken;
use crate::lex::parsers::common::ast_builder;
use crate::lex::parsers::common::location::default_location;
use crate::lex::parsers::ContentItem;

// ============================================================================
// HELPER FUNCTIONS (for annotation parsing - temporary until annotations migrated)
// ============================================================================

/// Extract text from a subset of token slice using byte range extraction.
/// Used by annotation parsing which needs custom label extraction.
/// TODO: Remove when annotations are migrated to new API
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

// ============================================================================
// PARAGRAPH UNWRAPPERS
// ============================================================================

/// Convert a line token to a Paragraph ContentItem.
///
/// Uses the new AST builder API which coordinates normalization, extraction, and creation.
pub fn unwrap_token_to_paragraph(token: &LineToken, source: &str) -> Result<ContentItem, String> {
    Ok(ast_builder::build_paragraph(
        std::slice::from_ref(token),
        source,
    ))
}

/// Convert multiple line tokens to a single Paragraph ContentItem with multiple lines.
///
/// Uses the new AST builder API which coordinates normalization, extraction, and creation.
pub fn unwrap_tokens_to_paragraph(
    tokens: Vec<LineToken>,
    source: &str,
) -> Result<ContentItem, String> {
    if tokens.is_empty() {
        return Err("Cannot create paragraph from empty token list".to_string());
    }

    Ok(ast_builder::build_paragraph(&tokens, source))
}

/// Convert an annotation line token to an Annotation ContentItem.
///
/// Annotations are lines with :: markers, format: :: label [params] :: [optional text]
/// This builds an Annotation element from the source tokens, extracting:
/// - Label and parameters between :: markers
/// - Optional trailing text after closing :: as a child paragraph
pub fn unwrap_annotation(token: &LineToken, source: &str) -> Result<ContentItem, String> {
    use crate::lex::lexers::tokens_core::Token;

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
        // We have :: label [params] :: [text]
        // Extract tokens between the two :: markers
        let first_dcolon = first_dcolon_idx.unwrap();
        let second_dcolon = second_dcolon_start.unwrap();

        // Get all token pairs for the label (between :: markers)
        let label_token_pairs: Vec<(Token, std::ops::Range<usize>)> = token
            .source_tokens
            .iter()
            .zip(token.token_spans.iter())
            .skip(first_dcolon + 1)
            .take(second_dcolon - first_dcolon - 1)
            .map(|(t, s)| (t.clone(), s.clone()))
            .collect();

        // Extract trailing text after second ::
        let trailing_text = extract_text_from_token_slice(
            token,
            second_dcolon + 1,
            token.source_tokens.len(),
            source,
        )?;

        // Build content with optional trailing text
        let content = if !trailing_text.is_empty() {
            vec![ast_builder::build_paragraph_from_text(
                vec![(trailing_text, location.clone())],
                location.clone(),
            )]
        } else {
            vec![]
        };

        // Use token-based API which will parse label AND parameters from tokens
        let annotation =
            ast_builder::build_annotation_from_tokens(label_token_pairs, content, source);
        return Ok(annotation);
    }

    // Fallback: single-line annotation without trailing text
    // Use token-based API which will parse label AND parameters
    Ok(ast_builder::build_annotation(token, vec![], source))
}

/// Create an annotation with block content from an opening annotation token and parsed content
pub fn unwrap_annotation_with_content(
    opening_token: &LineToken,
    content: Vec<ContentItem>,
    source: &str,
) -> Result<ContentItem, String> {
    // Use token-based API which will parse label AND parameters from tokens
    Ok(ast_builder::build_annotation(
        opening_token,
        content,
        source,
    ))
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

/// Create a Session AST node from a subject line token and content
///
/// Used by the parser when it matches: SUBJECT_LINE + BLANK_LINE + INDENT
///
/// Uses the new AST builder API which coordinates normalization, extraction, and creation.
pub fn unwrap_session(
    subject_token: &LineToken,
    content: Vec<ContentItem>,
    source: &str,
) -> Result<ContentItem, String> {
    Ok(ast_builder::build_session(subject_token, content, source))
}

/// Create a Definition AST node from a subject token and content
///
/// Used by the parser when it matches: SUBJECT_LINE + INDENT (no blank line)
///
/// Uses the new AST builder API which coordinates normalization, extraction, and creation.
pub fn unwrap_definition(
    subject_token: &LineToken,
    content: Vec<ContentItem>,
    source: &str,
) -> Result<ContentItem, String> {
    Ok(ast_builder::build_definition(
        subject_token,
        content,
        source,
    ))
}

/// Create a List AST node from multiple list item tokens
///
/// Used by the parser when it matches: BLANK_LINE + 2+ list items
///
/// Uses the new AST builder API which handles location computation from all child items.
pub fn unwrap_list(list_items: Vec<ContentItem>, _source: &str) -> Result<ContentItem, String> {
    if list_items.is_empty() {
        return Err("Cannot create list with no items".to_string());
    }

    // Extract ListItems from ContentItems
    let items: Vec<_> = list_items
        .into_iter()
        .filter_map(|item| match item {
            ContentItem::ListItem(li) => Some(li),
            _ => None,
        })
        .collect();

    Ok(ast_builder::build_list(items))
}

/// Create a ListItem AST node from a list line token and optional nested content
///
/// Called for each item in a list
///
/// Uses the new AST builder API which coordinates normalization, extraction, and creation.
pub fn unwrap_list_item(
    item_token: &LineToken,
    content: Vec<ContentItem>,
    source: &str,
) -> Result<ContentItem, String> {
    let list_item = ast_builder::build_list_item(item_token, content, source);
    Ok(ContentItem::ListItem(list_item))
}

/// Create a ForeignBlock AST node from subject, content, and closing annotation
///
/// Used by the parser when it matches: SUBJECT_LINE + INDENT...DEDENT + ANNOTATION_LINE
///
/// Uses the new AST builder API which implements indentation wall stripping.
/// This ensures that foreign blocks at different nesting levels have identical content.
pub fn unwrap_foreign_block(
    subject_token: &LineToken,
    content_lines: Vec<&LineToken>,
    closing_annotation_token: &LineToken,
    source: &str,
) -> Result<ContentItem, String> {
    // Build the closing annotation using token-based API (parses label and parameters)
    let closing_annotation =
        match ast_builder::build_annotation(closing_annotation_token, vec![], source) {
            ContentItem::Annotation(annotation) => annotation,
            _ => unreachable!("build_annotation always returns Annotation"),
        };

    // Use new API to build foreign block with indentation wall stripping
    Ok(ast_builder::build_foreign_block(
        subject_token,
        content_lines,
        closing_annotation,
        source,
    ))
}
