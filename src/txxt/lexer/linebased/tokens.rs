//! Line-based token types for the experimental lexer pipeline
//!
//! This module contains token types specific to the line-based lexer pipeline:
//! - LineToken: Represents a logical line created from grouped raw tokens
//! - LineTokenType: Classification of line types
//! - LineTokenTree: Hierarchical tree structure for indentation-based nesting

use std::fmt;

use crate::txxt::lexer::tokens::Token;

/// A line token represents one logical line created from grouped raw tokens.
///
/// Line tokens are produced by the experimental line token transformation,
/// which groups raw tokens into semantic line units. Each line token stores:
/// - The original raw tokens that created it (for location information and AST construction)
/// - The line type (what kind of line this is)
/// - The source span (byte range in source) for location tracking
///
/// By preserving raw tokens and source span, we can later pass them directly to existing AST constructors,
/// which handles all location tracking and AST node creation automatically.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LineToken {
    /// The original raw tokens that comprise this line
    pub source_tokens: Vec<Token>,

    /// The type/classification of this line
    pub line_type: LineTokenType,

    /// The byte range in source code that this line spans
    /// Used for location tracking and mapping AST nodes back to source
    pub source_span: Option<std::ops::Range<usize>>,
}

/// The classification of a line token
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LineTokenType {
    /// Blank line (empty or whitespace only)
    BlankLine,

    /// Line with :: markers (annotation)
    AnnotationLine,

    /// Line ending with colon (could be subject/definition/session title)
    SubjectLine,

    /// Line starting with list marker (-, 1., a., I., etc.)
    ListLine,

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
            LineTokenType::AnnotationLine => "ANNOTATION_LINE",
            LineTokenType::SubjectLine => "SUBJECT_LINE",
            LineTokenType::ListLine => "LIST_LINE",
            LineTokenType::ParagraphLine => "PARAGRAPH_LINE",
            LineTokenType::IndentLevel => "INDENT",
            LineTokenType::DedentLevel => "DEDENT",
        };
        write!(f, "{}", name)
    }
}

/// A tree node in the hierarchical token structure.
///
/// The tree is built by processing IndentLevel/DedentLevel markers:
/// - Token variant holds a single LineToken
/// - Block variant holds a vector of tree nodes (children at deeper indentation)
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LineTokenTree {
    /// A single line token
    Token(LineToken),

    /// A block of nested tokens (represents indented content)
    Block(Vec<LineTokenTree>),
}
