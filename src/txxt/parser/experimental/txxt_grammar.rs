//! txxt Grammar Rules and Pattern Matching
//!
//! This module implements the txxt-specific grammar patterns using the generic regex grammar engine.
//! Grammar rules are applied in order of specificity (most specific first):
//!
//! 1. Annotation - Lines with :: markers (most specific)
//! 2. Paragraph - Fallback for any non-matching line (least specific)
//!
//! More complex patterns are added in subsequent steps (foreign blocks, lists, definitions, sessions).

use crate::txxt::lexer::tokens::LineTokenType;
use crate::txxt::parser::experimental::regex_grammar_engine::{RegexGrammarMatcher, TokenSeq};

/// Convert LineTokenType enum to its string representation for pattern matching
fn token_type_to_string(token_type: &LineTokenType) -> String {
    match token_type {
        LineTokenType::BlankLine => "BLANK_LINE",
        LineTokenType::AnnotationLine => "ANNOTATION_LINE",
        LineTokenType::SubjectLine => "SUBJECT_LINE",
        LineTokenType::ListLine => "LIST_LINE",
        LineTokenType::ParagraphLine => "PARAGRAPH_LINE",
        LineTokenType::IndentLevel => "INDENT",
        LineTokenType::DedentLevel => "DEDENT",
    }
    .to_string()
}

/// Convert a sequence of LineTokenType values to a space-separated string for regex matching
pub fn token_types_to_string(tokens: &[LineTokenType]) -> String {
    tokens
        .iter()
        .map(token_type_to_string)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Represents a recognized grammar pattern
#[derive(Debug, Clone, PartialEq)]
pub enum MatchedPattern {
    /// An annotation line (::)
    Annotation,
    /// A paragraph (fallback for any non-matching line)
    Paragraph,
    /// A foreign block (subject + indent + content + dedent + annotation)
    ForeignBlock,
    /// A list (blank line + 2+ list items)
    List,
    /// A definition (subject + indent, no blank line)
    Definition,
    /// A session (subject + blank line + indent)
    Session,
}

/// Grammar rules for txxt parsing
pub struct TxxtGrammarRules {
    annotation_pattern: RegexGrammarMatcher,
    // More patterns will be added in subsequent steps
}

impl TxxtGrammarRules {
    /// Create a new instance of grammar rules
    pub fn new() -> Result<Self, String> {
        // Pattern: A single ANNOTATION_LINE
        let annotation_pattern =
            RegexGrammarMatcher::new("ANNOTATION_LINE").map_err(|e| e.to_string())?;

        Ok(TxxtGrammarRules { annotation_pattern })
    }

    /// Try to match an annotation pattern at the given position
    /// Returns the number of tokens consumed if successful
    pub fn try_annotation(&self, token_types: &[LineTokenType]) -> Option<usize> {
        if token_types.is_empty() {
            return None;
        }

        let m = self
            .annotation_pattern
            .match_tokens(&TokenSeq::new(vec![token_type_to_string(&token_types[0])]));

        if m.matched {
            // Annotation is a single line
            return Some(1);
        }

        None
    }

    /// Try to match a paragraph (fallback - always succeeds, consumes tokens until blank or structural)
    pub fn try_paragraph(&self, token_types: &[LineTokenType]) -> Option<usize> {
        if token_types.is_empty() {
            return None;
        }

        // Paragraph matches until we hit a BLANK_LINE, INDENT, or DEDENT
        let mut count = 0;
        for token_type in token_types {
            match token_type {
                LineTokenType::BlankLine => break,
                LineTokenType::IndentLevel => break,
                LineTokenType::DedentLevel => break,
                _ => count += 1,
            }
        }

        if count > 0 {
            Some(count)
        } else {
            None
        }
    }
}

impl Default for TxxtGrammarRules {
    fn default() -> Self {
        Self::new().expect("Failed to create default grammar rules")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_type_to_string() {
        assert_eq!(
            token_type_to_string(&LineTokenType::BlankLine),
            "BLANK_LINE"
        );
        assert_eq!(
            token_type_to_string(&LineTokenType::AnnotationLine),
            "ANNOTATION_LINE"
        );
        assert_eq!(
            token_type_to_string(&LineTokenType::SubjectLine),
            "SUBJECT_LINE"
        );
        assert_eq!(
            token_type_to_string(&LineTokenType::ParagraphLine),
            "PARAGRAPH_LINE"
        );
    }

    #[test]
    fn test_token_types_to_string() {
        let tokens = vec![
            LineTokenType::AnnotationLine,
            LineTokenType::BlankLine,
            LineTokenType::ParagraphLine,
        ];
        let result = token_types_to_string(&tokens);
        assert_eq!(result, "ANNOTATION_LINE BLANK_LINE PARAGRAPH_LINE");
    }

    #[test]
    fn test_grammar_rules_creation() {
        let rules = TxxtGrammarRules::new();
        assert!(rules.is_ok());
    }

    #[test]
    fn test_annotation_pattern_match() {
        let rules = TxxtGrammarRules::new().unwrap();
        let tokens = vec![LineTokenType::AnnotationLine];

        let result = rules.try_annotation(&tokens);
        assert_eq!(result, Some(1));
    }

    #[test]
    fn test_annotation_pattern_no_match_paragraph() {
        let rules = TxxtGrammarRules::new().unwrap();
        let tokens = vec![LineTokenType::ParagraphLine];

        let result = rules.try_annotation(&tokens);
        assert_eq!(result, None);
    }

    #[test]
    fn test_paragraph_pattern_match_single() {
        let rules = TxxtGrammarRules::new().unwrap();
        let tokens = vec![LineTokenType::ParagraphLine];

        let result = rules.try_paragraph(&tokens);
        assert_eq!(result, Some(1));
    }

    #[test]
    fn test_paragraph_pattern_match_multiple() {
        let rules = TxxtGrammarRules::new().unwrap();
        let tokens = vec![
            LineTokenType::ParagraphLine,
            LineTokenType::ParagraphLine,
            LineTokenType::ParagraphLine,
        ];

        let result = rules.try_paragraph(&tokens);
        assert_eq!(result, Some(3));
    }

    #[test]
    fn test_paragraph_stops_at_blank_line() {
        let rules = TxxtGrammarRules::new().unwrap();
        let tokens = vec![
            LineTokenType::ParagraphLine,
            LineTokenType::BlankLine,
            LineTokenType::ParagraphLine,
        ];

        let result = rules.try_paragraph(&tokens);
        assert_eq!(result, Some(1));
    }

    #[test]
    fn test_paragraph_stops_at_indent() {
        let rules = TxxtGrammarRules::new().unwrap();
        let tokens = vec![
            LineTokenType::ParagraphLine,
            LineTokenType::IndentLevel,
            LineTokenType::ParagraphLine,
        ];

        let result = rules.try_paragraph(&tokens);
        assert_eq!(result, Some(1));
    }

    #[test]
    fn test_paragraph_stops_at_dedent() {
        let rules = TxxtGrammarRules::new().unwrap();
        let tokens = vec![
            LineTokenType::ParagraphLine,
            LineTokenType::DedentLevel,
            LineTokenType::ParagraphLine,
        ];

        let result = rules.try_paragraph(&tokens);
        assert_eq!(result, Some(1));
    }

    #[test]
    fn test_paragraph_with_subject_line() {
        let rules = TxxtGrammarRules::new().unwrap();
        let tokens = vec![LineTokenType::SubjectLine];

        let result = rules.try_paragraph(&tokens);
        assert_eq!(result, Some(1));
    }

    #[test]
    fn test_paragraph_with_list_line() {
        let rules = TxxtGrammarRules::new().unwrap();
        let tokens = vec![LineTokenType::ListLine];

        let result = rules.try_paragraph(&tokens);
        assert_eq!(result, Some(1));
    }

    #[test]
    fn test_paragraph_no_match_empty() {
        let rules = TxxtGrammarRules::new().unwrap();
        let tokens = vec![];

        let result = rules.try_paragraph(&tokens);
        assert_eq!(result, None);
    }

    #[test]
    fn test_default_creation() {
        let _rules = TxxtGrammarRules::default();
        // If we get here without panicking, default works
    }

    #[test]
    fn test_step1_integration_annotation_and_paragraph() {
        // Step 1: Test that we can parse annotations and paragraphs
        use crate::txxt::lexer::tokens::{LineToken, LineTokenType, Token};
        use crate::txxt::lexer::transformations::experimental_transform_indentation_to_token_tree::LineTokenTree;
        use crate::txxt::parser::experimental::parse_experimental;

        // Create a simple tree with annotation and paragraph
        let tree = vec![
            LineTokenTree::Token(LineToken {
                source_tokens: vec![
                    Token::TxxtMarker,
                    Token::Text("note".to_string()),
                    Token::TxxtMarker,
                ],
                line_type: LineTokenType::AnnotationLine,
            }),
            LineTokenTree::Token(LineToken {
                source_tokens: vec![Token::Text("Some text".to_string())],
                line_type: LineTokenType::ParagraphLine,
            }),
        ];

        let result = parse_experimental(tree, ":: note ::\nSome text\n");
        assert!(result.is_ok(), "Failed to parse annotation and paragraph");

        let doc = result.unwrap();
        assert_eq!(doc.root.content.len(), 2, "Expected 2 content items");

        // First should be annotation
        assert!(
            matches!(
                &doc.root.content[0],
                crate::txxt::parser::ContentItem::Annotation(_)
            ),
            "First item should be annotation"
        );

        // Second should be paragraph
        assert!(
            matches!(
                &doc.root.content[1],
                crate::txxt::parser::ContentItem::Paragraph(_)
            ),
            "Second item should be paragraph"
        );
    }
}
