//! Fluent assertion API for AST nodes

use super::testing_matchers::TextMatch;
use crate::lex::ast::{
    Annotation, Container, ContentItem, Definition, Document, List, ListItem, Paragraph, Session,
    Verbatim,
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
        let actual = self.doc.root.children.len();
        assert_eq!(
            actual,
            expected,
            "Expected {} items, found {} items: [{}]",
            expected,
            actual,
            summarize_items(&self.doc.root.children)
        );
        self
    }

    /// Assert on a specific item by index
    pub fn item<F>(self, index: usize, assertion: F) -> Self
    where
        F: FnOnce(ContentItemAssertion<'a>),
    {
        assert!(
            index < self.doc.root.children.len(),
            "Item index {} out of bounds (document has {} items)",
            index,
            self.doc.root.children.len()
        );

        let item = &self.doc.root.children[index];
        assertion(ContentItemAssertion {
            item,
            context: format!("items[{}]", index),
        });
        self
    }

    /// Assert the root session location starts at the given line and column
    pub fn root_location_starts_at(self, expected_line: usize, expected_column: usize) -> Self {
        let actual = self.doc.root.location.clone();
        assert_eq!(
            actual.clone().start.line,
            expected_line,
            "Expected root session location start line {}, found {}",
            expected_line,
            actual.clone().start.line
        );
        assert_eq!(
            actual.clone().start.column,
            expected_column,
            "Expected root session location start column {}, found {}",
            expected_column,
            actual.clone().start.column
        );
        self
    }

    /// Assert the root session location ends at the given line and column
    pub fn root_location_ends_at(self, expected_line: usize, expected_column: usize) -> Self {
        let actual = self.doc.root.location.clone();
        assert_eq!(
            actual.end.line, expected_line,
            "Expected root session location end line {}, found {}",
            expected_line, actual.end.line
        );
        assert_eq!(
            actual.end.column, expected_column,
            "Expected root session location end column {}, found {}",
            expected_column, actual.end.column
        );
        self
    }

    /// Assert the root session location contains the given position
    pub fn root_location_contains(self, line: usize, column: usize) -> Self {
        use crate::lex::ast::range::Position;

        let pos = Position::new(line, column);
        let location = self.doc.root.location.clone();
        assert!(
            location.contains(pos),
            "Expected root session location {} to contain position {}:{}",
            location,
            line,
            column
        );
        self
    }

    /// Assert the root session location does NOT contain the given position
    pub fn root_location_excludes(self, line: usize, column: usize) -> Self {
        use crate::lex::ast::range::Position;

        let pos = Position::new(line, column);
        let location = self.doc.root.location.clone();
        assert!(
            !location.contains(pos),
            "Expected root session location {} to NOT contain position {}:{}",
            location,
            line,
            column
        );
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

    /// Assert this item is a VerbatimBlock and return foreign block-specific assertions
    pub fn assert_verbatim_block(self) -> VerbatimBlockkAssertion<'a> {
        match self.item {
            ContentItem::VerbatimBlock(fb) => VerbatimBlockkAssertion {
                verbatim_block: fb,
                context: self.context,
            },
            _ => panic!(
                "{}: Expected VerbatimBlock, found {}",
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
        let actual = self.list.items.len();
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
            index < self.list.items.len(),
            "{}: Item index {} out of bounds (list has {} items)",
            self.context,
            index,
            self.list.items.len()
        );
        let content_item = &self.list.items[index];
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

    /// Assert that a parameter with the given key exists (any value)
    pub fn has_parameter(self, key: &str) -> Self {
        let found = self.annotation.parameters.iter().any(|p| p.key == key);
        assert!(
            found,
            "{}: Expected parameter with key '{}' to exist, but found parameters: [{}]",
            self.context,
            key,
            self.annotation
                .parameters
                .iter()
                .map(|p| format!("{}={}", p.key, p.value))
                .collect::<Vec<_>>()
                .join(", ")
        );
        self
    }

    /// Assert that a parameter with the given key does NOT exist
    pub fn no_parameter(self, key: &str) -> Self {
        let found = self.annotation.parameters.iter().any(|p| p.key == key);
        assert!(
            !found,
            "{}: Expected no parameter with key '{}', but found it with value '{}'",
            self.context,
            key,
            self.annotation
                .parameters
                .iter()
                .find(|p| p.key == key)
                .map(|p| p.value.as_str())
                .unwrap_or("")
        );
        self
    }

    /// Assert that a parameter with the given key has the expected value
    pub fn has_parameter_with_value(self, key: &str, value: &str) -> Self {
        let param = self.annotation.parameters.iter().find(|p| p.key == key);
        match param {
            Some(p) => {
                assert_eq!(
                    p.value, value,
                    "{}: Expected parameter '{}' to have value '{}', but got '{}'",
                    self.context, key, value, p.value
                );
            }
            None => {
                panic!(
                    "{}: Expected parameter '{}={}' to exist, but parameter '{}' not found. Available parameters: [{}]",
                    self.context,
                    key,
                    value,
                    key,
                    self.annotation
                        .parameters
                        .iter()
                        .map(|p| format!("{}={}", p.key, p.value))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }
        self
    }

    /// Assert on a specific parameter by index
    pub fn parameter(self, index: usize, expected_key: &str, expected_value: &str) -> Self {
        assert!(
            index < self.annotation.parameters.len(),
            "{}: Parameter index {} out of bounds (annotation has {} parameters)",
            self.context,
            index,
            self.annotation.parameters.len()
        );
        let param = &self.annotation.parameters[index];
        assert_eq!(
            param.key, expected_key,
            "{}: Expected parameter[{}].key to be '{}', but got '{}'",
            self.context, index, expected_key, param.key
        );
        assert_eq!(
            param.value, expected_value,
            "{}: Expected parameter[{}].value to be '{}', but got '{}'",
            self.context, index, expected_value, param.value
        );
        self
    }

    /// Assert that parameter at given index has the expected key (any value)
    pub fn parameter_key(self, index: usize, expected_key: &str) -> Self {
        assert!(
            index < self.annotation.parameters.len(),
            "{}: Parameter index {} out of bounds (annotation has {} parameters)",
            self.context,
            index,
            self.annotation.parameters.len()
        );
        let param = &self.annotation.parameters[index];
        assert_eq!(
            param.key, expected_key,
            "{}: Expected parameter[{}].key to be '{}', but got '{}'",
            self.context, index, expected_key, param.key
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
// Verbatim Block Assertions
// ============================================================================

pub struct VerbatimBlockkAssertion<'a> {
    verbatim_block: &'a Verbatim,
    context: String,
}

impl<'a> VerbatimBlockkAssertion<'a> {
    pub fn subject(self, expected: &str) -> Self {
        let actual = self.verbatim_block.subject.as_string();
        assert_eq!(
            actual, expected,
            "{}: Expected foreign block subject to be '{}', but got '{}'",
            self.context, expected, actual
        );
        self
    }
    pub fn content_contains(self, substring: &str) -> Self {
        // Collect all content lines into a single string
        let actual: String = self
            .verbatim_block
            .children
            .iter()
            .filter_map(|child| child.as_foreign_line())
            .map(|line| line.content.as_string())
            .collect::<Vec<_>>()
            .join("\n");

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
        assert!(
            self.verbatim_block.children.is_empty(),
            "{}: Expected foreign block to be marker form (empty children), but got {} children",
            self.context,
            self.verbatim_block.children.len()
        );
        self
    }
    pub fn closing_label(self, expected: &str) -> Self {
        let actual = &self.verbatim_block.closing_annotation.label.value;
        assert_eq!(
            actual, expected,
            "{}: Expected closing annotation label to be '{}', but got '{}'",
            self.context, expected, actual
        );
        self
    }
    pub fn has_closing_parameter_with_value(self, key: &str, value: &str) -> Self {
        let found = self
            .verbatim_block
            .closing_annotation
            .parameters
            .iter()
            .any(|p| p.key == key && p.value == value);
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

// ============================================================================
// Tests for Assertions (these tests inspect raw AST)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::ast::range::{Position, Range};
    use crate::lex::ast::{Annotation, Document, Label, Parameter, Session};

    #[test]
    fn test_root_location_starts_at() {
        let location = Range::new(0..0, Position::new(0, 0), Position::new(0, 10));
        let mut session = Session::with_title(String::new());
        session.location = location;
        let doc = Document {
            metadata: Vec::new(),
            root: session,
        };

        // Should pass
        assert_ast(&doc).root_location_starts_at(0, 0);
    }

    #[test]
    #[should_panic(expected = "Expected root session location start line 5, found 0")]
    fn test_root_location_starts_at_fails_wrong_line() {
        let location = Range::new(0..0, Position::new(0, 0), Position::new(0, 10));
        let mut session = Session::with_title(String::new());
        session.location = location;
        let doc = Document {
            metadata: Vec::new(),
            root: session,
        };

        assert_ast(&doc).root_location_starts_at(5, 0);
    }

    #[test]
    fn test_root_location_ends_at() {
        let location = Range::new(0..0, Position::new(0, 0), Position::new(2, 15));
        let mut session = Session::with_title(String::new());
        session.location = location;
        let doc = Document {
            metadata: Vec::new(),
            root: session,
        };

        assert_ast(&doc).root_location_ends_at(2, 15);
    }

    #[test]
    #[should_panic(expected = "Expected root session location end column 10, found 15")]
    fn test_root_location_ends_at_fails_wrong_column() {
        let location = Range::new(0..0, Position::new(0, 0), Position::new(2, 15));
        let mut session = Session::with_title(String::new());
        session.location = location;
        let doc = Document {
            metadata: Vec::new(),
            root: session,
        };

        assert_ast(&doc).root_location_ends_at(2, 10);
    }

    #[test]
    fn test_root_location_contains() {
        let location = Range::new(0..0, Position::new(1, 0), Position::new(3, 10));
        let mut session = Session::with_title(String::new());
        session.location = location;
        let doc = Document {
            metadata: Vec::new(),
            root: session,
        };

        assert_ast(&doc).root_location_contains(2, 5); // Inside
    }

    #[test]
    #[should_panic(expected = "Expected root session location")]
    fn test_root_location_contains_fails() {
        let location = Range::new(0..0, Position::new(1, 0), Position::new(3, 10));
        let mut session = Session::with_title(String::new());
        session.location = location;
        let doc = Document {
            metadata: Vec::new(),
            root: session,
        };

        assert_ast(&doc).root_location_contains(5, 5); // Outside
    }

    #[test]
    fn test_root_location_excludes() {
        let location = Range::new(0..0, Position::new(1, 0), Position::new(3, 10));
        let mut session = Session::with_title(String::new());
        session.location = location;
        let doc = Document {
            metadata: Vec::new(),
            root: session,
        };

        assert_ast(&doc).root_location_excludes(5, 5); // Outside
    }

    #[test]
    #[should_panic(expected = "Expected root session location")]
    fn test_root_location_excludes_fails() {
        let location = Range::new(0..0, Position::new(1, 0), Position::new(3, 10));
        let mut session = Session::with_title(String::new());
        session.location = location;
        let doc = Document {
            metadata: Vec::new(),
            root: session,
        };

        assert_ast(&doc).root_location_excludes(2, 5); // Inside
    }

    #[test]
    fn test_location_assertions_are_fluent() {
        let location = Range::new(0..0, Position::new(0, 0), Position::new(5, 20));
        let mut session = Session::with_title(String::new());
        session.location = location;
        let doc = Document {
            metadata: Vec::new(),
            root: session,
        };

        // Test fluent chaining
        assert_ast(&doc)
            .root_location_starts_at(0, 0)
            .root_location_ends_at(5, 20)
            .root_location_contains(2, 10)
            .root_location_excludes(10, 0)
            .item_count(0);
    }

    // ============================================================================
    // Parameter Assertion Tests (these tests inspect raw AST)
    // ============================================================================

    fn create_test_annotation(label: &str, parameters: Vec<(&str, &str)>) -> Annotation {
        let location = Range::new(0..0, Position::new(0, 0), Position::new(0, 10));
        let label = Label::new(label.to_string()).at(location.clone());
        let parameters: Vec<Parameter> = parameters
            .into_iter()
            .map(|(k, v)| Parameter {
                key: k.to_string(),
                value: v.to_string(),
                location: location.clone(),
            })
            .collect();
        Annotation {
            label,
            parameters,
            children: crate::lex::ast::elements::container::GeneralContainer::empty(),
            location,
        }
    }

    #[test]
    fn test_annotation_label_assertion() {
        let annotation = create_test_annotation("test", vec![]);
        // Directly verify the annotation has the expected label in raw AST
        assert_eq!(annotation.label.value, "test");

        // Now test the assertion API
        let annotation_assertion = AnnotationAssertion {
            annotation: &annotation,
            context: "test".to_string(),
        };
        annotation_assertion.label("test");
    }

    #[test]
    #[should_panic(expected = "Expected annotation label to be 'wrong'")]
    fn test_annotation_label_assertion_fails() {
        let annotation = create_test_annotation("test", vec![]);
        let annotation_assertion = AnnotationAssertion {
            annotation: &annotation,
            context: "test".to_string(),
        };
        annotation_assertion.label("wrong");
    }

    #[test]
    fn test_parameter_count_assertion() {
        let annotation = create_test_annotation("test", vec![("key1", "val1"), ("key2", "val2")]);
        // Directly verify parameter count in raw AST
        assert_eq!(annotation.parameters.len(), 2);

        // Test assertion API
        let annotation_assertion = AnnotationAssertion {
            annotation: &annotation,
            context: "test".to_string(),
        };
        annotation_assertion.parameter_count(2);
    }

    #[test]
    #[should_panic(expected = "Expected 3 parameters, found 2 parameters")]
    fn test_parameter_count_assertion_fails() {
        let annotation = create_test_annotation("test", vec![("key1", "val1"), ("key2", "val2")]);
        let annotation_assertion = AnnotationAssertion {
            annotation: &annotation,
            context: "test".to_string(),
        };
        annotation_assertion.parameter_count(3);
    }

    #[test]
    fn test_has_parameter_assertion() {
        let annotation = create_test_annotation("test", vec![("foo", "bar"), ("baz", "qux")]);
        // Directly verify parameter exists in raw AST
        assert!(annotation.parameters.iter().any(|p| p.key == "foo"));

        // Test assertion API
        let annotation_assertion = AnnotationAssertion {
            annotation: &annotation,
            context: "test".to_string(),
        };
        annotation_assertion.has_parameter("foo");
    }

    #[test]
    #[should_panic(expected = "Expected parameter with key 'missing'")]
    fn test_has_parameter_assertion_fails() {
        let annotation = create_test_annotation("test", vec![("foo", "bar")]);
        let annotation_assertion = AnnotationAssertion {
            annotation: &annotation,
            context: "test".to_string(),
        };
        annotation_assertion.has_parameter("missing");
    }

    #[test]
    fn test_no_parameter_assertion() {
        let annotation = create_test_annotation("test", vec![("foo", "bar")]);
        // Directly verify parameter doesn't exist in raw AST
        assert!(!annotation.parameters.iter().any(|p| p.key == "missing"));

        // Test assertion API
        let annotation_assertion = AnnotationAssertion {
            annotation: &annotation,
            context: "test".to_string(),
        };
        annotation_assertion.no_parameter("missing");
    }

    #[test]
    #[should_panic(expected = "Expected no parameter with key 'foo'")]
    fn test_no_parameter_assertion_fails() {
        let annotation = create_test_annotation("test", vec![("foo", "bar")]);
        let annotation_assertion = AnnotationAssertion {
            annotation: &annotation,
            context: "test".to_string(),
        };
        annotation_assertion.no_parameter("foo");
    }

    #[test]
    fn test_has_parameter_with_value_assertion() {
        let annotation = create_test_annotation("test", vec![("key", "value"), ("other", "data")]);
        // Directly verify parameter key-value pair in raw AST
        let param = annotation.parameters.iter().find(|p| p.key == "key");
        assert!(param.is_some());
        assert_eq!(param.unwrap().value, "value");

        // Test assertion API
        let annotation_assertion = AnnotationAssertion {
            annotation: &annotation,
            context: "test".to_string(),
        };
        annotation_assertion.has_parameter_with_value("key", "value");
    }

    #[test]
    #[should_panic(expected = "Expected parameter 'key' to have value 'wrong'")]
    fn test_has_parameter_with_value_assertion_fails_wrong_value() {
        let annotation = create_test_annotation("test", vec![("key", "value")]);
        let annotation_assertion = AnnotationAssertion {
            annotation: &annotation,
            context: "test".to_string(),
        };
        annotation_assertion.has_parameter_with_value("key", "wrong");
    }

    #[test]
    #[should_panic(expected = "parameter 'missing' not found")]
    fn test_has_parameter_with_value_assertion_fails_missing_key() {
        let annotation = create_test_annotation("test", vec![("key", "value")]);
        let annotation_assertion = AnnotationAssertion {
            annotation: &annotation,
            context: "test".to_string(),
        };
        annotation_assertion.has_parameter_with_value("missing", "value");
    }

    #[test]
    fn test_parameter_by_index_assertion() {
        let annotation = create_test_annotation("test", vec![("first", "1"), ("second", "2")]);
        // Directly verify parameter at index in raw AST
        assert_eq!(annotation.parameters[0].key, "first");
        assert_eq!(annotation.parameters[0].value, "1");
        assert_eq!(annotation.parameters[1].key, "second");
        assert_eq!(annotation.parameters[1].value, "2");

        // Test assertion API
        let annotation_assertion = AnnotationAssertion {
            annotation: &annotation,
            context: "test".to_string(),
        };
        annotation_assertion
            .parameter(0, "first", "1")
            .parameter(1, "second", "2");
    }

    #[test]
    #[should_panic(expected = "Parameter index 2 out of bounds")]
    fn test_parameter_by_index_assertion_out_of_bounds() {
        let annotation = create_test_annotation("test", vec![("key", "value")]);
        let annotation_assertion = AnnotationAssertion {
            annotation: &annotation,
            context: "test".to_string(),
        };
        annotation_assertion.parameter(2, "key", "value");
    }

    #[test]
    #[should_panic(expected = "Expected parameter[0].key to be 'wrong'")]
    fn test_parameter_by_index_assertion_fails_key() {
        let annotation = create_test_annotation("test", vec![("key", "value")]);
        let annotation_assertion = AnnotationAssertion {
            annotation: &annotation,
            context: "test".to_string(),
        };
        annotation_assertion.parameter(0, "wrong", "value");
    }

    #[test]
    #[should_panic(expected = "Expected parameter[0].value to be 'wrong'")]
    fn test_parameter_by_index_assertion_fails_value() {
        let annotation = create_test_annotation("test", vec![("key", "value")]);
        let annotation_assertion = AnnotationAssertion {
            annotation: &annotation,
            context: "test".to_string(),
        };
        annotation_assertion.parameter(0, "key", "wrong");
    }

    #[test]
    fn test_parameter_key_by_index_assertion() {
        let annotation = create_test_annotation("test", vec![("key1", "val1"), ("key2", "val2")]);
        // Directly verify parameter keys in raw AST
        assert_eq!(annotation.parameters[0].key, "key1");
        assert_eq!(annotation.parameters[1].key, "key2");

        // Test assertion API (doesn't check value)
        let annotation_assertion = AnnotationAssertion {
            annotation: &annotation,
            context: "test".to_string(),
        };
        annotation_assertion
            .parameter_key(0, "key1")
            .parameter_key(1, "key2");
    }

    #[test]
    fn test_fluent_parameter_assertions() {
        let annotation = create_test_annotation(
            "test",
            vec![("foo", "bar"), ("baz", "qux"), ("other", "data")],
        );

        // Test fluent chaining of parameter assertions
        let annotation_assertion = AnnotationAssertion {
            annotation: &annotation,
            context: "test".to_string(),
        };
        annotation_assertion
            .label("test")
            .parameter_count(3)
            .has_parameter("foo")
            .has_parameter("baz")
            .has_parameter_with_value("foo", "bar")
            .has_parameter_with_value("baz", "qux")
            .parameter(0, "foo", "bar")
            .parameter(1, "baz", "qux")
            .no_parameter("missing")
            .child_count(0);
    }
}
