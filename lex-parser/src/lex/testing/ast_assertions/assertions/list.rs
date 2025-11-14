//! List and ListItem assertions

use super::{annotation::AnnotationAssertion, summarize_items, ChildrenAssertion};
use crate::lex::ast::traits::{AstNode, Container};
use crate::lex::ast::{ContentItem, List, ListItem};
use crate::lex::testing::ast_assertions::ContentItemAssertion;
use crate::lex::testing::matchers::TextMatch;

pub struct ListAssertion<'a> {
    pub(crate) list: &'a List,
    pub(crate) context: String,
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

    pub fn annotation_count(self, expected: usize) -> Self {
        let actual = self.list.annotations.len();
        assert_eq!(
            actual, expected,
            "{}: Expected {} annotations, found {} annotations",
            self.context, expected, actual
        );
        self
    }

    pub fn annotation<F>(self, index: usize, assertion: F) -> Self
    where
        F: FnOnce(AnnotationAssertion<'a>),
    {
        assert!(
            index < self.list.annotations.len(),
            "{}: Annotation index {} out of bounds (list has {} annotations)",
            self.context,
            index,
            self.list.annotations.len()
        );
        let annotation = &self.list.annotations[index];
        assertion(AnnotationAssertion {
            annotation,
            context: format!("{}:annotations[{}]", self.context, index),
        });
        self
    }
}

pub struct ListItemAssertion<'a> {
    pub(crate) item: &'a ListItem,
    pub(crate) context: String,
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

    pub fn annotation_count(self, expected: usize) -> Self {
        let actual = self.item.annotations.len();
        assert_eq!(
            actual, expected,
            "{}: Expected {} annotations, found {} annotations",
            self.context, expected, actual
        );
        self
    }

    pub fn annotation<F>(self, index: usize, assertion: F) -> Self
    where
        F: FnOnce(AnnotationAssertion<'a>),
    {
        assert!(
            index < self.item.annotations.len(),
            "{}: Annotation index {} out of bounds (list item has {} annotations)",
            self.context,
            index,
            self.item.annotations.len()
        );
        let annotation = &self.item.annotations[index];
        assertion(AnnotationAssertion {
            annotation,
            context: format!("{}:annotations[{}]", self.context, index),
        });
        self
    }
}
