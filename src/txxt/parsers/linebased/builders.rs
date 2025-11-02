//! Linebased Parser Unwrapper - Pattern to AST Conversion
//!
//! This module handles converting matched patterns and tokens into AST nodes.
//! It reuses existing element builders from src/txxt/parser/elements/ which properly
//! handle location tracking and AST construction.
//!
//! The unwrapper is responsible for:
//! 1. Taking matched pattern data + tokens
//! 2. Extracting source locations from tokens via source_tokens field
//! 3. Building appropriate AST node types using existing builders
//! 4. Handling recursive content from nested blocks

use crate::txxt::ast::location::SourceLocation;
use crate::txxt::ast::traits::AstNode;
use crate::txxt::ast::{
    Annotation, Definition, Label, List, ListItem, Paragraph, Session, TextContent, TextLine,
};
use crate::txxt::lexers::{LineToken, Token};
use crate::txxt::parsers::common::location::{
    aggregate_locations, compute_location_from_locations, default_location,
};
use crate::txxt::parsers::{ContentItem, Location};

// ============================================================================
// LOCATION UTILITIES
// ============================================================================
//
// Location utilities are now provided by crate::txxt::parsers::common::location
// See that module for compute_location_from_locations, aggregate_locations, etc.

/// Stub: Convert a line token to a Paragraph ContentItem.
///
/// This is a temporary implementation that treats any token as paragraph text.
/// Later, this will be enhanced with pattern matching to recognize
/// Sessions, Definitions, Lists, etc.
pub fn unwrap_token_to_paragraph(token: &LineToken, source: &str) -> Result<ContentItem, String> {
    // Extract text from the token
    let text_content = extract_text_from_token(token);

    // Extract location from source span
    let location = extract_location_from_token(token, source);

    // Create a TextLine from the text
    let text_line = TextLine {
        content: TextContent::from_string(text_content, None),
        location,
    };

    // Wrap in a Paragraph
    let paragraph = Paragraph {
        lines: vec![ContentItem::TextLine(text_line)],
        location,
    };

    Ok(ContentItem::Paragraph(paragraph))
}

/// Convert multiple line tokens to a single Paragraph ContentItem with multiple lines.
///
/// This handles multi-line paragraphs where consecutive lines are grouped into
/// a single paragraph node, each line becoming a TextLine in the paragraph.
pub fn unwrap_tokens_to_paragraph(
    tokens: Vec<LineToken>,
    source: &str,
) -> Result<ContentItem, String> {
    if tokens.is_empty() {
        return Err("Cannot create paragraph from empty token list".to_string());
    }

    // Extract combined location spanning all tokens
    let location = extract_location_from_tokens(&tokens, source);

    // Create a TextLine for each token with its own location
    let lines: Vec<ContentItem> = tokens
        .into_iter()
        .map(|token| {
            let text_content = extract_text_from_token(&token);
            let line_location = extract_location_from_token(&token, source);
            ContentItem::TextLine(TextLine {
                content: TextContent::from_string(text_content, None),
                location: line_location,
            })
        })
        .collect();

    // Wrap all lines in a single Paragraph with combined location
    let paragraph = Paragraph { lines, location };

    Ok(ContentItem::Paragraph(paragraph))
}

/// Convert an annotation line token to an Annotation ContentItem.
///
/// Annotations are lines with :: markers, format: :: label [params] :: [optional text]
/// This builds an Annotation element from the source tokens, extracting:
/// - Label and parameters between :: markers
/// - Optional trailing text after closing :: as a child paragraph
pub fn unwrap_annotation(token: &LineToken, source: &str) -> Result<ContentItem, String> {
    use crate::txxt::lexers::tokens::Token;

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
        if matches!(&token.source_tokens[i], Token::TxxtMarker) {
            // Token::TxxtMarker represents the :: marker
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
        // Extract label tokens between the two :: markers
        let first_dcolon = first_dcolon_idx.unwrap();
        let second_dcolon = second_dcolon_start.unwrap();

        let label_tokens = &token.source_tokens[first_dcolon + 1..second_dcolon];
        let label_text = extract_text_from_tokens(label_tokens);

        // Extract text after second ::
        let remaining_tokens = &token.source_tokens[second_dcolon + 1..];
        let trailing_text = extract_text_from_tokens(remaining_tokens);

        // Create annotation with proper location
        let mut annotation = Annotation {
            label: Label::from_string(&label_text),
            parameters: vec![],
            content: vec![],
            location,
        };

        // If there's trailing text, create a paragraph as content
        if !trailing_text.is_empty() {
            let text_line = TextLine {
                content: TextContent::from_string(trailing_text, None),
                location,
            };
            let paragraph = Paragraph {
                lines: vec![ContentItem::TextLine(text_line)],
                location,
            };
            annotation.content.push(ContentItem::Paragraph(paragraph));
        }

        return Ok(ContentItem::Annotation(annotation));
    }

    // Fallback: single-line annotation without trailing text
    let text_content = extract_text_from_token(token);

    // Create an annotation with the extracted text and proper location
    let annotation = Annotation {
        label: Label::from_string(&text_content),
        parameters: vec![],
        content: vec![],
        location,
    };

    Ok(ContentItem::Annotation(annotation))
}

/// Create an annotation with block content from an opening annotation token and parsed content
pub fn unwrap_annotation_with_content(
    opening_token: &LineToken,
    content: Vec<ContentItem>,
    source: &str,
) -> Result<ContentItem, String> {
    // Extract text content from the opening annotation
    let text_content = extract_text_from_token(opening_token);

    // Extract location from the opening token
    let location = extract_location_from_token(opening_token, source);

    // Create an annotation with the extracted text and content
    let annotation = Annotation {
        label: Label::from_string(&text_content),
        parameters: vec![],
        content,
        location,
    };

    Ok(ContentItem::Annotation(annotation))
}

/// Extract human-readable text from a line token's source tokens.
///
/// Extracts semantic content from all token types (Text, Number, Dash, etc.)
/// while skipping whitespace, newlines, and synthetic indentation tokens.
/// This provides proper text reconstruction for annotations, definitions, and other
/// semantic structures that may contain non-Text tokens (e.g., numbers in ordered lists).
fn extract_text_from_token(token: &LineToken) -> String {
    extract_text_from_tokens(&token.source_tokens)
}

/// Extract text from a slice of tokens, properly handling all token types.
///
/// Extracts semantic content from all tokens (Text, Number, Dash, Period, etc.)
/// while skipping whitespace, newlines, and synthetic indentation tokens.
/// Concatenates tokens directly, preserving the original token structure without
/// forcing spaces between everything (unlike simple join which always adds spaces).
fn extract_text_from_tokens(tokens: &[Token]) -> String {
    let mut result = String::new();
    let mut prev_was_content = false;

    for token in tokens {
        match token {
            // Semantic content tokens - extract their string representation
            Token::Text(s) => {
                // Add space before text if previous token was content
                if prev_was_content {
                    result.push(' ');
                }
                result.push_str(s);
                prev_was_content = true;
            }
            Token::Number(s) => {
                // Add space before number if previous token was content
                if prev_was_content {
                    result.push(' ');
                }
                result.push_str(s);
                prev_was_content = true;
            }
            // Punctuation and symbols - no spaces around them
            Token::Dash => {
                result.push('-');
                prev_was_content = true;
            }
            Token::Period => {
                result.push('.');
                prev_was_content = true;
            }
            Token::OpenParen => {
                result.push('(');
                prev_was_content = true;
            }
            Token::CloseParen => {
                result.push(')');
                prev_was_content = false; // Reset for next token
            }
            Token::Colon => {
                result.push(':');
                prev_was_content = true;
            }
            Token::Comma => {
                result.push(',');
                prev_was_content = true;
            }
            Token::Quote => {
                result.push('"');
                prev_was_content = true;
            }
            Token::Equals => {
                result.push('=');
                prev_was_content = true;
            }
            Token::TxxtMarker => {
                result.push_str("::");
                prev_was_content = true;
            }

            // Whitespace and newlines - skip these
            Token::Whitespace | Token::Newline | Token::BlankLine | Token::Indent => {
                // Skip whitespace
            }

            // Synthetic tokens - skip (generated during transformation)
            Token::IndentLevel | Token::DedentLevel => {
                // Skip synthetic tokens
            }
        }
    }

    result
}

/// Extract location information from a LineToken using its source span
///
/// This uses the source_span stored in the LineToken to determine the exact
/// line and column positions in the source code. If no source span is available,
/// returns a default location.
fn extract_location_from_token(token: &LineToken, source: &str) -> Location {
    match &token.source_span {
        Some(span) => {
            let source_location = SourceLocation::new(source);
            source_location.range_to_location(span)
        }
        None => default_location(),
    }
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
fn extract_location_from_tokens(tokens: &[LineToken], source: &str) -> Location {
    if tokens.is_empty() {
        return default_location();
    }

    // Convert each token's span to a Location
    let locations: Vec<Location> = tokens
        .iter()
        .filter_map(|token| {
            token.source_span.as_ref().map(|span| {
                let source_location = SourceLocation::new(source);
                source_location.range_to_location(span)
            })
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
/// Location is aggregated from the subject title and all child content,
/// matching the reference parser's hierarchical location approach.
pub fn unwrap_session(
    subject_token: &LineToken,
    content: Vec<ContentItem>,
    source: &str,
) -> Result<ContentItem, String> {
    let title_text = extract_text_from_token(subject_token);
    let title = TextContent::from_string(title_text, None);

    // Extract location from the subject token
    let title_location = extract_location_from_token(subject_token, source);

    // Aggregate title location with all child content locations
    let location = aggregate_locations(title_location, &content);

    let session = Session::new(title, content).at(location);
    Ok(ContentItem::Session(session))
}

/// Create a Definition AST node from a subject token and content
///
/// Used by the parser when it matches: SUBJECT_LINE + INDENT (no blank line)
///
/// Location is aggregated from the subject title and all child content,
/// matching the reference parser's hierarchical location approach.
pub fn unwrap_definition(
    subject_token: &LineToken,
    content: Vec<ContentItem>,
    source: &str,
) -> Result<ContentItem, String> {
    let subject_text = extract_text_from_token(subject_token);
    let subject = TextContent::from_string(subject_text, None);

    // Extract location from the subject token
    let subject_location = extract_location_from_token(subject_token, source);

    // Aggregate subject location with all child content locations
    let location = aggregate_locations(subject_location, &content);

    let definition = Definition::new(subject, content).at(location);
    Ok(ContentItem::Definition(definition))
}

/// Create a List AST node from multiple list item tokens
///
/// Used by the parser when it matches: BLANK_LINE + 2+ list items
///
/// Location is computed from all child list item locations,
/// matching the reference parser's hierarchical location approach.
pub fn unwrap_list(list_items: Vec<ContentItem>, _source: &str) -> Result<ContentItem, String> {
    if list_items.is_empty() {
        return Err("Cannot create list with no items".to_string());
    }

    // Aggregate location from all child list items
    let locations: Vec<Location> = list_items.iter().map(|item| item.location()).collect();

    let location = if locations.is_empty() {
        default_location()
    } else {
        compute_location_from_locations(&locations)
    };

    let list = List::new(list_items).at(location);
    Ok(ContentItem::List(list))
}

/// Create a ListItem AST node from a list line token and optional nested content
///
/// Called for each item in a list
///
/// Location is aggregated from the item text and any nested content,
/// matching the reference parser's hierarchical location approach.
pub fn unwrap_list_item(
    item_token: &LineToken,
    content: Vec<ContentItem>,
    source: &str,
) -> Result<ContentItem, String> {
    let item_text = extract_text_from_token(item_token);

    // Extract location from the item token
    let item_location = extract_location_from_token(item_token, source);

    // Aggregate with child content locations if present
    let location = if content.is_empty() {
        item_location
    } else {
        aggregate_locations(item_location, &content)
    };

    let list_item = if content.is_empty() {
        ListItem::new(item_text).at(location)
    } else {
        let text_content = TextContent::from_string(item_text, None);
        ListItem::with_text_content(text_content, content).at(location)
    };

    Ok(ContentItem::ListItem(list_item))
}

/// Create a ForeignBlock AST node from subject, content, and closing annotation
///
/// Used by the parser when it matches: SUBJECT_LINE + INDENT...DEDENT + ANNOTATION_LINE
///
/// Location is aggregated from subject, content lines, and closing annotation,
/// matching the reference parser's hierarchical location approach.
pub fn unwrap_foreign_block(
    subject_token: &LineToken,
    content_lines: Vec<&LineToken>,
    closing_annotation_token: &LineToken,
    source: &str,
) -> Result<ContentItem, String> {
    let subject_text = extract_text_from_token(subject_token);

    // Combine all content lines into a single text block
    let content_text = content_lines
        .iter()
        .map(|token| extract_text_from_token(token))
        .collect::<Vec<_>>()
        .join("\n");

    // Create the closing annotation with proper location
    let annotation_text = extract_text_from_token(closing_annotation_token);
    let annotation_location = extract_location_from_token(closing_annotation_token, source);
    let closing_annotation = Annotation {
        label: Label::from_string(&annotation_text),
        parameters: vec![],
        content: vec![],
        location: annotation_location,
    };

    // Aggregate location from subject, content lines, and closing annotation
    let subject_location = extract_location_from_token(subject_token, source);

    // Convert content_lines from Vec<&LineToken> to locations
    let content_locations: Vec<Location> = content_lines
        .iter()
        .filter_map(|token| {
            token.source_span.as_ref().map(|span| {
                let source_location = SourceLocation::new(source);
                source_location.range_to_location(span)
            })
        })
        .collect();

    let mut locations = vec![subject_location, annotation_location];
    locations.extend(content_locations);
    let combined_location = compute_location_from_locations(&locations);

    let foreign_block =
        crate::txxt::ast::ForeignBlock::new(subject_text, content_text, closing_annotation)
            .at(combined_location);

    Ok(ContentItem::ForeignBlock(foreign_block))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::txxt::lexers::{LineTokenType, Token};
    use crate::txxt::parsers::Position;

    fn make_line_token(line_type: LineTokenType, tokens: Vec<Token>) -> LineToken {
        LineToken {
            source_tokens: tokens,
            line_type,
            source_span: None,
        }
    }

    #[test]
    fn test_unwrap_simple_paragraph_token() {
        let token = make_line_token(
            LineTokenType::ParagraphLine,
            vec![Token::Text("Hello world".to_string())],
        );

        let result = unwrap_token_to_paragraph(&token, "Hello world\n");
        assert!(result.is_ok());

        let item = result.unwrap();
        assert!(matches!(item, ContentItem::Paragraph(_)));

        if let ContentItem::Paragraph(para) = item {
            assert_eq!(para.lines.len(), 1);
        }
    }

    #[test]
    fn test_unwrap_multiple_text_tokens() {
        let token = make_line_token(
            LineTokenType::ParagraphLine,
            vec![
                Token::Text("Hello".to_string()),
                Token::Whitespace,
                Token::Text("world".to_string()),
            ],
        );

        let result = unwrap_token_to_paragraph(&token, "Hello world\n");
        assert!(result.is_ok());

        let item = result.unwrap();
        if let ContentItem::Paragraph(para) = item {
            assert_eq!(para.lines.len(), 1);
            if let ContentItem::TextLine(line) = &para.lines[0] {
                // Text should be extracted from tokens
                assert!(!line.content.as_string().is_empty());
            }
        }
    }

    #[test]
    fn test_unwrap_subject_line_token() {
        let token = make_line_token(
            LineTokenType::SubjectLine,
            vec![Token::Text("Title".to_string()), Token::Colon],
        );

        let result = unwrap_token_to_paragraph(&token, "Title:\n");
        assert!(result.is_ok());

        // For now, subjects are treated as paragraphs
        let item = result.unwrap();
        assert!(matches!(item, ContentItem::Paragraph(_)));
    }

    #[test]
    fn test_unwrap_list_line_token() {
        let token = make_line_token(
            LineTokenType::ListLine,
            vec![
                Token::Dash,
                Token::Whitespace,
                Token::Text("Item".to_string()),
            ],
        );

        let result = unwrap_token_to_paragraph(&token, "- Item\n");
        assert!(result.is_ok());

        // For now, list items are treated as paragraphs
        let item = result.unwrap();
        assert!(matches!(item, ContentItem::Paragraph(_)));
    }

    #[test]
    fn test_unwrap_blank_line_token() {
        let token = make_line_token(LineTokenType::BlankLine, vec![Token::Newline]);

        let result = unwrap_token_to_paragraph(&token, "\n");
        assert!(result.is_ok());

        let item = result.unwrap();
        assert!(matches!(item, ContentItem::Paragraph(_)));
    }

    #[test]
    fn test_extract_text_with_single_token() {
        let token = make_line_token(
            LineTokenType::ParagraphLine,
            vec![Token::Text("Single".to_string())],
        );

        let text = extract_text_from_token(&token);
        assert_eq!(text, "Single");
    }

    #[test]
    fn test_extract_text_handles_all_token_types() {
        let token = make_line_token(
            LineTokenType::SubjectLine,
            vec![
                Token::Text("Title".to_string()),
                Token::Colon,
                Token::Newline,
            ],
        );

        let text = extract_text_from_token(&token);
        // Now properly handles all semantic content tokens, including Colon
        // Punctuation is directly concatenated without spaces
        // Newline is still filtered out as it's whitespace
        assert_eq!(text, "Title:");
    }

    #[test]
    fn test_extract_text_multiple_text_tokens() {
        let token = make_line_token(
            LineTokenType::ParagraphLine,
            vec![
                Token::Text("Hello".to_string()),
                Token::Whitespace,
                Token::Text("world".to_string()),
            ],
        );

        let text = extract_text_from_token(&token);
        // Should join text tokens with space
        assert!(text.contains("Hello"));
        assert!(text.contains("world"));
    }

    #[test]
    fn test_extract_text_empty_token() {
        let token = make_line_token(LineTokenType::BlankLine, vec![]);

        let text = extract_text_from_token(&token);
        assert_eq!(text, "");
    }

    // ========== LOCATION PRESERVATION TESTS ==========
    // These tests verify that location information flows from source spans
    // through the unwrapper functions into the AST nodes

    #[test]
    fn test_unwrap_paragraph_with_span_extracts_location() {
        let mut token = make_line_token(
            LineTokenType::ParagraphLine,
            vec![Token::Text("Hello world".to_string())],
        );
        // Simulate source span: characters 0-11 in source
        token.source_span = Some(0..11);

        let source = "Hello world\n";
        let result = unwrap_token_to_paragraph(&token, source);
        assert!(result.is_ok());

        if let Ok(ContentItem::Paragraph(para)) = result {
            // When span is present, location should be extracted from it
            // The location should span the text (columns 0-11 on line 0)
            assert_eq!(para.location.start.line, 0);
            assert_eq!(para.location.start.column, 0);
            assert_eq!(para.location.end.line, 0);
            assert_eq!(para.location.end.column, 11);
        } else {
            panic!("Expected Paragraph");
        }
    }

    #[test]
    fn test_unwrap_annotation_with_span_extracts_location() {
        let mut token = make_line_token(
            LineTokenType::AnnotationStartLine,
            vec![
                Token::TxxtMarker,
                Token::Whitespace,
                Token::Text("note".to_string()),
                Token::Whitespace,
                Token::TxxtMarker,
            ],
        );
        // Simulate source span: ":: note ::" at bytes 0-10
        token.source_span = Some(0..10);

        let source = ":: note ::\nSome text\n";
        let result = unwrap_annotation(&token, source);
        assert!(result.is_ok());

        if let Ok(ContentItem::Annotation(anno)) = result {
            // When span is present, location should be extracted
            // Should span the ":: note ::" on line 0
            assert_eq!(anno.location.start.line, 0);
            assert_eq!(anno.location.start.column, 0);
            assert_eq!(anno.location.end.line, 0);
            assert_eq!(anno.location.end.column, 10);
        } else {
            panic!("Expected Annotation");
        }
    }

    #[test]
    fn test_unwrap_session_with_span_extracts_location() {
        let mut token = make_line_token(
            LineTokenType::SubjectLine,
            vec![Token::Text("Session Title".to_string()), Token::Colon],
        );
        // Simulate source span: "Session Title:" at bytes 0-14
        token.source_span = Some(0..14);

        let source = "Session Title:\n    Nested content\n";
        let result = unwrap_session(&token, vec![], source);
        assert!(result.is_ok());

        if let Ok(ContentItem::Session(session)) = result {
            // When span is present, location should be extracted
            // Should span the title line
            assert_eq!(session.location.start.line, 0);
            assert_eq!(session.location.start.column, 0);
            assert_eq!(session.location.end.line, 0);
            assert_eq!(session.location.end.column, 14);
        } else {
            panic!("Expected Session");
        }
    }

    #[test]
    fn test_unwrap_definition_with_span_extracts_location() {
        let mut token = make_line_token(
            LineTokenType::SubjectLine,
            vec![Token::Text("Term".to_string()), Token::Colon],
        );
        // Simulate source span: "Term:" at bytes 0-5
        token.source_span = Some(0..5);

        let source = "Term:\n    Definition content\n";
        let result = unwrap_definition(&token, vec![], source);
        assert!(result.is_ok());

        if let Ok(ContentItem::Definition(def)) = result {
            // When span is present, location should be extracted
            // Should span "Term:" on line 0
            assert_eq!(def.location.start.line, 0);
            assert_eq!(def.location.start.column, 0);
            assert_eq!(def.location.end.line, 0);
            assert_eq!(def.location.end.column, 5);
        } else {
            panic!("Expected Definition");
        }
    }

    #[test]
    fn test_unwrap_list_item_with_span_extracts_location() {
        let mut token = make_line_token(
            LineTokenType::ListLine,
            vec![
                Token::Dash,
                Token::Whitespace,
                Token::Text("Item content".to_string()),
            ],
        );
        // Simulate source span: "- Item content" at bytes 0-14
        token.source_span = Some(0..14);

        let source = "- Item content\n";
        let result = unwrap_list_item(&token, vec![], source);
        assert!(result.is_ok());

        if let Ok(ContentItem::ListItem(item)) = result {
            // When span is present, location should be extracted
            // Should span the list item
            assert_eq!(item.location.start.line, 0);
            assert_eq!(item.location.start.column, 0);
            assert_eq!(item.location.end.line, 0);
            assert_eq!(item.location.end.column, 14);
        } else {
            panic!("Expected ListItem");
        }
    }

    #[test]
    fn test_multiline_paragraph_with_spans_extracts_combined_location() {
        let mut token1 = make_line_token(
            LineTokenType::ParagraphLine,
            vec![Token::Text("Line 1".to_string())],
        );
        token1.source_span = Some(0..6);

        let mut token2 = make_line_token(
            LineTokenType::ParagraphLine,
            vec![Token::Text("Line 2".to_string())],
        );
        token2.source_span = Some(7..13);

        let source = "Line 1\nLine 2\n";
        let result = unwrap_tokens_to_paragraph(vec![token1, token2], source);
        assert!(result.is_ok());

        if let Ok(ContentItem::Paragraph(para)) = result {
            // The combined location should span from start of first to end of second
            assert_eq!(para.location.start.line, 0);
            assert_eq!(para.location.start.column, 0);
            assert_eq!(para.location.end.line, 1);
            assert_eq!(para.location.end.column, 6);
        } else {
            panic!("Expected Paragraph");
        }
    }

    #[test]
    fn test_location_without_span_uses_default() {
        // If no source_span is set, location should be default (0,0)..(0,0)
        let token = make_line_token(
            LineTokenType::ParagraphLine,
            vec![Token::Text("Text".to_string())],
        );
        // Note: source_span is None by default

        let source = "Text\n";
        let result = unwrap_token_to_paragraph(&token, source);
        assert!(result.is_ok());

        if let Ok(ContentItem::Paragraph(para)) = result {
            // Without a source span, location should be default
            assert_eq!(para.location.start, Position { line: 0, column: 0 });
            assert_eq!(para.location.end, Position { line: 0, column: 0 });
        } else {
            panic!("Expected Paragraph");
        }
    }

    // ========== HIERARCHICAL LOCATION AGGREGATION TESTS ==========
    // These tests verify that container nodes properly aggregate locations
    // from their header/subject and all child content items.

    #[test]
    fn test_unwrap_session_aggregates_header_and_child_locations() {
        // Session title on line 0, columns 0-14
        let mut subject_token = make_line_token(
            LineTokenType::SubjectLine,
            vec![Token::Text("Session Title".to_string()), Token::Colon],
        );
        subject_token.source_span = Some(0..14);

        // Create mock child content with known locations:
        // Child 1: line 1, columns 4-20 (nested paragraph)
        let child1_location = Location::new(Position::new(1, 4), Position::new(1, 20));
        let child1 = ContentItem::Paragraph(Paragraph {
            lines: vec![],
            location: child1_location,
        });

        // Child 2: line 2, columns 4-25 (another nested paragraph)
        let child2_location = Location::new(Position::new(2, 4), Position::new(2, 25));
        let child2 = ContentItem::Paragraph(Paragraph {
            lines: vec![],
            location: child2_location,
        });

        let source = "Session Title:\n    First line\n    Second line\n";
        let result = unwrap_session(&subject_token, vec![child1, child2], source);
        assert!(result.is_ok());

        if let Ok(ContentItem::Session(session)) = result {
            // Location should be bounding box: from start of header (0:0) to end of last child (2:25)
            assert_eq!(session.location.start.line, 0);
            assert_eq!(session.location.start.column, 0);
            assert_eq!(session.location.end.line, 2);
            assert_eq!(session.location.end.column, 25);
        } else {
            panic!("Expected Session");
        }
    }

    #[test]
    fn test_unwrap_definition_aggregates_subject_and_child_locations() {
        // Definition subject on line 0, columns 0-5
        let mut subject_token = make_line_token(
            LineTokenType::SubjectLine,
            vec![Token::Text("Term".to_string()), Token::Colon],
        );
        subject_token.source_span = Some(0..5);

        // Create mock child content with known locations:
        // Child 1: line 1, columns 4-18
        let child1_location = Location::new(Position::new(1, 4), Position::new(1, 18));
        let child1 = ContentItem::Paragraph(Paragraph {
            lines: vec![],
            location: child1_location,
        });

        // Child 2: line 2, columns 4-22
        let child2_location = Location::new(Position::new(2, 4), Position::new(2, 22));
        let child2 = ContentItem::Paragraph(Paragraph {
            lines: vec![],
            location: child2_location,
        });

        let source = "Term:\n    Definition part 1\n    Definition part 2\n";
        let result = unwrap_definition(&subject_token, vec![child1, child2], source);
        assert!(result.is_ok());

        if let Ok(ContentItem::Definition(definition)) = result {
            // Location should be bounding box: from start of subject (0:0) to end of last child (2:22)
            assert_eq!(definition.location.start.line, 0);
            assert_eq!(definition.location.start.column, 0);
            assert_eq!(definition.location.end.line, 2);
            assert_eq!(definition.location.end.column, 22);
        } else {
            panic!("Expected Definition");
        }
    }

    #[test]
    fn test_unwrap_list_item_aggregates_item_and_nested_content() {
        // List item on line 0, columns 0-14
        let mut item_token = make_line_token(
            LineTokenType::ListLine,
            vec![
                Token::Dash,
                Token::Whitespace,
                Token::Text("Item text".to_string()),
            ],
        );
        item_token.source_span = Some(0..14);

        // Create mock nested content:
        // Nested child: line 1, columns 4-30
        let nested_location = Location::new(Position::new(1, 4), Position::new(1, 30));
        let nested_content = ContentItem::Paragraph(Paragraph {
            lines: vec![],
            location: nested_location,
        });

        let source = "- Item text\n    Nested content here\n";
        let result = unwrap_list_item(&item_token, vec![nested_content], source);
        assert!(result.is_ok());

        if let Ok(ContentItem::ListItem(item)) = result {
            // Location should be bounding box: from start of item (0:0) to end of nested content (1:30)
            assert_eq!(item.location.start.line, 0);
            assert_eq!(item.location.start.column, 0);
            assert_eq!(item.location.end.line, 1);
            assert_eq!(item.location.end.column, 30);
        } else {
            panic!("Expected ListItem");
        }
    }

    #[test]
    fn test_unwrap_list_aggregates_all_item_locations() {
        // Create mock list items with different locations

        // Item 1: line 0, columns 0-8 ("- Item 1" = 8 chars)
        let item1_location = Location::new(Position::new(0, 0), Position::new(0, 8));
        let item1 = ContentItem::ListItem(ListItem {
            text: vec![TextContent::from_string("Item 1".to_string(), None)],
            content: vec![],
            location: item1_location,
        });

        // Item 2: line 1, columns 0-8 ("- Item 2" = 8 chars)
        let item2_location = Location::new(Position::new(1, 0), Position::new(1, 8));
        let item2 = ContentItem::ListItem(ListItem {
            text: vec![TextContent::from_string("Item 2".to_string(), None)],
            content: vec![],
            location: item2_location,
        });

        // Item 3: line 2, columns 0-8 ("- Item 3" = 8 chars)
        let item3_location = Location::new(Position::new(2, 0), Position::new(2, 8));
        let item3 = ContentItem::ListItem(ListItem {
            text: vec![TextContent::from_string("Item 3".to_string(), None)],
            content: vec![],
            location: item3_location,
        });

        let source = "- Item 1\n- Item 2\n- Item 3\n";
        let result = unwrap_list(vec![item1, item2, item3], source);
        assert!(result.is_ok());

        if let Ok(ContentItem::List(list)) = result {
            // Location should be bounding box encompassing all items:
            // from start of item 1 (0:0) to end of item 3 (2:8)
            assert_eq!(list.location.start.line, 0);
            assert_eq!(list.location.start.column, 0);
            assert_eq!(list.location.end.line, 2);
            assert_eq!(list.location.end.column, 8);
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_unwrap_session_with_children_on_different_lines() {
        // Header spanning lines 0-1
        let mut subject_token = make_line_token(
            LineTokenType::SubjectLine,
            vec![
                Token::Text("Multi-line".to_string()),
                Token::Whitespace,
                Token::Text("Title".to_string()),
                Token::Colon,
            ],
        );
        // Span from line 0 col 0 to line 1 col 5
        subject_token.source_span = Some(0..20);

        // Child starting earlier on line 0 (edge case: child starts before or overlaps with header)
        // This tests that min/max logic correctly computes bounding box
        let child_location = Location::new(
            Position::new(0, 15), // Overlaps with header end
            Position::new(3, 10), // Extends beyond header
        );
        let child = ContentItem::Paragraph(Paragraph {
            lines: vec![],
            location: child_location,
        });

        let source = "Multi-line\n Title:\n    Content line 1\n    Content line 2\n";
        let result = unwrap_session(&subject_token, vec![child], source);
        assert!(result.is_ok());

        if let Ok(ContentItem::Session(session)) = result {
            // Bounding box should span from header start (0:0) to child end (3:10)
            assert_eq!(session.location.start.line, 0);
            assert_eq!(session.location.start.column, 0);
            assert_eq!(session.location.end.line, 3);
            assert_eq!(session.location.end.column, 10);
        } else {
            panic!("Expected Session");
        }
    }

    #[test]
    fn test_unwrap_definition_empty_children_uses_subject_location() {
        // Definition with no children should use only subject location
        let mut subject_token = make_line_token(
            LineTokenType::SubjectLine,
            vec![Token::Text("Term".to_string()), Token::Colon],
        );
        subject_token.source_span = Some(0..5);

        let source = "Term:\n";
        let result = unwrap_definition(&subject_token, vec![], source);
        assert!(result.is_ok());

        if let Ok(ContentItem::Definition(definition)) = result {
            // With empty children, location should match just the subject
            assert_eq!(definition.location.start.line, 0);
            assert_eq!(definition.location.start.column, 0);
            assert_eq!(definition.location.end.line, 0);
            assert_eq!(definition.location.end.column, 5);
        } else {
            panic!("Expected Definition");
        }
    }

    #[test]
    fn test_unwrap_list_item_no_nested_content_uses_item_location() {
        // List item with no nested content should use only item location
        let mut item_token = make_line_token(
            LineTokenType::ListLine,
            vec![
                Token::Dash,
                Token::Whitespace,
                Token::Text("Item".to_string()),
            ],
        );
        // Span covers "- Item" (6 characters)
        item_token.source_span = Some(0..6);

        let source = "- Item\n";
        let result = unwrap_list_item(&item_token, vec![], source);
        assert!(result.is_ok());

        if let Ok(ContentItem::ListItem(item)) = result {
            // With no nested content, location should match just the item
            assert_eq!(item.location.start.line, 0);
            assert_eq!(item.location.start.column, 0);
            assert_eq!(item.location.end.line, 0);
            assert_eq!(item.location.end.column, 6);
        } else {
            panic!("Expected ListItem");
        }
    }
}
