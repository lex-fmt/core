//! Line-based token types for the linebased lexer pipeline
//!
//! This module contains token types specific to the line-based lexer pipeline:
//! - LineToken: Represents a logical line created from grouped raw tokens
//! - LineTokenType: Classification of line types
//! - LineContainerToken: Hierarchical tree structure where nodes are either line tokens or nested containers

use std::fmt;

use crate::lex::lexers::tokens::Token;

/// A line token represents one logical line created from grouped raw tokens.
///
/// Line tokens are produced by the linebased line token transformation,
/// which groups raw tokens into semantic line units. Each line token stores:
/// - The original raw tokens that created it (for location information and AST construction)
/// - The line type (what kind of line this is)
/// - Individual token spans (to enable byte-accurate text extraction from token subsets)
///
/// By preserving raw tokens and their individual spans, we can later
/// pass them directly to existing AST constructors (using the same unified approach as the
/// reference parser), which handles all location tracking and AST node creation automatically.
///
/// Note: LineToken does NOT store an aggregate source_span. The AST construction facade
/// will compute bounding boxes from the individual token_spans when needed.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LineToken {
    /// The original raw tokens that comprise this line
    pub source_tokens: Vec<Token>,

    /// The byte range in source code for each token
    /// Must be the same length as source_tokens
    pub token_spans: Vec<std::ops::Range<usize>>,

    /// The type/classification of this line
    pub line_type: LineTokenType,
}

impl LineToken {
    /// Get source tokens as (Token, Range<usize>) pairs.
    ///
    /// This creates owned pairs from the separate source_tokens and token_spans vectors.
    /// Used by the AST construction facade to get tokens in the format expected by
    /// the token processing utilities.
    ///
    /// Note: LineToken stores tokens and spans separately for serialization efficiency.
    /// This method creates the paired format needed for location tracking.
    pub fn source_token_pairs(&self) -> Vec<(Token, std::ops::Range<usize>)> {
        self.source_tokens
            .iter()
            .zip(self.token_spans.iter())
            .map(|(token, span)| (token.clone(), span.clone()))
            .collect()
    }
}

/// The classification of a line token
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LineTokenType {
    /// Blank line (empty or whitespace only)
    BlankLine,

    /// Annotation end line: a line starting with :: marker and having no further content
    AnnotationEndLine,

    /// Annotation start line: follows annotation grammar <lex-marker><space>(<label><space>)?<parameters>? <lex-marker> <content>?
    AnnotationStartLine,

    /// Line ending with colon (could be subject/definition/session title)
    SubjectLine,

    /// Line starting with list marker (-, 1., a., I., etc.)
    ListLine,

    /// Line starting with list marker and ending with colon (subject and list item combined)
    SubjectOrListItemLine,

    /// Any other line (paragraph text)
    ParagraphLine,

    /// Indentation marker (pass-through from prior transformation)
    IndentLevel,

    /// Dedentation marker (pass-through from prior transformation)
    DedentLevel,
}

impl fmt::Display for LineTokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            LineTokenType::BlankLine => "BLANK_LINE",
            LineTokenType::AnnotationEndLine => "ANNOTATION_END_LINE",
            LineTokenType::AnnotationStartLine => "ANNOTATION_START_LINE",
            LineTokenType::SubjectLine => "SUBJECT_LINE",
            LineTokenType::ListLine => "LIST_LINE",
            LineTokenType::SubjectOrListItemLine => "SUBJECT_OR_LIST_ITEM_LINE",
            LineTokenType::ParagraphLine => "PARAGRAPH_LINE",
            LineTokenType::IndentLevel => "INDENT",
            LineTokenType::DedentLevel => "DEDENT",
        };
        write!(f, "{}", name)
    }
}

impl LineTokenType {
    /// Format token type as grammar notation: `<token-name>`
    ///
    /// Converts UPPER_CASE_WITH_UNDERSCORES to <lower-case-with-dashes>
    ///
    /// Examples:
    /// - BlankLine -> `<blank-line>`
    /// - AnnotationStartLine -> `<annotation-start-line>`
    /// - SubjectLine -> `<subject-line>`
    pub fn to_grammar_string(&self) -> String {
        let name = match self {
            LineTokenType::BlankLine => "blank-line",
            LineTokenType::AnnotationEndLine => "annotation-end-line",
            LineTokenType::AnnotationStartLine => "annotation-start-line",
            LineTokenType::SubjectLine => "subject-line",
            LineTokenType::ListLine => "list-line",
            LineTokenType::SubjectOrListItemLine => "subject-or-list-item-line",
            LineTokenType::ParagraphLine => "paragraph-line",
            LineTokenType::IndentLevel => "indent",
            LineTokenType::DedentLevel => "dedent",
        };
        format!("<{}>", name)
    }
}

/// The primary tree structure for the lexer output.
///
/// This is a recursive enum representing the complete hierarchical structure of line tokens.
/// Every node in the tree is either a line token or a container of child nodes.
///
/// The tree is built by processing IndentLevel/DedentLevel markers:
/// - Token variant: A single line token (e.g., SubjectLine, ParagraphLine, ListLine)
/// - Container variant: A grouped set of child nodes at a deeper indentation level
///
/// This structure allows the parser to match patterns by checking token types while
/// maintaining the complete source structure (source tokens, nesting).
///
/// Note: Container does NOT store an aggregate source_span. The AST construction facade
/// will compute bounding boxes by recursively unrolling children to their source tokens.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LineContainerToken {
    /// A single line token
    Token(LineToken),

    /// A container of child nodes (represents indented content or grouped lines at same level)
    Container { children: Vec<LineContainerToken> },
}

impl LineContainerToken {
    /// Check if this container is empty (only valid for root containers)
    pub fn is_empty(&self) -> bool {
        match self {
            LineContainerToken::Token(_) => false,
            LineContainerToken::Container { children, .. } => children.is_empty(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_type_to_grammar_string() {
        assert_eq!(LineTokenType::BlankLine.to_grammar_string(), "<blank-line>");
        assert_eq!(
            LineTokenType::AnnotationStartLine.to_grammar_string(),
            "<annotation-start-line>"
        );
        assert_eq!(
            LineTokenType::AnnotationEndLine.to_grammar_string(),
            "<annotation-end-line>"
        );
        assert_eq!(
            LineTokenType::SubjectLine.to_grammar_string(),
            "<subject-line>"
        );
        assert_eq!(LineTokenType::ListLine.to_grammar_string(), "<list-line>");
        assert_eq!(
            LineTokenType::SubjectOrListItemLine.to_grammar_string(),
            "<subject-or-list-item-line>"
        );
        assert_eq!(
            LineTokenType::ParagraphLine.to_grammar_string(),
            "<paragraph-line>"
        );
        assert_eq!(LineTokenType::IndentLevel.to_grammar_string(), "<indent>");
        assert_eq!(LineTokenType::DedentLevel.to_grammar_string(), "<dedent>");
    }

    #[test]
    fn test_token_sequence_formatting() {
        // Test creating a sequence of tokens and formatting them
        let tokens = [
            LineTokenType::SubjectLine,
            LineTokenType::IndentLevel,
            LineTokenType::ParagraphLine,
            LineTokenType::DedentLevel,
        ];

        let formatted = tokens
            .iter()
            .map(|t| t.to_grammar_string())
            .collect::<Vec<_>>()
            .join("");

        assert_eq!(formatted, "<subject-line><indent><paragraph-line><dedent>");
    }

    #[test]
    fn test_blank_line_group_formatting() {
        let tokens = [
            LineTokenType::BlankLine,
            LineTokenType::BlankLine,
            LineTokenType::BlankLine,
        ];

        let formatted = tokens
            .iter()
            .map(|t| t.to_grammar_string())
            .collect::<Vec<_>>()
            .join("");

        assert_eq!(formatted, "<blank-line><blank-line><blank-line>");
    }

    #[test]
    fn test_complex_pattern_formatting() {
        // Session pattern: blank + content + blank + container
        let tokens = [
            LineTokenType::BlankLine,
            LineTokenType::SubjectLine,
            LineTokenType::BlankLine,
            LineTokenType::IndentLevel,
            LineTokenType::ParagraphLine,
            LineTokenType::DedentLevel,
        ];

        let formatted = tokens
            .iter()
            .map(|t| t.to_grammar_string())
            .collect::<Vec<_>>()
            .join("");

        assert_eq!(
            formatted,
            "<blank-line><subject-line><blank-line><indent><paragraph-line><dedent>"
        );
    }

    #[test]
    fn test_line_token_source_token_pairs() {
        // Test that LineToken can provide source tokens in paired format
        let line_token = LineToken {
            source_tokens: vec![
                Token::Text("hello".to_string()),
                Token::Whitespace,
                Token::Text("world".to_string()),
            ],
            token_spans: vec![0..5, 5..6, 6..11],
            line_type: LineTokenType::ParagraphLine,
        };

        let pairs = line_token.source_token_pairs();
        assert_eq!(pairs.len(), 3);
        assert_eq!(pairs[0].1, 0..5);
        assert_eq!(pairs[1].1, 5..6);
        assert_eq!(pairs[2].1, 6..11);

        // Verify tokens match
        match &pairs[0].0 {
            Token::Text(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected Text token"),
        }
    }
}
