//! Testing utilities for AST assertions
//!
//! # Parser Testing Guidelines
//!
//! Testing the parser must follow strict rules to ensure reliability and maintainability.
//! This module provides two essential tools that **must** be used together:
//!
//! 1. **[Lexplore](crate::lex::testing::lexplore::Lexplore)** - For verified lex content
//! 2. **[assert_ast](fn@assert_ast)** - For comprehensive AST verification
//!
//! ## Rule 1: Always Use Lexplore for Test Content
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
//! Use the `Lexplore` library to access verified, curated lex sample files.
//! This ensures only vetted sources are used and makes writing tests much easier.
//!
//! ```rust-example
//! use crate::lex::testing::lexplore::{Lexplore, ElementType, DocumentType};
//!
//! // CORRECT: Use verified sample files
//! let source = Lexplore::get_source_for(ElementType::Paragraph, 1)?;
//! let doc = parse_document(&source)?;
//!
//! // OR use the fluent API
//! let parsed = Lexplore::paragraph(1).parse();
//! let paragraph = parsed.root.expect_paragraph();
//!
//! // WRONG: Don't write lex content directly in tests
//! let doc = parse_document("Some paragraph\n\nAnother paragraph\n\n")?;
//! ```
//!
//! **Available sources:**
//! - Elements: `Lexplore::get_source_for(ElementType::Paragraph, 1)` - Individual elements
//! - Documents: `Lexplore::get_document_source_for(DocumentType::Trifecta, 0)` - Full documents
//! - Fluent API: `Lexplore::paragraph(1)`, `Lexplore::list(1)`, etc.
//!
//! See the [Lexplore documentation](crate::lex::testing::lexplore) for more details.
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
//!         assert_eq!(s.children.len(), 2);
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
//! - `VerbatimBlockkAssertion` - Raw content blocks

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

mod ast_assertions;
pub mod lexplore;
mod matchers;

pub use ast_assertions::{
    assert_ast, AnnotationAssertion, ChildrenAssertion, ContentItemAssertion, DefinitionAssertion,
    DocumentAssertion, InlineAssertion, InlineExpectation, ListAssertion, ListItemAssertion,
    ParagraphAssertion, ReferenceExpectation, SessionAssertion, VerbatimBlockkAssertion,
};
pub use matchers::TextMatch;

// Public submodule path: crate::lex::testing::factories
pub mod factories {
    pub use crate::lex::token::testing::*;
}

/// Get a path relative to the workspace root for testing purposes.
///
/// In a workspace, `CARGO_MANIFEST_DIR` points to the crate directory (lex-parser/),
/// so we need to go up one level to reach the workspace root where docs/, fixtures/, etc. live.
///
/// # Example
/// ```rust,ignore
/// let path = workspace_path("docs/specs/v1/elements/paragraph/01-simple.lex");
/// let content = std::fs::read_to_string(path).unwrap();
/// ```
pub fn workspace_path(relative_path: &str) -> std::path::PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = std::path::Path::new(manifest_dir).parent().unwrap();
    workspace_root.join(relative_path)
}

/// Parse a Lex document without running the annotation attachment stage.
///
/// This is useful for tests that need annotations to remain in the content tree
/// rather than being attached as metadata. Common use cases:
/// - Testing annotation parsing in isolation
/// - Testing the attachment logic itself
/// - Element tests that expect annotations as content items
///
/// # Example
/// ```rust,ignore
/// use crate::lex::testing::parse_without_annotation_attachment;
///
/// let source = ":: note ::\nSome paragraph\n";
/// let doc = parse_without_annotation_attachment(source).unwrap();
///
/// // Annotation is still in content tree, not attached as metadata
/// assert!(doc.root.children.iter().any(|item| matches!(item, ContentItem::Annotation(_))));
/// ```
pub fn parse_without_annotation_attachment(
    source: &str,
) -> Result<crate::lex::ast::Document, String> {
    use crate::lex::parsing::engine::parse_from_flat_tokens;
    use crate::lex::transforms::standard::LEXING;

    let source = if !source.is_empty() && !source.ends_with('\n') {
        format!("{}\n", source)
    } else {
        source.to_string()
    };
    let tokens = LEXING.run(source.clone()).map_err(|e| e.to_string())?;
    parse_from_flat_tokens(tokens, &source).map_err(|e| e.to_string())
}
