//! Inline AST nodes shared across formatting, literal, and reference elements.
//!
//! These nodes are intentionally lightweight so the inline parser can be used
//! from unit tests before it is integrated into the higher level AST builders.

/// Sequence of inline nodes produced from a [`TextContent`](crate::lex::ast::TextContent).
pub type InlineContent = Vec<InlineNode>;

/// Inline node variants supported by the initial flat inline parser.
#[derive(Debug, Clone, PartialEq)]
pub enum InlineNode {
    /// Plain text segment with no formatting.
    Plain(String),
    /// Strong emphasis delimited by `*`.
    Strong(InlineContent),
    /// Emphasis delimited by `_`.
    Emphasis(InlineContent),
    /// Inline code delimited by `` ` ``.
    Code(String),
    /// Simple math span delimited by `#`.
    Math(String),
    /// Reference enclosed by square brackets.
    Reference(ReferenceInline),
}

impl InlineNode {
    /// Returns the plain text from this node when available.
    pub fn as_plain(&self) -> Option<&str> {
        match self {
            InlineNode::Plain(text) => Some(text),
            InlineNode::Code(text) => Some(text),
            InlineNode::Math(text) => Some(text),
            _ => None,
        }
    }

    /// Returns nested inline content for container nodes (strong/emphasis).
    pub fn children(&self) -> Option<&InlineContent> {
        match self {
            InlineNode::Strong(children) | InlineNode::Emphasis(children) => Some(children),
            _ => None,
        }
    }

    /// Returns `true` when this node is plain text.
    pub fn is_plain(&self) -> bool {
        matches!(self, InlineNode::Plain(_))
    }
}

/// Reference inline node with raw content and classified type.
#[derive(Debug, Clone, PartialEq)]
pub struct ReferenceInline {
    pub raw: String,
    pub reference_type: ReferenceType,
}

impl ReferenceInline {
    pub fn new(raw: String) -> Self {
        Self {
            raw,
            reference_type: ReferenceType::NotSure,
        }
    }
}

/// Reference type classification derived from its content.
#[derive(Debug, Clone, PartialEq)]
pub enum ReferenceType {
    /// `[TK]` or `[TK-identifier]`
    ToCome { identifier: Option<String> },
    /// `[@citation]` with structured citation data.
    Citation(CitationData),
    /// `[^note]`
    FootnoteLabeled { label: String },
    /// `[12]`
    FootnoteNumber { number: u32 },
    /// `[#42]`
    Session { target: String },
    /// `[https://example.com]`
    Url { target: String },
    /// `[./file.txt]`
    File { target: String },
    /// `[Introduction]` or other document references.
    General { target: String },
    /// Unable to classify.
    NotSure,
}

/// Structured citation payload capturing parsed information.
#[derive(Debug, Clone, PartialEq)]
pub struct CitationData {
    pub keys: Vec<String>,
    pub locator: Option<CitationLocator>,
}

/// Citation locator derived from the `p.` / `pp.` segment.
#[derive(Debug, Clone, PartialEq)]
pub struct CitationLocator {
    pub format: PageFormat,
    pub ranges: Vec<PageRange>,
    /// Raw locator string as authored (e.g. `p.45-46`).
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PageFormat {
    P,
    Pp,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageRange {
    pub start: u32,
    pub end: Option<u32>,
}
