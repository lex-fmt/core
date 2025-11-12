//! Building stage pipeline helpers.
//!
//! Currently this is a thin wrapper around the `AstBuilder` that converts parse
//! nodes produced by the analyzers into the final `Document` AST.

use crate::lex::parsing::builder::AstBuilder;
use crate::lex::parsing::ir::ParseNode;
use crate::lex::parsing::Document;

/// Build a `Document` from a parsed root node.
pub fn build_document(root: ParseNode, source: &str) -> Document {
    AstBuilder::new(source).build(root)
}
