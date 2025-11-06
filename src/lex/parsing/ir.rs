//! Intermediate Representation for Parsers
//!
//! This module defines the Intermediate Representation (IR) that parsers produce.
//! The IR is a tree of `ParseNode`s, which describes the desired AST structure
//! without coupling the parser to the AST building logic.

use crate::lex::lexing::Token;
use std::ops::Range;

/// Type alias for token with location
pub type TokenLocation = (Token, Range<usize>);

/// The type of a node in the parse tree.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeType {
    Document,
    Paragraph,
    Session,
    ListItem,
    List,
    Definition,
    Annotation,
    VerbatimBlock,
    ForeignBlockSubject,
    ForeignBlockContent,
    ForeignBlockClosing,
}

/// A node in the parse tree.
#[derive(Debug, Clone)]
pub struct ParseNode {
    pub node_type: NodeType,
    pub tokens: Vec<TokenLocation>,
    pub children: Vec<ParseNode>,
}

impl ParseNode {
    /// Creates a new `ParseNode`.
    pub fn new(node_type: NodeType, tokens: Vec<TokenLocation>, children: Vec<ParseNode>) -> Self {
        Self {
            node_type,
            tokens,
            children,
        }
    }
}
