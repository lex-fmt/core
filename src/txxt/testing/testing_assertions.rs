//! Fluent assertion API for AST nodes

use super::testing_matchers::TextMatch;
use crate::txxt::ast::{
    Annotation, Container, ContentItem, Definition, Document, ForeignBlock, List, ListItem,
    Paragraph, Session,
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
        let actual = self.doc.content.len();
        assert_eq!(
            actual,
            expected,
            "Expected {} items, found {} items: [{}]",
            expected,
            actual,
            summarize_items(&self.doc.content)
        );
        self
    }

    /// Assert on a specific item by index
    pub fn item<F>(self, index: usize, assertion: F) -> Self
    where
        F: FnOnce(ContentItemAssertion<'a>),
    {
        assert!(
            index < self.doc.content.len(),
            "Item index {} out of bounds (document has {} items)",
            index,
            self.doc.content.len()
        );

        let item = &self.doc.content[index];
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
            _ => panic!(
                "{}: Expected Paragraph, found {}",
                self.context,
                self.item.node_type()
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
            _ => panic!(
                "{}: Expected Session, found {}",
                self.context,
                self.item.node_type()
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
            _ => panic!(
                "{}: Expected List, found {}",
                self.context,
                self.item.node_type()
            ),
        }
    }

    /// Assert this item is a Definition and return definition-specific assertions
    pub fn assert_definition(self) -> DefinitionAssertion<'a> {
        match self.item {
            ContentItem::Definition(d) => DefinitionAssertion {
                definition: d,
                context: self.context,
            },
            _ => panic!(
                "{}: Expected Definition, found {}",
                self.context,
                self.item.node_type()
            ),
        }
    }

    /// Assert this item is an Annotation and return annotation-specific assertions
    pub fn assert_annotation(self) -> AnnotationAssertion<'a> {
        match self.item {
            ContentItem::Annotation(a) => AnnotationAssertion {
                annotation: a,
                context: self.context,
            },
            _ => panic!(
                "{}: Expected Annotation, found {}",
                self.context,
                self.item.node_type()
            ),
        }
    }

    /// Assert this item is a ForeignBlock and return foreign block-specific assertions
    pub fn assert_foreign_block(self) -> ForeignBlockAssertion<'a> {
        match self.item {
            ContentItem::ForeignBlock(fb) => ForeignBlockAssertion {
                foreign_block: fb,
                context: self.context,
            },
            _ => panic!(
                "{}: Expected ForeignBlock, found {}",
                self.context,
                self.item.node_type()
            ),
        }
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
    pub fn text(self, expected: &str) -> Self {
        TextMatch::Exact(expected.to_string()).assert(&self.para.text(), &self.context);
        self
    }
    pub fn text_starts_with(self, prefix: &str) -> Self {
        TextMatch::StartsWith(prefix.to_string()).assert(&self.para.text(), &self.context);
        self
    }
    pub fn text_contains(self, substring: &str) -> Self {
        TextMatch::Contains(substring.to_string()).assert(&self.para.text(), &self.context);
        self
    }
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
    pub fn label(self, expected: &str) -> Self {
        let actual = self.session.label();
        assert_eq!(
            actual, expected,
            "{}: Expected session label to be '{}', but got '{}'",
            self.context, expected, actual
        );
        self
    }
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
    pub fn item_count(self, expected: usize) -> Self {
        let actual = self.list.content.len();
        assert_eq!(
            actual, expected,
            "{}: Expected {} list items, found {} list items",
            self.context, expected, actual
        );
        self
    }
    pub fn item<F>(self, index: usize, assertion: F) -> Self
    where
        F: FnOnce(ListItemAssertion<'a>),
    {
        assert!(
            index < self.list.content.len(),
            "{}: Item index {} out of bounds (list has {} items)",
            self.context,
            index,
            self.list.content.len()
        );
        let content_item = &self.list.content[index];
        let item = if let ContentItem::ListItem(li) = content_item {
            li
        } else {
            panic!(
                "{}: Expected ListItem at index {}, but found {:?}",
                self.context,
                index,
                content_item.node_type()
            );
        };
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
    pub fn text(self, expected: &str) -> Self {
        TextMatch::Exact(expected.to_string()).assert(self.item.text(), &self.context);
        self
    }
    pub fn text_starts_with(self, prefix: &str) -> Self {
        TextMatch::StartsWith(prefix.to_string()).assert(self.item.text(), &self.context);
        self
    }
    pub fn text_contains(self, substring: &str) -> Self {
        TextMatch::Contains(substring.to_string()).assert(self.item.text(), &self.context);
        self
    }
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
// Definition Assertions
// ============================================================================

pub struct DefinitionAssertion<'a> {
    definition: &'a Definition,
    context: String,
}

impl<'a> DefinitionAssertion<'a> {
    pub fn subject(self, expected: &str) -> Self {
        TextMatch::Exact(expected.to_string())
            .assert(self.definition.subject.as_string(), &self.context);
        self
    }
    pub fn subject_starts_with(self, prefix: &str) -> Self {
        TextMatch::StartsWith(prefix.to_string())
            .assert(self.definition.subject.as_string(), &self.context);
        self
    }
    pub fn subject_contains(self, substring: &str) -> Self {
        TextMatch::Contains(substring.to_string())
            .assert(self.definition.subject.as_string(), &self.context);
        self
    }
    pub fn child_count(self, expected: usize) -> Self {
        let actual = self.definition.children().len();
        assert_eq!(
            actual,
            expected,
            "{}: Expected {} children, found {} children: [{}]",
            self.context,
            expected,
            actual,
            summarize_items(self.definition.children())
        );
        self
    }
    pub fn child<F>(self, index: usize, assertion: F) -> Self
    where
        F: FnOnce(ContentItemAssertion<'a>),
    {
        let children = self.definition.children();
        assert!(
            index < children.len(),
            "{}: Child index {} out of bounds (definition has {} children)",
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
    pub fn children<F>(self, assertion: F) -> Self
    where
        F: FnOnce(ChildrenAssertion<'a>),
    {
        assertion(ChildrenAssertion {
            children: self.definition.children(),
            context: format!("{}:children", self.context),
        });
        self
    }
}

// ============================================================================
// Annotation Assertions
// ============================================================================

pub struct AnnotationAssertion<'a> {
    annotation: &'a Annotation,
    context: String,
}

impl<'a> AnnotationAssertion<'a> {
    pub fn label(self, expected: &str) -> Self {
        let actual = &self.annotation.label.value;
        assert_eq!(
            actual, expected,
            "{}: Expected annotation label to be '{}', but got '{}'",
            self.context, expected, actual
        );
        self
    }
    pub fn parameter_count(self, expected: usize) -> Self {
        let actual = self.annotation.parameters.len();
        assert_eq!(
            actual, expected,
            "{}: Expected {} parameters, found {} parameters",
            self.context, expected, actual
        );
        self
    }
    pub fn has_parameter_with_value(self, key: &str, value: &str) -> Self {
        let found = self
            .annotation
            .parameters
            .iter()
            .any(|p| p.key == key && p.value.as_deref() == Some(value));
        assert!(
            found,
            "{}: Expected parameter '{}={}' to exist",
            self.context, key, value
        );
        self
    }
    pub fn has_boolean_parameter(self, key: &str) -> Self {
        let found = self
            .annotation
            .parameters
            .iter()
            .any(|p| p.key == key && p.value.is_none());
        assert!(
            found,
            "{}: Expected boolean parameter '{}' to exist",
            self.context, key
        );
        self
    }
    pub fn child_count(self, expected: usize) -> Self {
        let actual = self.annotation.children().len();
        assert_eq!(
            actual,
            expected,
            "{}: Expected {} children, found {} children: [{}]",
            self.context,
            expected,
            actual,
            summarize_items(self.annotation.children())
        );
        self
    }
    pub fn child<F>(self, index: usize, assertion: F) -> Self
    where
        F: FnOnce(ContentItemAssertion<'a>),
    {
        let children = self.annotation.children();
        assert!(
            index < children.len(),
            "{}: Child index {} out of bounds (annotation has {} children)",
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
}

// ============================================================================
// Foreign Block Assertions
// ============================================================================

pub struct ForeignBlockAssertion<'a> {
    foreign_block: &'a ForeignBlock,
    context: String,
}

impl<'a> ForeignBlockAssertion<'a> {
    pub fn subject(self, expected: &str) -> Self {
        let actual = self.foreign_block.subject.as_string();
        assert_eq!(
            actual, expected,
            "{}: Expected foreign block subject to be '{}', but got '{}'",
            self.context, expected, actual
        );
        self
    }
    pub fn content_contains(self, substring: &str) -> Self {
        let actual = self.foreign_block.content.as_string();
        assert!(
            actual.contains(substring),
            "{}: Expected foreign block content to contain '{}', but got '{}'",
            self.context,
            substring,
            actual
        );
        self
    }
    pub fn is_marker_form(self) -> Self {
        let actual = self.foreign_block.content.as_string();
        assert!(
            actual.is_empty(),
            "{}: Expected foreign block to be marker form (empty content), but got '{}'",
            self.context,
            actual
        );
        self
    }
    pub fn closing_label(self, expected: &str) -> Self {
        let actual = &self.foreign_block.closing_annotation.label.value;
        assert_eq!(
            actual, expected,
            "{}: Expected closing annotation label to be '{}', but got '{}'",
            self.context, expected, actual
        );
        self
    }
    pub fn has_closing_parameter_with_value(self, key: &str, value: &str) -> Self {
        let found = self
            .foreign_block
            .closing_annotation
            .parameters
            .iter()
            .any(|p| p.key == key && p.value.as_deref() == Some(value));
        assert!(
            found,
            "{}: Expected closing annotation to have parameter '{}={}'",
            self.context, key, value
        );
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

fn summarize_items(items: &[ContentItem]) -> String {
    items
        .iter()
        .map(|item| item.node_type())
        .collect::<Vec<_>>()
        .join(", ")
}
