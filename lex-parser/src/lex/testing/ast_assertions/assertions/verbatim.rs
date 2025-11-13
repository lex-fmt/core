//! Verbatim block assertions

use crate::lex::ast::elements::container::VerbatimContainer;
use crate::lex::ast::{ContentItem, TextContent, Verbatim};

pub struct VerbatimBlockkAssertion<'a> {
    pub(crate) verbatim_block: &'a Verbatim,
    pub(crate) context: String,
}

impl<'a> VerbatimBlockkAssertion<'a> {
    pub fn subject(self, expected: &str) -> Self {
        let actual = self.verbatim_block.subject.as_string();
        assert_eq!(
            actual, expected,
            "{}: Expected verbatim block subject to be '{}', but got '{}'",
            self.context, expected, actual
        );
        self
    }
    pub fn content_contains(self, substring: &str) -> Self {
        let actual = collect_verbatim_content(&self.verbatim_block.children);

        assert!(
            actual.contains(substring),
            "{}: Expected verbatim block content to contain '{}', but got '{}'",
            self.context,
            substring,
            actual
        );
        self
    }
    pub fn assert_marker_form(self) -> Self {
        assert!(
            self.verbatim_block.children.is_empty(),
            "{}: Expected verbatim block to be marker form (empty children), but got {} children",
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

    pub fn line_count(self, expected: usize) -> Self {
        let actual = self.verbatim_block.children.len();
        assert_eq!(
            actual, expected,
            "{}: Expected verbatim block to have {} lines, but got {}",
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

    pub fn group_count(self, expected: usize) -> Self {
        let actual = self.verbatim_block.group_len();
        assert_eq!(
            actual, expected,
            "{}: Expected {} verbatim groups, found {}",
            self.context, expected, actual
        );
        self
    }

    pub fn group<F>(self, index: usize, assertion: F) -> Self
    where
        F: FnOnce(VerbatimGroupAssertion<'a>),
    {
        let group_ref = self.verbatim_block.group().nth(index).unwrap_or_else(|| {
            panic!(
                "{}: Verbatim group index {} out of bounds ({} groups)",
                self.context,
                index,
                self.verbatim_block.group_len()
            )
        });

        assertion(VerbatimGroupAssertion {
            subject: group_ref.subject,
            children: group_ref.children,
            context: format!("{}::group[{}]", self.context, index),
        });

        self
    }
}

pub struct VerbatimGroupAssertion<'a> {
    pub(crate) subject: &'a TextContent,
    pub(crate) children: &'a VerbatimContainer,
    pub(crate) context: String,
}

impl<'a> VerbatimGroupAssertion<'a> {
    pub fn subject(self, expected: &str) -> Self {
        let actual = self.subject.as_string();
        assert_eq!(
            actual, expected,
            "{}: Expected verbatim group subject to be '{}', but got '{}'",
            self.context, expected, actual
        );
        self
    }

    pub fn content_contains(self, substring: &str) -> Self {
        let actual = collect_verbatim_content(self.children);
        assert!(
            actual.contains(substring),
            "{}: Expected verbatim group content to contain '{}', but got '{}'",
            self.context,
            substring,
            actual
        );
        self
    }

    pub fn line_count(self, expected: usize) -> Self {
        let actual = self.children.len();
        assert_eq!(
            actual, expected,
            "{}: Expected verbatim group to have {} lines, but got {}",
            self.context, expected, actual
        );
        self
    }

    pub fn assert_marker_form(self) -> Self {
        assert!(
            self.children.is_empty(),
            "{}: Expected group marker form to be empty, but got {} lines",
            self.context,
            self.children.len()
        );
        self
    }
}

fn collect_verbatim_content(children: &VerbatimContainer) -> String {
    children
        .iter()
        .filter_map(|child| {
            if let ContentItem::VerbatimLine(line) = child {
                Some(line.content.as_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
