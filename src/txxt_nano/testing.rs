//! Testing utilities for AST assertions
//!
//! # Parser Testing Guidelines
//!
//! Testing the parser must follow strict rules to ensure reliability and maintainability.
//! This module provides two essential tools that **must** be used together:
//!
//! 1. **[TxxtSources](crate::txxt_nano::processor::txxt_sources::TxxtSources)** - For verified txxt content
//! 2. **[assert_ast](fn@assert_ast)** - For comprehensive AST verification
//!
//! ## Rule 1: Always Use TxxtSources for Test Content
//!
//! **Why this matters:**
//!
//! txxt is a novel format that's still evolving. People regularly get small details wrong,
//! leading to false positives in tests. When txxt changes, we need to verify and update
//! all source files. If txxt content is scattered across many test files, this becomes
//! a maintenance nightmare.
//!
//! **The solution:**
//!
//! Use the `TxxtSources` library to access verified, curated txxt sample files.
//! This ensures only vetted sources are used and makes writing tests much easier.
//!
//! ```rust,ignore
//! use crate::txxt_nano::processor::txxt_sources::TxxtSources;
//! use crate::txxt_nano::parser::parse_document;
//!
//! // ✅ CORRECT: Use verified sample files
//! let source = TxxtSources::get_string("000-paragraphs.txxt")?;
//! let doc = parse_document(&source)?;
//!
//! // ❌ WRONG: Don't write txxt content directly in tests
//! let doc = parse_document("Some paragraph\n\nAnother paragraph\n\n")?;
//! ```
//!
//! **Available samples:**
//! - `000-paragraphs.txxt` - Basic paragraph parsing
//! - `010-paragraphs-sessions-flat-single.txxt` - Single session
//! - `020-paragraphs-sessions-flat-multiple.txxt` - Multiple sessions
//! - `030-paragraphs-sessions-nested-multiple.txxt` - Nested sessions
//! - `040-lists.txxt` - List structures
//! - `050-paragraph-lists.txxt` - Mixed content
//!
//! ## Rule 2: Always Use assert_ast for AST Verification
//!
//! **Why this matters:**
//!
//! What we want for every document test is to ensure that the AST shape is correct
//! per the grammar, that all attributes are correct (children, content, etc.).
//! Asserting generalities like node counts is useless - it's not informative.
//! We want assurance on the AST shape and content.
//!
//! This is also very hard to write, time-consuming, and when the txxt spec changes,
//! very hard to update.
//!
//! **The solution:**
//!
//! Use the `assert_ast` library with its fluent API. It allows testing entire
//! hierarchies of nodes at once with 10-20x less code.
//!
//! ### The Problem with Manual Testing
//!
//! Testing a nested session traditionally looks like this:
//!
//! ```rust,ignore
//! match &doc.items[0] {
//!     ContentItem::Session(s) => {
//!         assert_eq!(s.title, "Introduction");
//!         assert_eq!(s.content.len(), 2);
//!         match &s.content[0] {
//!             ContentItem::Paragraph(p) => {
//!                 assert_eq!(p.lines.len(), 1);
//!                 assert!(p.lines[0].starts_with("Hello"));
//!             }
//!             _ => panic!("Expected paragraph"),
//!         }
//!         // ... repeat for second child
//!     }
//!     _ => panic!("Expected session"),
//! }
//! ```
//!
//! 20+ lines of boilerplate. Hard to see what's actually being tested.
//!
//! ### The Solution with assert_ast
//!
//! The same test with our fluent API:
//!
//! ```rust,ignore
//! use txxt_nano::testing::assert_ast;
//!
//! assert_ast(&doc).item(0, |item| {
//!     item.assert_session()
//!         .label("Introduction")
//!         .child_count(2)
//!         .child(0, |child| {
//!             child.assert_paragraph()
//!                 .text_starts_with("Hello");
//!         });
//! });
//! ```
//!
//! 7 lines. The structure mirrors the AST. Clear what's being tested.
//!
//! ## Complete Testing Example
//!
//! Here's how to write a proper parser test using both required tools:
//!
//! ```rust,ignore
//! use crate::txxt_nano::processor::txxt_sources::TxxtSources;
//! use crate::txxt_nano::parser::parse_document;
//! use crate::txxt_nano::testing::assert_ast;
//!
//! #[test]
//! fn test_nested_sessions() -> Result<(), Box<dyn std::error::Error>> {
//!     // Rule 1: Use verified txxt sources
//!     let source = TxxtSources::get_string("030-paragraphs-sessions-nested-multiple.txxt")?;
//!     let doc = parse_document(&source)?;
//!
//!     // Rule 2: Use assert_ast for comprehensive verification
//!     assert_ast(&doc)
//!         .item_count(5)
//!         .item(0, |item| {
//!             item.assert_paragraph()
//!                 .text_contains("Nested Sessions Test");
//!         })
//!         .item(2, |item| {
//!             item.assert_session()
//!                 .label("1. Root Session")
//!                 .child_count(4)
//!                 .child(1, |child| {
//!                     child.assert_session()
//!                         .label("1.1. First Sub-session")
//!                         .child_count(2)
//!                         .child(0, |para| {
//!                             para.assert_paragraph()
//!                                 .text_contains("This is content");
//!                         });
//!                 });
//!         });
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Key Features of assert_ast
//!
//! **Type-safe**: Compiler catches mismatched assertions. Can't call `.label()` on a paragraph.
//!
//! **Clear errors**: Shows exactly what failed with context:
//! ```text
//! items[2]:children[1]: Expected text to start with 'Hello', but got 'Welcome'
//! ```
//!
//! **Smart summaries**: Count mismatches show actual structure:
//! ```text
//! Expected 3 children, found 4: [Paragraph, Paragraph, Session, Paragraph]
//! ```
//!
//! **Fluent API**: Mirrors the AST structure, making tests readable and maintainable.
//!
//! ## Adding New AST Nodes
//!
//! When you add a new container node (ListItem, Definition, etc.):
//!
//! 1. **Implement the traits** in `ast.rs`:
//!    ```rust,ignore
//!    impl Container for ListItem {
//!        fn label(&self) -> &str { &self.item_line }
//!        fn children(&self) -> &[ContentItem] { &self.content }
//!        fn children_mut(&mut self) -> &mut Vec<ContentItem> { &mut self.content }
//!    }
//!    ```
//!
//! 2. **Add to ContentItem enum**: Update the enum and its helper methods
//!
//! 3. **Add assertion type** in `testing/assertions.rs`:
//!    ```rust,ignore
//!    pub struct ListItemAssertion<'a> { /* ... */ }
//!    impl ListItemAssertion {
//!        pub fn item_line(self, expected: &str) -> Self { /* ... */ }
//!        pub fn child_count(self, expected: usize) -> Self { /* ... */ }
//!    }
//!    ```
//!
//! 4. **Add to ContentItemAssertion**:
//!    ```rust,ignore
//!    pub fn assert_list_item(self) -> ListItemAssertion<'a> { /* ... */ }
//!    ```
//!
//! That's it! The fluent API automatically works with the new node type.

mod testing_assertions;
mod testing_matchers;

pub use testing_assertions::{
    assert_ast, ChildrenAssertion, ContentItemAssertion, DocumentAssertion, ParagraphAssertion,
    SessionAssertion,
};
pub use testing_matchers::TextMatch;
