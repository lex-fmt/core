//! Building stage pipeline helpers.
//!
//! Currently this is a thin wrapper around the `AstTreeBuilder` that converts parse
//! nodes produced by the analyzers into the final `Document` AST.

use crate::lex::building::ast_tree::AstTreeBuilder;
use crate::lex::parsing::ir::ParseNode;
use crate::lex::parsing::Document;

/// Build a `Document` from a parsed root node.
pub fn build_document(root: ParseNode, source: &str) -> Document {
    AstTreeBuilder::new(source).build(root)
}
