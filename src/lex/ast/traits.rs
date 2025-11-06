//! AST traits - Common interfaces for uniform node access
//!
//! This module defines the common traits that provide uniform access
//! to AST node information across all node types.

use super::elements::ContentItem;
use super::elements::VerbatimLine;
use super::range::{Position, Range};
use super::text_content::TextContent;

/// Visitor trait for traversing the AST
///
/// Implement this trait to walk the AST. Each visit method corresponds to a node type.
/// Default implementations are empty, so you only need to override the methods you care about.
///
/// # Example
///
/// ```ignore
/// struct MyVisitor;
///
/// impl Visitor for MyVisitor {
///     fn visit_paragraph(&mut self, para: &Paragraph) {
///         println!("Found paragraph: {}", para.text());
///     }
/// }
///
/// let mut visitor = MyVisitor;
/// document.accept(&mut visitor);
/// ```
pub trait Visitor {
    // Container nodes with labels and children
    fn visit_session(&mut self, _session: &super::Session) {}
    fn visit_definition(&mut self, _definition: &super::Definition) {}
    fn visit_list(&mut self, _list: &super::List) {}
    fn visit_list_item(&mut self, _list_item: &super::ListItem) {}

    // Leaf nodes
    fn visit_paragraph(&mut self, _paragraph: &super::Paragraph) {}
    fn visit_text_line(&mut self, _text_line: &super::elements::paragraph::TextLine) {}
    fn visit_foreign_block(&mut self, _foreign_block: &super::Verbatim) {}
    fn visit_foreign_line(&mut self, _foreign_line: &VerbatimLine) {}
    fn visit_annotation(&mut self, _annotation: &super::Annotation) {}
    fn visit_blank_line_group(
        &mut self,
        _blank_line_group: &super::elements::blank_line_group::BlankLineGroup,
    ) {
    }
}

/// Helper function to visit all children in a ContentItem slice
pub fn visit_children(visitor: &mut dyn Visitor, items: &[ContentItem]) {
    for item in items {
        item.accept(visitor);
    }
}

/// Common interface for all AST nodes
pub trait AstNode {
    fn node_type(&self) -> &'static str;
    fn display_label(&self) -> String;
    fn range(&self) -> &Range;
    fn start_position(&self) -> Position {
        self.range().start
    }

    /// Accept a visitor for traversing this node and its children
    fn accept(&self, visitor: &mut dyn Visitor);
}

/// Trait for container nodes that have a label and children
pub trait Container: AstNode {
    fn label(&self) -> &str;
    fn children(&self) -> &[ContentItem];
    fn children_mut(&mut self) -> &mut Vec<ContentItem>;
}

/// Trait for leaf nodes that contain text
pub trait TextNode: AstNode {
    fn text(&self) -> String;
    fn lines(&self) -> &[TextContent];
}

#[cfg(test)]
mod tests {
    use super::super::elements::{Paragraph, Session};
    use super::*;

    #[test]
    fn test_visitor_traversal() {
        // Create a simple structure: Session with a Paragraph
        let para = Paragraph::from_line("Hello, World!".to_string());
        let session = Session::with_title("Test Session".to_string());

        // Create a visitor that counts nodes
        struct CountingVisitor {
            session_count: usize,
            paragraph_count: usize,
            text_line_count: usize,
        }

        impl Visitor for CountingVisitor {
            fn visit_session(&mut self, _: &super::super::Session) {
                self.session_count += 1;
            }
            fn visit_paragraph(&mut self, _: &super::super::Paragraph) {
                self.paragraph_count += 1;
            }
            fn visit_text_line(&mut self, _: &super::super::elements::paragraph::TextLine) {
                self.text_line_count += 1;
            }
        }

        let mut visitor = CountingVisitor {
            session_count: 0,
            paragraph_count: 0,
            text_line_count: 0,
        };

        // Visit the paragraph
        para.accept(&mut visitor);
        assert_eq!(visitor.paragraph_count, 1);
        assert_eq!(visitor.text_line_count, 1); // Paragraph contains one TextLine
        assert_eq!(visitor.session_count, 0);

        // Reset and visit the session
        visitor.session_count = 0;
        visitor.paragraph_count = 0;
        visitor.text_line_count = 0;
        session.accept(&mut visitor);
        assert_eq!(visitor.session_count, 1);
        assert_eq!(visitor.paragraph_count, 0); // Session has no children yet
    }
}
