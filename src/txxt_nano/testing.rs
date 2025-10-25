//! Testing utilities for AST assertions
//!
//! ## Why This Exists
//!
//! Parser testing has a fundamental problem: ASTs get deeply nested and verbose,
//! making tests hard to read and maintain. Traditional approaches lead to either:
//!
//! - **Verbose manual matching**: 50+ lines of pattern matching and assertions
//! - **Weak testing**: Only checking counts and types, missing actual content
//!
//! This library solves that with a fluent API that mirrors the AST structure,
//! making tests both concise and thorough.
//!
//! ## The Problem
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
//! ## The Solution
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
//! ## Key Features
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
//! ## Usage Example
//!
//! ```rust,ignore
//! use crate::txxt_nano::testing::assert_ast;
//! use crate::txxt_nano::parser::parse_document;
//!
//! let doc = parse_document(r#"
//! Introduction
//!
//!     Welcome to the guide.
//!
//!     Getting Started
//!
//!         First, install the software.
//! "#).unwrap();
//!
//! assert_ast(&doc)
//!     .item_count(1)
//!     .item(0, |item| {
//!         item.assert_session()
//!             .label("Introduction")
//!             .child_count(2)
//!             .child(0, |child| {
//!                 child.assert_paragraph()
//!                     .text_contains("Welcome");
//!             })
//!             .child(1, |child| {
//!                 child.assert_session()
//!                     .label("Getting Started")
//!                     .child(0, |para| {
//!                         para.assert_paragraph()
//!                             .text_starts_with("First");
//!                     });
//!             });
//!     });
//! ```
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
