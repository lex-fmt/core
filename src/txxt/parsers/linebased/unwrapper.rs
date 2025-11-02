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
use crate::txxt::ast::{
    Annotation, Definition, Label, List, ListItem, Paragraph, Session, TextContent, TextLine,
};
use crate::txxt::lexers::{LineToken, Token};
use crate::txxt::parsers::{ContentItem, Location, Position};

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
/// Creates a location that starts at the first token and ends at the last token,
/// covering all content in between.
fn extract_location_from_tokens(tokens: &[LineToken], source: &str) -> Location {
    if tokens.is_empty() {
        return default_location();
    }

    let source_location = SourceLocation::new(source);
    let mut combined_start: Option<usize> = None;
    let mut combined_end: Option<usize> = None;

    for token in tokens {
        if let Some(ref span) = token.source_span {
            combined_start = Some(combined_start.map_or(span.start, |s| s.min(span.start)));
            combined_end = Some(combined_end.map_or(span.end, |e| e.max(span.end)));
        }
    }

    match (combined_start, combined_end) {
        (Some(start), Some(end)) => source_location.range_to_location(&(start..end)),
        _ => default_location(),
    }
}

/// Create a default location (for now, until we add proper location tracking from source)
///
/// Used when source span information is not available.
fn default_location() -> Location {
    Location {
        start: Position { line: 0, column: 0 },
        end: Position { line: 0, column: 0 },
    }
}

/// Create a Session AST node from a subject line token and content
///
/// Used by the parser when it matches: SUBJECT_LINE + BLANK_LINE + INDENT
pub fn unwrap_session(
    subject_token: &LineToken,
    content: Vec<ContentItem>,
    source: &str,
) -> Result<ContentItem, String> {
    let title_text = extract_text_from_token(subject_token);
    let title = TextContent::from_string(title_text, None);

    // Extract location from the subject token
    let location = extract_location_from_token(subject_token, source);

    let session = Session::new(title, content).at(location);
    Ok(ContentItem::Session(session))
}

/// Create a Definition AST node from a subject token and content
///
/// Used by the parser when it matches: SUBJECT_LINE + INDENT (no blank line)
pub fn unwrap_definition(
    subject_token: &LineToken,
    content: Vec<ContentItem>,
    source: &str,
) -> Result<ContentItem, String> {
    let subject_text = extract_text_from_token(subject_token);
    let subject = TextContent::from_string(subject_text, None);

    // Extract location from the subject token
    let location = extract_location_from_token(subject_token, source);

    let definition = Definition::new(subject, content).at(location);
    Ok(ContentItem::Definition(definition))
}

/// Create a List AST node from multiple list item tokens
///
/// Used by the parser when it matches: BLANK_LINE + 2+ list items
pub fn unwrap_list(list_items: Vec<ContentItem>, _source: &str) -> Result<ContentItem, String> {
    if list_items.is_empty() {
        return Err("Cannot create list with no items".to_string());
    }

    // Lists are constructed from parsed children, so we use default location
    // (The location would be computed from child item locations if needed)
    let location = default_location();

    let list = List::new(list_items).at(location);
    Ok(ContentItem::List(list))
}

/// Create a ListItem AST node from a list line token and optional nested content
///
/// Called for each item in a list
pub fn unwrap_list_item(
    item_token: &LineToken,
    content: Vec<ContentItem>,
    source: &str,
) -> Result<ContentItem, String> {
    let item_text = extract_text_from_token(item_token);

    // Extract location from the item token
    let location = extract_location_from_token(item_token, source);

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

    // Extract location from subject token, extending to closing annotation end
    let subject_location = extract_location_from_token(subject_token, source);
    let closing_location = extract_location_from_token(closing_annotation_token, source);
    let combined_location = Location::new(subject_location.start, closing_location.end);

    let foreign_block =
        crate::txxt::ast::ForeignBlock::new(subject_text, content_text, closing_annotation)
            .at(combined_location);

    Ok(ContentItem::ForeignBlock(foreign_block))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::txxt::lexers::{LineTokenType, Token};

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
            LineTokenType::AnnotationLine,
            vec![
                Token::TxxtMarker,
                Token::Text("note".to_string()),
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
}
