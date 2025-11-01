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

    /// Try to match a foreign block pattern
    /// Pattern: SUBJECT_LINE + optional BLANK_LINE + INDENT...DEDENT + ANNOTATION_LINE
    pub fn try_foreign_block(&self, token_types: &[LineTokenType]) -> Option<(usize, usize)> {
        if token_types.is_empty() {
            return None;
        }

        // Must start with SUBJECT_LINE
        if token_types[0] != LineTokenType::SubjectLine {
            return None;
        }

        // Find pattern: SUBJECT_LINE ... INDENT ... DEDENT ... ANNOTATION_LINE
        let mut indent_idx = None;
        let mut dedent_idx = None;
        let mut annotation_idx = None;

        for (i, token_type) in token_types.iter().enumerate() {
            if *token_type == LineTokenType::IndentLevel && indent_idx.is_none() {
                indent_idx = Some(i);
            } else if *token_type == LineTokenType::DedentLevel
                && dedent_idx.is_none()
                && indent_idx.is_some()
            {
                dedent_idx = Some(i);
            } else if *token_type == LineTokenType::AnnotationLine
                && annotation_idx.is_none()
                && dedent_idx.is_some()
            {
                annotation_idx = Some(i);
                break;
            }
        }

        // Check if we found the complete pattern
        match (indent_idx, dedent_idx, annotation_idx) {
            (Some(indent), Some(_dedent), Some(annotation)) => {
                let end = annotation + 1;
                Some((end, indent))
            }
            _ => None,
        }
    }

    /// Try to match a list pattern
    /// Pattern: BLANK_LINE + 2+ list items (LIST_LINE or SUBJECT_LINE with list marker)
    pub fn try_list(&self, token_types: &[LineTokenType]) -> Option<usize> {
        if token_types.is_empty() {
            return None;
        }

        // Must start with BLANK_LINE
        if token_types[0] != LineTokenType::BlankLine {
            return None;
        }

        // Count consecutive LIST_LINE or SUBJECT_LINE items after blank line
        let mut count = 1; // Count the blank line
        let mut item_count = 0;

        for token_type in &token_types[1..] {
            match token_type {
                LineTokenType::ListLine | LineTokenType::SubjectLine => {
                    item_count += 1;
                    count += 1;
                }
                LineTokenType::BlankLine
                | LineTokenType::IndentLevel
                | LineTokenType::DedentLevel => {
                    break;
                }
                _ => {
                    // Non-list token - stop here
                    break;
                }
            }
        }

        // Require at least 2 list items
        if item_count >= 2 {
            eprintln!(
                "DEBUG try_list: MATCHED! count={}, item_count={}, remaining={:?}",
                count,
                item_count,
                token_types.iter().skip(count).take(3).collect::<Vec<_>>()
            );
            Some(count)
        } else {
            eprintln!(
                "DEBUG try_list: NO MATCH. count={}, item_count={}, first_5={:?}",
                count,
                item_count,
                token_types.iter().take(5).collect::<Vec<_>>()
            );
            None
        }
    }

    /// Try to match a definition pattern (no blank line between subject and block)
    /// Pattern: SUBJECT_LINE (without BLANK_LINE following)
    /// Note: When used with has_following_block, the Block itself represents the INDENT
    pub fn try_definition(&self, token_types: &[LineTokenType]) -> Option<usize> {
        if token_types.is_empty() {
            return None;
        }

        // Must start with SUBJECT_LINE
        if token_types[0] != LineTokenType::SubjectLine {
            return None;
        }

        // If we have 2+ tokens, the next must NOT be BLANK_LINE (that would be a session)
        if token_types.len() > 1 && token_types[1] == LineTokenType::BlankLine {
            return None;
        }

        // Check if there's an explicit INDENT token
        if token_types.len() >= 2 && token_types[1] == LineTokenType::IndentLevel {
            Some(2) // SUBJECT_LINE + INDENT
        } else {
            // If no explicit INDENT token, just SUBJECT_LINE (Block will follow implicitly)
            Some(1)
        }
    }

    /// Try to match a session pattern (with blank line between subject and block)
    /// Pattern: SUBJECT_LINE + BLANK_LINE
    /// Note: When used with has_following_block, the Block itself represents the INDENT
    pub fn try_session(&self, token_types: &[LineTokenType]) -> Option<usize> {
        if token_types.is_empty() {
            return None;
        }

        // Must start with SUBJECT_LINE
        if token_types[0] != LineTokenType::SubjectLine {
            return None;
        }

        // Next must be BLANK_LINE
        if token_types.len() < 2 || token_types[1] != LineTokenType::BlankLine {
            return None;
        }

        // Check if there's an explicit INDENT token after blank line
        if token_types.len() >= 3 && token_types[2] == LineTokenType::IndentLevel {
            Some(3) // SUBJECT_LINE + BLANK_LINE + INDENT
        } else {
            // If no explicit INDENT token, just SUBJECT_LINE + BLANK_LINE (Block will follow implicitly)
            Some(2)
        }
    }

    /// Try to match a paragraph (fallback - always succeeds, consumes tokens until blank or structural)
    pub fn try_paragraph(&self, token_types: &[LineTokenType]) -> Option<usize> {
        if token_types.is_empty() {
            return None;
        }

        // Special case: lone blank line (returns 1 to skip it, doesn't generate content)
        if token_types[0] == LineTokenType::BlankLine {
            return Some(1);
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
    fn test_foreign_block_pattern_match() {
        let rules = TxxtGrammarRules::new().unwrap();
        let tokens = vec![
            LineTokenType::SubjectLine,
            LineTokenType::IndentLevel,
            LineTokenType::ParagraphLine,
            LineTokenType::DedentLevel,
            LineTokenType::AnnotationLine,
        ];

        let result = rules.try_foreign_block(&tokens);
        assert!(result.is_some(), "Should match foreign block pattern");
    }

    #[test]
    fn test_foreign_block_pattern_no_match_missing_annotation() {
        let rules = TxxtGrammarRules::new().unwrap();
        let tokens = vec![
            LineTokenType::SubjectLine,
            LineTokenType::IndentLevel,
            LineTokenType::ParagraphLine,
            LineTokenType::DedentLevel,
        ];

        let result = rules.try_foreign_block(&tokens);
        assert!(
            result.is_none(),
            "Should not match without closing annotation"
        );
    }

    #[test]
    fn test_list_pattern_match_two_items() {
        let rules = TxxtGrammarRules::new().unwrap();
        let tokens = vec![
            LineTokenType::BlankLine,
            LineTokenType::ListLine,
            LineTokenType::ListLine,
        ];

        let result = rules.try_list(&tokens);
        assert_eq!(result, Some(3), "Should match list with 2+ items");
    }

    #[test]
    fn test_list_pattern_no_match_single_item() {
        let rules = TxxtGrammarRules::new().unwrap();
        let tokens = vec![LineTokenType::BlankLine, LineTokenType::ListLine];

        let result = rules.try_list(&tokens);
        assert_eq!(result, None, "Should not match single list item");
    }

    #[test]
    fn test_list_pattern_stops_at_blank_line() {
        let rules = TxxtGrammarRules::new().unwrap();
        let tokens = vec![
            LineTokenType::BlankLine,
            LineTokenType::ListLine,
            LineTokenType::ListLine,
            LineTokenType::BlankLine,
            LineTokenType::ListLine,
        ];

        let result = rules.try_list(&tokens);
        assert_eq!(
            result,
            Some(3),
            "List should stop at next blank line, consuming 3 items"
        );
    }

    #[test]
    fn test_definition_pattern_match() {
        let rules = TxxtGrammarRules::new().unwrap();
        let tokens = vec![LineTokenType::SubjectLine, LineTokenType::IndentLevel];

        let result = rules.try_definition(&tokens);
        assert_eq!(result, Some(2), "Should match definition pattern");
    }

    #[test]
    fn test_definition_pattern_no_match_blank_line_between() {
        let rules = TxxtGrammarRules::new().unwrap();
        let tokens = vec![
            LineTokenType::SubjectLine,
            LineTokenType::BlankLine,
            LineTokenType::IndentLevel,
        ];

        let result = rules.try_definition(&tokens);
        assert_eq!(
            result, None,
            "Should not match definition with blank line after subject"
        );
    }

    #[test]
    fn test_session_pattern_match() {
        let rules = TxxtGrammarRules::new().unwrap();
        let tokens = vec![
            LineTokenType::SubjectLine,
            LineTokenType::BlankLine,
            LineTokenType::IndentLevel,
        ];

        let result = rules.try_session(&tokens);
        assert_eq!(result, Some(3), "Should match session pattern");
    }

    #[test]
    fn test_session_pattern_no_match_no_blank_line() {
        let rules = TxxtGrammarRules::new().unwrap();
        let tokens = vec![LineTokenType::SubjectLine, LineTokenType::IndentLevel];

        let result = rules.try_session(&tokens);
        assert_eq!(result, None, "Should not match session without blank line");
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
