//! Line-based token types for the linebased lexer pipeline
//!
//! This module contains token types specific to the line-based lexer pipeline:
//! - LineToken: Represents a logical line created from grouped raw tokens
//! - LineTokenType: Classification of line types
//! - LineContainerToken: Hierarchical tree structure where nodes are either line tokens or nested containers

use std::fmt;

use crate::txxt::lexers::tokens::Token;

/// A line token represents one logical line created from grouped raw tokens.
///
/// Line tokens are produced by the linebased line token transformation,
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

    /// Annotation end line: a line starting with :: marker and having no further content
    AnnotationEndLine,

    /// Annotation start line: follows annotation grammar <txxt-marker><space>(<label><space>)?<parameters>? <txxt-marker> <content>?
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
/// maintaining the complete source structure (source spans, source tokens, nesting).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LineContainerToken {
    /// A single line token
    Token(LineToken),

    /// A container of child nodes (represents indented content or grouped lines at same level)
    ///
    /// children: The line tokens and nested containers at this level
    /// source_span: The byte range covering all children in this container
    Container {
        children: Vec<LineContainerToken>,
        source_span: Option<std::ops::Range<usize>>,
    },
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

/// TEMPORARY: Legacy token tree structure for parser compatibility.
///
/// This is kept for Delivery 1 to allow the parser to work unchanged while the lexer
/// outputs the new LineContainerToken structure. The unwrapper function converts
/// LineContainerToken to Vec<LineTokenTree> for the parser.
///
/// In Delivery 2, this will be removed and the parser will work directly with LineContainerToken.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LineContainerTokenLegacy {
    /// The line tokens that comprise this container (all at the same indentation level)
    pub source_tokens: Vec<LineToken>,

    /// The byte range in source code that this container spans
    /// Used for location tracking and mapping AST nodes back to source
    pub source_span: Option<std::ops::Range<usize>>,
}

/// TEMPORARY: Legacy token tree enum for parser compatibility.
///
/// This is kept for Delivery 1 to allow the parser to work unchanged.
/// In Delivery 2, this will be removed entirely.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LineTokenTree {
    /// A single line token
    Token(LineToken),

    /// A container of line tokens at the same indentation level
    Container(LineContainerTokenLegacy),

    /// A block of nested tokens (represents indented content)
    Block(Vec<LineTokenTree>),
}
