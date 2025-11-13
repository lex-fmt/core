//! AST Assertion Modules
//!
//! This module contains element-specific assertion types organized by element type.

mod annotation;
mod children;
mod definition;
mod document;
mod list;
mod paragraph;
mod session;
mod verbatim;

pub use annotation::AnnotationAssertion;
pub use children::ChildrenAssertion;
pub use definition::DefinitionAssertion;
pub use document::DocumentAssertion;
pub use list::{ListAssertion, ListItemAssertion};
pub use paragraph::ParagraphAssertion;
pub use session::SessionAssertion;
pub use verbatim::VerbatimBlockkAssertion;

use crate::lex::ast::traits::AstNode;
use crate::lex::ast::ContentItem;

// ============================================================================
// Helper Functions (shared across modules)
// ============================================================================

pub(super) fn summarize_items(items: &[ContentItem]) -> String {
    items
        .iter()
        .map(|item| item.node_type())
        .collect::<Vec<_>>()
        .join(", ")
}
