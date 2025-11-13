//! Children assertions (bulk operations)

use super::summarize_items;
use crate::lex::ast::traits::AstNode;
use crate::lex::ast::ContentItem;
use crate::lex::testing::ast_assertions::ContentItemAssertion;

pub struct ChildrenAssertion<'a> {
    pub(crate) children: &'a [ContentItem],
    pub(crate) context: String,
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
