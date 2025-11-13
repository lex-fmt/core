//! Session assertions

use super::{summarize_items, ChildrenAssertion};
use crate::lex::ast::traits::Container;
use crate::lex::ast::Session;
use crate::lex::testing::ast_assertions::ContentItemAssertion;

pub struct SessionAssertion<'a> {
    pub(crate) session: &'a Session,
    pub(crate) context: String,
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
