//! Testing utilities for AST assertions
//!
//! This module provides a fluent API for asserting on AST structures,
//! making parser tests more readable and maintainable.
//!
//! # Example
//!
//! ```rust
//! use txxt_nano::testing::assert_ast;
//!
//! let doc = parse_document("Hello world\n\n").unwrap();
//!
//! assert_ast(&doc)
//!     .item_count(1)
//!     .item(0, |item| {
//!         item.assert_paragraph()
//!             .text("Hello world")
//!     });
//! ```

mod assertions;
mod matchers;

pub use assertions::{
    assert_ast, ChildrenAssertion, ContentItemAssertion, DocumentAssertion, ParagraphAssertion,
    SessionAssertion,
};
pub use matchers::TextMatch;
