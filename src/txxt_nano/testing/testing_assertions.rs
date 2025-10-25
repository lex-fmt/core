//! Fluent assertion API for AST nodes

use super::testing_matchers::TextMatch;
use crate::txxt_nano::parser::ast::{
    Container, ContentItem, Document, List, ListItem, Paragraph, Session,
};

// ============================================================================
// Entry Point
// ============================================================================

/// Create an assertion builder for a document
pub fn assert_ast(doc: &Document) -> DocumentAssertion<'_> {
    DocumentAssertion { doc }
}

// ============================================================================
// Document Assertions
// ============================================================================

pub struct DocumentAssertion<'a> {
    doc: &'a Document,
}

impl<'a> DocumentAssertion<'a> {
    /// Assert the number of items in the document
    pub fn item_count(self, expected: usize) -> Self {
        let actual = self.doc.items.len();
        assert_eq!(
            actual,
            expected,
            "Expected {} items, found {} items: [{}]",
            expected,
            actual,
            summarize_items(&self.doc.items)
        );
        self
    }

    /// Assert on a specific item by index
    pub fn item<F>(self, index: usize, assertion: F) -> Self
    where
        F: FnOnce(ContentItemAssertion<'a>),
    {
        assert!(
            index < self.doc.items.len(),
            "Item index {} out of bounds (document has {} items)",
            index,
            self.doc.items.len()
        );

        let item = &self.doc.items[index];
        assertion(ContentItemAssertion {
            item,
            context: format!("items[{}]", index),
        });
        self
    }
}

// ============================================================================
// ContentItem Assertions
// ============================================================================

pub struct ContentItemAssertion<'a> {
    item: &'a ContentItem,
    context: String,
}

impl<'a> ContentItemAssertion<'a> {
    /// Assert this item is a Paragraph and return paragraph-specific assertions
    pub fn assert_paragraph(self) -> ParagraphAssertion<'a> {
        match self.item {
            ContentItem::Paragraph(p) => ParagraphAssertion {
                para: p,
                context: self.context,
            },
            ContentItem::Session(s) => panic!(
                "{}: Expected Paragraph, found Session with label '{}'",
                self.context,
                s.label()
            ),
            ContentItem::List(l) => panic!(
                "{}: Expected Paragraph, found List with {} items",
                self.context,
                l.items.len()
            ),
        }
    }

    /// Assert this item is a Session and return session-specific assertions
    pub fn assert_session(self) -> SessionAssertion<'a> {
        match self.item {
            ContentItem::Session(s) => SessionAssertion {
                session: s,
                context: self.context,
            },
            ContentItem::Paragraph(p) => {
                let text = p.text();
                let display_text = if text.len() > 50 {
                    format!("{}...", &text[..50])
                } else {
                    text
                };
                panic!(
                    "{}: Expected Session, found Paragraph with text '{}'",
                    self.context, display_text
                )
            }
            ContentItem::List(l) => panic!(
                "{}: Expected Session, found List with {} items",
                self.context,
                l.items.len()
            ),
        }
    }

    /// Assert this item is a List and return list-specific assertions
    pub fn assert_list(self) -> ListAssertion<'a> {
        match self.item {
            ContentItem::List(l) => ListAssertion {
                list: l,
                context: self.context,
            },
            ContentItem::Paragraph(p) => {
                let text = p.text();
                let display_text = if text.len() > 50 {
                    format!("{}...", &text[..50])
                } else {
                    text
                };
                panic!(
                    "{}: Expected List, found Paragraph with text '{}'",
                    self.context, display_text
                )
            }
            ContentItem::Session(s) => panic!(
                "{}: Expected List, found Session with label '{}'",
                self.context,
                s.label()
            ),
        }
    }

    /// Check if this item is a paragraph (non-panicking)
    pub fn is_paragraph(&self) -> bool {
        matches!(self.item, ContentItem::Paragraph(_))
    }

    /// Check if this item is a session (non-panicking)
    pub fn is_session(&self) -> bool {
        matches!(self.item, ContentItem::Session(_))
    }

    /// Check if this item is a list (non-panicking)
    pub fn is_list(&self) -> bool {
        matches!(self.item, ContentItem::List(_))
    }
}

// ============================================================================
// Paragraph Assertions
// ============================================================================

pub struct ParagraphAssertion<'a> {
    para: &'a Paragraph,
    context: String,
}

impl<'a> ParagraphAssertion<'a> {
    /// Assert exact text match
    pub fn text(self, expected: &str) -> Self {
        TextMatch::Exact(expected.to_string()).assert(&self.para.text(), &self.context);
        self
    }

    /// Assert text starts with prefix
    pub fn text_starts_with(self, prefix: &str) -> Self {
        TextMatch::StartsWith(prefix.to_string()).assert(&self.para.text(), &self.context);
        self
    }

    /// Assert text contains substring
    pub fn text_contains(self, substring: &str) -> Self {
        TextMatch::Contains(substring.to_string()).assert(&self.para.text(), &self.context);
        self
    }

    /// Assert the number of lines in the paragraph
    pub fn line_count(self, expected: usize) -> Self {
        let actual = self.para.lines.len();
        assert_eq!(
            actual, expected,
            "{}: Expected {} lines, found {} lines",
            self.context, expected, actual
        );
        self
    }
}

// ============================================================================
// Session Assertions
// ============================================================================

pub struct SessionAssertion<'a> {
    session: &'a Session,
    context: String,
}

impl<'a> SessionAssertion<'a> {
    /// Assert exact label match
    pub fn label(self, expected: &str) -> Self {
        let actual = self.session.label();
        assert_eq!(
            actual, expected,
            "{}: Expected session label to be '{}', but got '{}'",
            self.context, expected, actual
        );
        self
    }

    /// Assert label starts with prefix
    pub fn label_starts_with(self, prefix: &str) -> Self {
        let actual = self.session.label();
        assert!(
            actual.starts_with(prefix),
            "{}: Expected session label to start with '{}', but got '{}'",
            self.context,
            prefix,
            actual
        );
        self
    }

    /// Assert label contains substring
    pub fn label_contains(self, substring: &str) -> Self {
        let actual = self.session.label();
        assert!(
            actual.contains(substring),
            "{}: Expected session label to contain '{}', but got '{}'",
            self.context,
            substring,
            actual
        );
        self
    }

    /// Assert the number of children
    pub fn child_count(self, expected: usize) -> Self {
        let actual = self.session.children().len();
        assert_eq!(
            actual,
            expected,
            "{}: Expected {} children, found {} children: [{}]",
            self.context,
            expected,
            actual,
            summarize_items(self.session.children())
        );
        self
    }

    /// Assert on a specific child by index
    pub fn child<F>(self, index: usize, assertion: F) -> Self
    where
        F: FnOnce(ContentItemAssertion<'a>),
    {
        let children = self.session.children();
        assert!(
            index < children.len(),
            "{}: Child index {} out of bounds (session has {} children)",
            self.context,
            index,
            children.len()
        );

        let child = &children[index];
        assertion(ContentItemAssertion {
            item: child,
            context: format!("{}:children[{}]", self.context, index),
        });
        self
    }

    /// Assert on all children using a builder
    pub fn children<F>(self, assertion: F) -> Self
    where
        F: FnOnce(ChildrenAssertion<'a>),
    {
        assertion(ChildrenAssertion {
            children: self.session.children(),
            context: format!("{}:children", self.context),
        });
        self
    }
}

// ============================================================================
// List Assertions
// ============================================================================

pub struct ListAssertion<'a> {
    list: &'a List,
    context: String,
}

impl<'a> ListAssertion<'a> {
    /// Assert the number of items in the list
    pub fn item_count(self, expected: usize) -> Self {
        let actual = self.list.items.len();
        assert_eq!(
            actual, expected,
            "{}: Expected {} list items, found {} list items",
            self.context, expected, actual
        );
        self
    }

    /// Assert on a specific list item by index
    pub fn item<F>(self, index: usize, assertion: F) -> Self
    where
        F: FnOnce(ListItemAssertion<'a>),
    {
        assert!(
            index < self.list.items.len(),
            "{}: Item index {} out of bounds (list has {} items)",
            self.context,
            index,
            self.list.items.len()
        );

        let item = &self.list.items[index];
        assertion(ListItemAssertion {
            item,
            context: format!("{}:items[{}]", self.context, index),
        });
        self
    }
}

// ============================================================================
// ListItem Assertions
// ============================================================================

pub struct ListItemAssertion<'a> {
    item: &'a ListItem,
    context: String,
}

impl<'a> ListItemAssertion<'a> {
    /// Assert exact text match
    pub fn text(self, expected: &str) -> Self {
        TextMatch::Exact(expected.to_string()).assert(self.item.text(), &self.context);
        self
    }

    /// Assert text starts with prefix
    pub fn text_starts_with(self, prefix: &str) -> Self {
        TextMatch::StartsWith(prefix.to_string()).assert(self.item.text(), &self.context);
        self
    }

    /// Assert text contains substring
    pub fn text_contains(self, substring: &str) -> Self {
        TextMatch::Contains(substring.to_string()).assert(self.item.text(), &self.context);
        self
    }

    /// Assert the number of children (nested content)
    pub fn child_count(self, expected: usize) -> Self {
        let actual = self.item.children().len();
        assert_eq!(
            actual,
            expected,
            "{}: Expected {} children, found {} children: [{}]",
            self.context,
            expected,
            actual,
            summarize_items(self.item.children())
        );
        self
    }

    /// Assert on a specific child by index
    pub fn child<F>(self, index: usize, assertion: F) -> Self
    where
        F: FnOnce(ContentItemAssertion<'a>),
    {
        let children = self.item.children();
        assert!(
            index < children.len(),
            "{}: Child index {} out of bounds (list item has {} children)",
            self.context,
            index,
            children.len()
        );

        let child = &children[index];
        assertion(ContentItemAssertion {
            item: child,
            context: format!("{}:children[{}]", self.context, index),
        });
        self
    }

    /// Assert on all children using a builder
    pub fn children<F>(self, assertion: F) -> Self
    where
        F: FnOnce(ChildrenAssertion<'a>),
    {
        assertion(ChildrenAssertion {
            children: self.item.children(),
            context: format!("{}:children", self.context),
        });
        self
    }
}

// ============================================================================
// Children Assertions (bulk operations)
// ============================================================================

pub struct ChildrenAssertion<'a> {
    children: &'a [ContentItem],
    context: String,
}

impl<'a> ChildrenAssertion<'a> {
    /// Assert the number of children
    pub fn count(self, expected: usize) -> Self {
        let actual = self.children.len();
        assert_eq!(
            actual,
            expected,
            "{}: Expected {} children, found {} children: [{}]",
            self.context,
            expected,
            actual,
            summarize_items(self.children)
        );
        self
    }

    /// Assert on a specific child by index
    pub fn item<F>(self, index: usize, assertion: F) -> Self
    where
        F: FnOnce(ContentItemAssertion<'a>),
    {
        assert!(
            index < self.children.len(),
            "{}: Child index {} out of bounds ({} children)",
            self.context,
            index,
            self.children.len()
        );

        let child = &self.children[index];
        assertion(ContentItemAssertion {
            item: child,
            context: format!("{}[{}]", self.context, index),
        });
        self
    }

    /// Assert all children are paragraphs
    pub fn all_paragraphs(self) -> Self {
        for (i, child) in self.children.iter().enumerate() {
            assert!(
                matches!(child, ContentItem::Paragraph(_)),
                "{}[{}]: Expected Paragraph, found {}",
                self.context,
                i,
                child.node_type()
            );
        }
        self
    }

    /// Assert all children are sessions
    pub fn all_sessions(self) -> Self {
        for (i, child) in self.children.iter().enumerate() {
            assert!(
                matches!(child, ContentItem::Session(_)),
                "{}[{}]: Expected Session, found {}",
                self.context,
                i,
                child.node_type()
            );
        }
        self
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Summarize items as "[Paragraph, Session, Paragraph]"
fn summarize_items(items: &[ContentItem]) -> String {
    items
        .iter()
        .map(|item| item.node_type())
        .collect::<Vec<_>>()
        .join(", ")
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::txxt_nano::parser::ast::{Document, Paragraph, Session};

    #[test]
    fn test_document_item_count() {
        let doc = Document::with_items(vec![
            ContentItem::Paragraph(Paragraph::from_line("Para 1".to_string())),
            ContentItem::Paragraph(Paragraph::from_line("Para 2".to_string())),
        ]);

        assert_ast(&doc).item_count(2);
    }

    #[test]
    #[should_panic(expected = "Expected 1 items, found 2 items: [Paragraph, Paragraph]")]
    fn test_document_item_count_failure() {
        let doc = Document::with_items(vec![
            ContentItem::Paragraph(Paragraph::from_line("Para 1".to_string())),
            ContentItem::Paragraph(Paragraph::from_line("Para 2".to_string())),
        ]);

        assert_ast(&doc).item_count(1);
    }

    #[test]
    fn test_paragraph_text() {
        let doc = Document::with_items(vec![ContentItem::Paragraph(Paragraph::from_line(
            "Hello world".to_string(),
        ))]);

        assert_ast(&doc).item(0, |item| {
            item.assert_paragraph().text("Hello world");
        });
    }

    #[test]
    #[should_panic(expected = "items[0]: Expected text to be 'Goodbye', but got 'Hello world'")]
    fn test_paragraph_text_failure() {
        let doc = Document::with_items(vec![ContentItem::Paragraph(Paragraph::from_line(
            "Hello world".to_string(),
        ))]);

        assert_ast(&doc).item(0, |item| {
            item.assert_paragraph().text("Goodbye");
        });
    }

    #[test]
    fn test_paragraph_text_starts_with() {
        let doc = Document::with_items(vec![ContentItem::Paragraph(Paragraph::from_line(
            "Hello world".to_string(),
        ))]);

        assert_ast(&doc).item(0, |item| {
            item.assert_paragraph().text_starts_with("Hello");
        });
    }

    #[test]
    fn test_paragraph_text_contains() {
        let doc = Document::with_items(vec![ContentItem::Paragraph(Paragraph::from_line(
            "Hello world".to_string(),
        ))]);

        assert_ast(&doc).item(0, |item| {
            item.assert_paragraph().text_contains("world");
        });
    }

    #[test]
    fn test_session_label() {
        let doc = Document::with_items(vec![ContentItem::Session(Session::with_title(
            "Introduction".to_string(),
        ))]);

        assert_ast(&doc).item(0, |item| {
            item.assert_session().label("Introduction");
        });
    }

    #[test]
    #[should_panic(
        expected = "items[0]: Expected session label to be 'Conclusion', but got 'Introduction'"
    )]
    fn test_session_label_failure() {
        let doc = Document::with_items(vec![ContentItem::Session(Session::with_title(
            "Introduction".to_string(),
        ))]);

        assert_ast(&doc).item(0, |item| {
            item.assert_session().label("Conclusion");
        });
    }

    #[test]
    fn test_session_child_count() {
        let doc = Document::with_items(vec![ContentItem::Session(Session::new(
            "Section".to_string(),
            vec![
                ContentItem::Paragraph(Paragraph::from_line("Para 1".to_string())),
                ContentItem::Paragraph(Paragraph::from_line("Para 2".to_string())),
            ],
        ))]);

        assert_ast(&doc).item(0, |item| {
            item.assert_session().child_count(2);
        });
    }

    #[test]
    #[should_panic(
        expected = "items[0]: Expected 3 children, found 2 children: [Paragraph, Paragraph]"
    )]
    fn test_session_child_count_failure() {
        let doc = Document::with_items(vec![ContentItem::Session(Session::new(
            "Section".to_string(),
            vec![
                ContentItem::Paragraph(Paragraph::from_line("Para 1".to_string())),
                ContentItem::Paragraph(Paragraph::from_line("Para 2".to_string())),
            ],
        ))]);

        assert_ast(&doc).item(0, |item| {
            item.assert_session().child_count(3);
        });
    }

    #[test]
    fn test_nested_assertions() {
        let doc = Document::with_items(vec![ContentItem::Session(Session::new(
            "Getting Started".to_string(),
            vec![
                ContentItem::Paragraph(Paragraph::from_line("Hello".to_string())),
                ContentItem::Paragraph(Paragraph::from_line("Not sure what to do".to_string())),
            ],
        ))]);

        assert_ast(&doc).item(0, |item| {
            item.assert_session()
                .label("Getting Started")
                .child_count(2)
                .child(0, |child| {
                    child.assert_paragraph().text_starts_with("Hello");
                })
                .child(1, |child| {
                    child.assert_paragraph().text_starts_with("Not sure");
                });
        });
    }

    #[test]
    fn test_children_assertion() {
        let doc = Document::with_items(vec![ContentItem::Session(Session::new(
            "Section".to_string(),
            vec![
                ContentItem::Paragraph(Paragraph::from_line("Para 1".to_string())),
                ContentItem::Paragraph(Paragraph::from_line("Para 2".to_string())),
            ],
        ))]);

        assert_ast(&doc).item(0, |item| {
            item.assert_session().children(|children| {
                children
                    .count(2)
                    .all_paragraphs()
                    .item(0, |child| {
                        child.assert_paragraph().text("Para 1");
                    })
                    .item(1, |child| {
                        child.assert_paragraph().text("Para 2");
                    });
            });
        });
    }

    #[test]
    #[should_panic(expected = "items[0]: Expected Paragraph, found Session with label 'Section'")]
    fn test_type_mismatch_session_as_paragraph() {
        let doc = Document::with_items(vec![ContentItem::Session(Session::with_title(
            "Section".to_string(),
        ))]);

        assert_ast(&doc).item(0, |item| {
            item.assert_paragraph();
        });
    }

    #[test]
    #[should_panic(
        expected = "items[0]: Expected Session, found Paragraph with text 'Hello world'"
    )]
    fn test_type_mismatch_paragraph_as_session() {
        let doc = Document::with_items(vec![ContentItem::Paragraph(Paragraph::from_line(
            "Hello world".to_string(),
        ))]);

        assert_ast(&doc).item(0, |item| {
            item.assert_session();
        });
    }
}
