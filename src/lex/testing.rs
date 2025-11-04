//! Testing utilities for AST assertions
//!
//! # Parser Testing Guidelines
//!
//! Testing the parser must follow strict rules to ensure reliability and maintainability.
//! This module provides two essential tools that **must** be used together:
//!
//! 1. **[LexSources](crate::lex::processor::lex_sources::LexSources)** - For verified lex content
//! 2. **[assert_ast](fn@assert_ast)** - For comprehensive AST verification
//!
//! ## Rule 1: Always Use LexSources for Test Content
//!
//! **Why this matters:**
//!
//! lex is a novel format that's still evolving. People regularly get small details wrong,
//! leading to false positives in tests. When lex changes, we need to verify and update
//! all source files. If lex content is scattered across many test files, this becomes
//! a maintenance nightmare.
//!
//! **The solution:**
//!
//! Use the `LexSources` library to access verified, curated lex sample files.
//! This ensures only vetted sources are used and makes writing tests much easier.
//!
//! ```rust-example
//! use crate::lex::processor::lex_sources::LexSources;
//! use crate::lex::parsers::parse_document;
//!
//! // CORRECT: Use verified sample files
//! let source = LexSources::get_string("000-paragraphs.lex")?;
//! let doc = parse_document(&source)?;
//!
//! // WRONG: Don't write lex content directly in tests
//! let doc = parse_document("Some paragraph\n\nAnother paragraph\n\n")?;
//! ```
//!
//! **Available samples:**
//! - `000-paragraphs.lex` - Basic paragraph parsing
//! - `010-paragraphs-sessions-flat-single.lex` - Single session
//! - `050-paragraph-lists.lex` - Mixed content
//!
//! and many more.
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
//! This is also very hard to write, time-consuming, and when the lex spec changes,
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
//! ```rust-example
//! use crate::lex::ast::ContentItem;
//!
//! match &doc.content[0] {
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

//! ### The Solution: Fluent Assertion API

//! With the `assert_ast` fluent API, the same test becomes:

//! ```rust-example
//! use crate::lex::testing::assert_ast;
//!
//! assert_ast(&doc)
//!     .item(0, |item| {
//!         item.assert_session()
//!             .label("Introduction")
//!             .child_count(2)
//!             .child(0, |child| {
//!                 child.assert_paragraph()
//!                     .text_starts_with("Hello")
//!             })
//!     });
//! ```

//! Concise, readable, and maintainable.

//! ## Available Node Types

//! The assertion API supports all AST node types:
//! - `ParagraphAssertion` - Text content nodes
//! - `SessionAssertion` - Titled container nodes  
//! - `ListAssertion` / `ListItemAssertion` - List structures
//! - `DefinitionAssertion` - Subject-definition pairs
//! - `AnnotationAssertion` - Metadata with parameters
//! - `ForeignBlockAssertion` - Raw content blocks

//!   Each assertion type provides type-specific methods (e.g., `label()` for
//!   sessions, `subject()` for definitions, `parameter_count()` for annotations).

//! ## Extending the Assertion API

//! To add support for a new container node type:
//!
//! 1. **Implement the traits** in `ast.rs`:
//!    ```rust-example
//!    use crate::lex::ast::{Container, ContentItem};
//!
//!    struct NewNode { content: Vec<ContentItem>, label: String }
//!
//!    impl Container for NewNode {
//!        fn label(&self) -> &str { &self.label }
//!        fn children(&self) -> &[ContentItem] { &self.content }
//!        fn children_mut(&mut self) -> &mut Vec<ContentItem> { &mut self.content }
//!    }
//!    ```
//!
//! 2. **Add to ContentItem enum** and implement helper methods
//!
//! 3. **Add assertion type** in `testing_assertions.rs`:
//!    ```rust-example
//!    pub struct NewNodeAssertion<'a> { /* ... */ }
//!
//!    impl NewNodeAssertion<'_> {
//!        pub fn custom_field(self, expected: &str) -> Self { /* ... */ }
//!        pub fn child_count(self, expected: usize) -> Self { /* ... */ }
//!    }
//!    ```
//!
//! 4. **Add to ContentItemAssertion** and export in `testing.rs`:
//!    ```rust-example
//!    pub fn assert_new_node(self) -> NewNodeAssertion<'a> { /* ... */ }
//!    ```

mod testing_assertions;
mod testing_factories;
mod testing_matchers;

pub use testing_assertions::{
    assert_ast, AnnotationAssertion, ChildrenAssertion, ContentItemAssertion, DefinitionAssertion,
    DocumentAssertion, ForeignBlockAssertion, ListAssertion, ListItemAssertion, ParagraphAssertion,
    SessionAssertion,
};
pub use testing_matchers::TextMatch;

// Public submodule path: crate::lex::testing::factories
pub mod factories {
    pub use super::testing_factories::*;
}
