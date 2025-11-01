//! Experimental Parser Unwrapper - Pattern to AST Conversion
//!
//! This module handles converting matched patterns and tokens into AST nodes.
//! For now, it contains stubbed builders that create simple AST nodes.
//! Later, these will be replaced with real pattern matching and proper element builders.
//!
//! The unwrapper is responsible for:
//! 1. Taking matched pattern data + tokens
//! 2. Extracting source locations from tokens
//! 3. Building appropriate AST node types
//! 4. Handling recursive content from nested blocks

use crate::txxt::ast::{Annotation, Label, Paragraph, TextContent, TextLine};
use crate::txxt::lexer::tokens::LineToken;
use crate::txxt::parser::{ContentItem, Location, Position};

/// Stub: Convert a line token to a Paragraph ContentItem.
///
/// This is a temporary implementation that treats any token as paragraph text.
/// Later, this will be enhanced with pattern matching to recognize
/// Sessions, Definitions, Lists, etc.
pub fn unwrap_token_to_paragraph(token: &LineToken, _source: &str) -> Result<ContentItem, String> {
    // Extract text from the token
    let text_content = extract_text_from_token(token);

    // Create a TextLine from the text
    let text_line = TextLine {
        content: TextContent::from_string(text_content, None),
        location: Location {
            start: Position { line: 0, column: 0 },
            end: Position { line: 0, column: 0 },
        },
    };

    // Wrap in a Paragraph
    let paragraph = Paragraph {
        lines: vec![ContentItem::TextLine(text_line)],
        location: Location {
            start: Position { line: 0, column: 0 },
            end: Position { line: 0, column: 0 },
        },
    };

    Ok(ContentItem::Paragraph(paragraph))
}

/// Convert an annotation line token to an Annotation ContentItem.
///
/// Annotations are lines with :: markers.
/// This builds an Annotation element from the source tokens.
pub fn unwrap_annotation(token: &LineToken, _source: &str) -> Result<ContentItem, String> {
    // Extract text content from the annotation
    let text_content = extract_text_from_token(token);

    // Create an annotation with the extracted text
    let annotation = Annotation {
        label: Label::from_string(&text_content),
        parameters: vec![],
        content: vec![],
        location: Location {
            start: Position { line: 0, column: 0 },
            end: Position { line: 0, column: 0 },
        },
    };

    Ok(ContentItem::Annotation(annotation))
}

/// Extract human-readable text from a line token's source tokens.
///
/// This is a simple stub that concatenates Text tokens together.
/// Later, this will be replaced with proper token parsing.
fn extract_text_from_token(token: &LineToken) -> String {
    token
        .source_tokens
        .iter()
        .filter_map(|t| {
            if let crate::txxt::lexer::Token::Text(s) = t {
                Some(s.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::txxt::lexer::tokens::{LineTokenType, Token};

    fn make_line_token(line_type: LineTokenType, tokens: Vec<Token>) -> LineToken {
        LineToken {
            source_tokens: tokens,
            line_type,
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
    fn test_extract_text_ignores_non_text_tokens() {
        let token = make_line_token(
            LineTokenType::SubjectLine,
            vec![
                Token::Text("Title".to_string()),
                Token::Colon,
                Token::Newline,
            ],
        );

        let text = extract_text_from_token(&token);
        assert_eq!(text, "Title");
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
}
