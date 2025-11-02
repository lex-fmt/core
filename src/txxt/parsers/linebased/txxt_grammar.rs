//! txxt Grammar Rules and Pattern Matching
//!
//! This module implements the txxt-specific grammar patterns using the generic regex grammar engine.
//! Grammar rules are applied in order of specificity (most specific first):
//!
//! 1. Annotation - Lines with :: markers (most specific)
//! 2. Paragraph - Fallback for any non-matching line (least specific)
//!
//! More complex patterns are added in subsequent steps (foreign blocks, lists, definitions, sessions).

use crate::txxt::lexers::LineTokenType;
use crate::txxt::parsers::linebased::regex_grammar_engine::{RegexGrammarMatcher, TokenSeq};

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

/// Helper to analyze lead characteristics
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LeadType {
    Annotation,  // Has :: markers
    SeqMarker,   // Starts with list marker (-, 1., a., I., etc.)
    SubjectLine, // Ends with colon
    PlainText,   // Plain text line (could be session, paragraph, etc.)
}

/// Analyze a line token to determine its lead type
pub fn analyze_lead(token: &crate::txxt::lexers::LineToken) -> LeadType {
    match token.line_type {
        LineTokenType::AnnotationLine => LeadType::Annotation,
        LineTokenType::ListLine => LeadType::SeqMarker,
        LineTokenType::SubjectLine => LeadType::SubjectLine,
        LineTokenType::ParagraphLine => LeadType::PlainText,
        _ => LeadType::PlainText,
    }
}

/// Extract the list marker type from source tokens
/// Returns a marker type identifier to distinguish dash lists, numbered lists, lettered lists, etc.
fn extract_list_marker_type(tokens: &[crate::txxt::lexers::tokens::Token]) -> String {
    use crate::txxt::lexers::tokens::Token;

    for token in tokens {
        match token {
            Token::Dash => return "dash".to_string(),
            Token::Number(_) => return "number".to_string(),
            Token::Text(s) if s.len() == 1 && s.chars().next().unwrap().is_alphabetic() => {
                return "letter".to_string();
            }
            Token::Text(s)
                if s.chars()
                    .all(|c| matches!(c, 'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M'))
                    && !s.is_empty() =>
            {
                return "roman".to_string();
            }
            _ => {}
        }
    }

    "unknown".to_string()
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

    /// Try to match a foreign block pattern from tree
    /// Supports two forms:
    /// 1. Block form: SUBJECT_LINE + optional BLANK_LINE + BLOCK + ANNOTATION_LINE (closing)
    ///    Example: "Language:\n    code content\n:: language ::"
    /// 2. Marker form: SUBJECT_LINE + ANNOTATION_LINE (no content block)
    ///    Example: "Image:\n:: image src=... ::"
    pub fn try_foreign_block_from_tree(
        &self,
        tree: &[crate::txxt::lexers::LineTokenTree],
    ) -> Option<usize> {
        use crate::txxt::lexers::LineTokenTree;

        if tree.is_empty() {
            return None;
        }

        // Must start with SUBJECT_LINE
        let subject_is_valid = matches!(tree.first(), Some(LineTokenTree::Token(t)) if t.line_type == LineTokenType::SubjectLine);
        if !subject_is_valid {
            return None;
        }

        let mut idx = 1;

        // Optional BLANK_LINE after subject
        let has_blank = matches!(tree.get(idx), Some(LineTokenTree::Token(t)) if t.line_type == LineTokenType::BlankLine);
        if has_blank {
            idx += 1;
        }

        // Check what comes next: BLOCK or ANNOTATION_LINE
        match tree.get(idx) {
            // Form 1: BLOCK + ANNOTATION_LINE (block form with content)
            Some(LineTokenTree::Block(_)) => {
                idx += 1;
                // Must have a closing ANNOTATION_LINE after the block
                let has_closing_annotation = matches!(tree.get(idx), Some(LineTokenTree::Token(t)) if t.line_type == LineTokenType::AnnotationLine);
                if has_closing_annotation {
                    idx += 1;
                    return Some(idx); // Successfully matched: subject (blank?) block annotation
                }
                None
            }
            // Form 2: ANNOTATION_LINE directly (marker form, no content block)
            Some(LineTokenTree::Token(t)) if t.line_type == LineTokenType::AnnotationLine => {
                idx += 1;
                Some(idx) // Successfully matched: subject (blank?) annotation
            }
            _ => None,
        }
    }

    /// Try to match a list pattern (DEPRECATED - use try_list_from_tree)
    /// Pattern: BLANK_LINE + 2+ list items (LIST_LINE or SUBJECT_LINE with list marker)
    #[deprecated]
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
            Some(count)
        } else {
            None
        }
    }

    /// Try to match a list pattern using tree structure
    ///
    /// Pattern: LIST_ITEM (BLANK_LINE? BLOCK)? LIST_ITEM (BLANK_LINE? BLOCK)? ...
    ///
    /// Key characteristics of a LIST:
    /// - Starts with SEQ_MARKER (-, 1., a., I., etc.)
    /// - NO blank line after the marker (forbidden between items)
    /// - Optional content block (indented children)
    /// - Next item is another SEQ_MARKER **of the same type** at same level
    /// - Requires at least 2 items to be a list
    ///
    /// Important: List items must use consistent marker type (all dashes, all numbers, etc.)
    /// This prevents "- item\n4. session-title" from being parsed as a mixed list.
    pub fn try_list_from_tree(&self, tree: &[crate::txxt::lexers::LineTokenTree]) -> Option<usize> {
        use crate::txxt::lexers::LineTokenTree;

        if tree.is_empty() {
            return None;
        }

        // Must start with LIST_LINE
        let first_list_token = match tree.first() {
            Some(LineTokenTree::Token(t)) if t.line_type == LineTokenType::ListLine => Some(t),
            _ => None,
        }?;

        // Extract the marker type from the first list item's source tokens
        let first_marker_type = extract_list_marker_type(&first_list_token.source_tokens);

        // Count list items: each LIST_ITEM = LIST_LINE (BLANK_LINE? BLOCK)?
        // KEY: NO blank line is allowed between items (blank lines are only WITHIN items, before their content block)
        let mut tree_idx = 0;
        let mut item_count = 0;

        while tree_idx < tree.len() {
            // Must have a LIST_LINE at this position
            match tree.get(tree_idx) {
                Some(LineTokenTree::Token(t)) if t.line_type == LineTokenType::ListLine => {
                    // Check that this list item uses the SAME marker type as the first item
                    let marker_type = extract_list_marker_type(&t.source_tokens);
                    if marker_type != first_marker_type {
                        // Different marker type - not part of this list
                        break;
                    }

                    item_count += 1;
                    tree_idx += 1;
                }
                _ => break, // No more list items
            }

            // Check for optional BLANK_LINE? BLOCK pattern (content of this list item)
            if tree_idx < tree.len() {
                // Check if next is BLANK_LINE
                let has_blank = matches!(tree.get(tree_idx), Some(LineTokenTree::Token(t)) if t.line_type == LineTokenType::BlankLine);
                if has_blank {
                    tree_idx += 1;
                }

                // Check if next is BLOCK (the indented content of this item)
                if matches!(tree.get(tree_idx), Some(LineTokenTree::Block(_))) {
                    tree_idx += 1;
                }
            }

            // Stop if next item is not a LIST_LINE
            // This prevents "1. Item\n2. Item" from being parsed as a list when followed by "1. Session"
            if !matches!(tree.get(tree_idx), Some(LineTokenTree::Token(t)) if t.line_type == LineTokenType::ListLine)
            {
                break;
            }
        }

        // Require at least 2 list items
        if item_count >= 2 {
            Some(tree_idx)
        } else {
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
    #[deprecated]
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

    /// Try to match a session pattern using tree structure
    ///
    /// Pattern: (BLANK_LINE?) <ANY_LINE> BLANK_LINE BLOCK
    ///
    /// Key characteristics of a SESSION:
    /// - The lead line is surrounded by blank lines (before and after)
    /// - Followed by indented content block
    /// - The lead can be ANY line type except annotation (which stands alone)
    ///
    /// Note: A SESSION requires a blank line AFTER the lead (distinguishes from DEFINITION)
    ///
    /// This function is self-contained: it matches and consumes the entire session pattern
    /// including any leading blank line, the lead, the blank line after lead, and the block.
    ///
    /// # Returns
    /// The consumed count for the entire matched pattern (includes everything: optional leading blank, lead, blank, block).
    pub fn try_session_from_tree(
        &self,
        tree: &[crate::txxt::lexers::LineTokenTree],
    ) -> Option<usize> {
        use crate::txxt::lexers::LineTokenTree;

        if tree.len() < 3 {
            return None; // Need at least: lead + blank + block
        }

        // Check for optional BLANK_LINE before lead
        let mut idx = 0;
        if matches!(tree.first(), Some(LineTokenTree::Token(t)) if t.line_type == LineTokenType::BlankLine)
        {
            idx = 1;
        }

        // Must have a lead (any line except blank or structural)
        let lead_is_valid = matches!(tree.get(idx), Some(LineTokenTree::Token(t))
            if t.line_type != LineTokenType::BlankLine
            && t.line_type != LineTokenType::IndentLevel
            && t.line_type != LineTokenType::DedentLevel);

        if !lead_is_valid {
            return None;
        }
        idx += 1; // Move past lead

        // Must have a BLANK_LINE after lead
        let has_blank_after = matches!(tree.get(idx), Some(LineTokenTree::Token(t)) if t.line_type == LineTokenType::BlankLine);
        if !has_blank_after {
            return None;
        }
        idx += 1; // Move past blank

        // Must have a BLOCK (indented content) after blank
        let has_block = matches!(tree.get(idx), Some(LineTokenTree::Block(_)));
        if !has_block {
            return None;
        }
        idx += 1; // Move past block

        // Successfully matched entire pattern
        // Return total consumed count
        Some(idx)
    }

    /// Try to match an annotation with block content pattern from tree
    /// Pattern: ANNOTATION_LINE (opening with label+params) BLANK_LINE? BLOCK ANNOTATION_LINE (closing)
    /// Examples:
    /// - `:: note ::\n    paragraph\n::`  (simple case)
    /// - `:: note author="Jane" ::\n    paragraph\n::`  (with parameters)
    pub fn try_annotation_from_tree(
        &self,
        tree: &[crate::txxt::lexers::LineTokenTree],
    ) -> Option<usize> {
        use crate::txxt::lexers::LineTokenTree;

        if tree.len() < 3 {
            return None; // Need at least: opening_annotation + block + closing_annotation
        }

        // Must start with ANNOTATION_LINE (opening)
        let opening_is_annotation = matches!(tree.first(), Some(LineTokenTree::Token(t)) if t.line_type == LineTokenType::AnnotationLine);
        if !opening_is_annotation {
            return None;
        }

        let mut idx = 1;

        // Optional BLANK_LINE after opening annotation
        let has_blank = matches!(tree.get(idx), Some(LineTokenTree::Token(t)) if t.line_type == LineTokenType::BlankLine);
        if has_blank {
            idx += 1;
        }

        // Must have a BLOCK (indented content) after blank (or opening if no blank)
        let has_block = matches!(tree.get(idx), Some(LineTokenTree::Block(_)));
        if !has_block {
            return None;
        }
        idx += 1;

        // Must have a closing ANNOTATION_LINE (::)
        let has_closing = matches!(tree.get(idx), Some(LineTokenTree::Token(t)) if t.line_type == LineTokenType::AnnotationLine);
        if !has_closing {
            return None;
        }
        idx += 1;

        // Successfully matched: annotation block blank block annotation
        Some(idx)
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
    #[allow(deprecated)]
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
    #[allow(deprecated)]
    fn test_list_pattern_no_match_single_item() {
        let rules = TxxtGrammarRules::new().unwrap();
        let tokens = vec![LineTokenType::BlankLine, LineTokenType::ListLine];

        let result = rules.try_list(&tokens);
        assert_eq!(result, None, "Should not match single list item");
    }

    #[test]
    #[allow(deprecated)]
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
    #[allow(deprecated)]
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
    #[allow(deprecated)]
    fn test_session_pattern_no_match_no_blank_line() {
        let rules = TxxtGrammarRules::new().unwrap();
        let tokens = vec![LineTokenType::SubjectLine, LineTokenType::IndentLevel];

        let result = rules.try_session(&tokens);
        assert_eq!(result, None, "Should not match session without blank line");
    }

    #[test]
    fn test_step1_integration_annotation_and_paragraph() {
        // Step 1: Test that we can parse annotations and paragraphs
        use crate::txxt::lexers::LineTokenTree;
        use crate::txxt::lexers::{LineToken, LineTokenType, Token};
        use crate::txxt::parsers::linebased::parse_experimental;

        // Create a simple tree with annotation and paragraph
        let tree = vec![
            LineTokenTree::Token(LineToken {
                source_tokens: vec![
                    Token::TxxtMarker,
                    Token::Text("note".to_string()),
                    Token::TxxtMarker,
                ],
                line_type: LineTokenType::AnnotationLine,
                source_span: None,
            }),
            LineTokenTree::Token(LineToken {
                source_tokens: vec![Token::Text("Some text".to_string())],
                line_type: LineTokenType::ParagraphLine,
                source_span: None,
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
                crate::txxt::parsers::ContentItem::Annotation(_)
            ),
            "First item should be annotation"
        );

        // Second should be paragraph
        assert!(
            matches!(
                &doc.root.content[1],
                crate::txxt::parsers::ContentItem::Paragraph(_)
            ),
            "Second item should be paragraph"
        );
    }

    #[test]
    fn test_all_token_types_to_string() {
        // Test all LineTokenType variants are correctly mapped to strings
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
        assert_eq!(token_type_to_string(&LineTokenType::ListLine), "LIST_LINE");
        assert_eq!(
            token_type_to_string(&LineTokenType::ParagraphLine),
            "PARAGRAPH_LINE"
        );
        assert_eq!(token_type_to_string(&LineTokenType::IndentLevel), "INDENT");
        assert_eq!(token_type_to_string(&LineTokenType::DedentLevel), "DEDENT");
    }

    #[test]
    fn test_analyze_lead_all_types() {
        use crate::txxt::lexers::{LineToken, Token};

        // Test annotation line
        let annotation_token = LineToken {
            source_tokens: vec![Token::TxxtMarker],
            line_type: LineTokenType::AnnotationLine,
            source_span: None,
        };
        assert_eq!(analyze_lead(&annotation_token), LeadType::Annotation);

        // Test list line
        let list_token = LineToken {
            source_tokens: vec![Token::Dash],
            line_type: LineTokenType::ListLine,
            source_span: None,
        };
        assert_eq!(analyze_lead(&list_token), LeadType::SeqMarker);

        // Test subject line
        let subject_token = LineToken {
            source_tokens: vec![Token::Text("title".to_string())],
            line_type: LineTokenType::SubjectLine,
            source_span: None,
        };
        assert_eq!(analyze_lead(&subject_token), LeadType::SubjectLine);

        // Test paragraph line
        let paragraph_token = LineToken {
            source_tokens: vec![Token::Text("content".to_string())],
            line_type: LineTokenType::ParagraphLine,
            source_span: None,
        };
        assert_eq!(analyze_lead(&paragraph_token), LeadType::PlainText);

        // Test structural tokens map to PlainText
        let indent_token = LineToken {
            source_tokens: vec![],
            line_type: LineTokenType::IndentLevel,
            source_span: None,
        };
        assert_eq!(analyze_lead(&indent_token), LeadType::PlainText);

        let blank_token = LineToken {
            source_tokens: vec![],
            line_type: LineTokenType::BlankLine,
            source_span: None,
        };
        assert_eq!(analyze_lead(&blank_token), LeadType::PlainText);
    }

    #[test]
    fn test_extract_list_marker_type_all_markers() {
        use crate::txxt::lexers::tokens::Token;

        // Test dash marker
        assert_eq!(extract_list_marker_type(&[Token::Dash]), "dash".to_string());

        // Test number marker
        assert_eq!(
            extract_list_marker_type(&[Token::Number("1".to_string())]),
            "number".to_string()
        );

        // Test single letter marker (lowercase)
        assert_eq!(
            extract_list_marker_type(&[Token::Text("a".to_string())]),
            "letter".to_string()
        );

        // Test single letter marker (uppercase)
        assert_eq!(
            extract_list_marker_type(&[Token::Text("A".to_string())]),
            "letter".to_string()
        );

        // Test roman numeral marker (multi-char uppercase roman only - single chars are letters)
        assert_eq!(
            extract_list_marker_type(&[Token::Text("II".to_string())]),
            "roman".to_string()
        );
        assert_eq!(
            extract_list_marker_type(&[Token::Text("VI".to_string())]),
            "roman".to_string()
        );
        assert_eq!(
            extract_list_marker_type(&[Token::Text("IV".to_string())]),
            "roman".to_string()
        );
        assert_eq!(
            extract_list_marker_type(&[Token::Text("XL".to_string())]),
            "roman".to_string()
        );

        // Test unknown marker (multi-char non-roman)
        assert_eq!(
            extract_list_marker_type(&[Token::Text("abc".to_string())]),
            "unknown".to_string()
        );

        // Test empty tokens returns unknown
        assert_eq!(extract_list_marker_type(&[]), "unknown".to_string());
    }
}
