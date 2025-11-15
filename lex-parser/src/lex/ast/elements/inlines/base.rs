//! Inline AST nodes shared across formatting, literal, and reference elements.
//!
//! These nodes are intentionally lightweight so the inline parser can be used
//! from unit tests before it is integrated into the higher level AST builders.

use super::references::ReferenceInline;

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
