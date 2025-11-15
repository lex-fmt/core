//! Inline content assertions used by parser tests.

use crate::lex::ast::TextContent;
use crate::lex::inlines::{InlineContent, InlineNode};
use crate::lex::testing::matchers::TextMatch;

#[allow(dead_code)]
pub struct InlineAssertion {
    nodes: InlineContent,
    context: String,
}

#[allow(dead_code)]
impl InlineAssertion {
    pub fn new(content: &TextContent, context: impl Into<String>) -> Self {
        Self {
            nodes: content.inline_items(),
            context: context.into(),
        }
    }

    /// Assert that the inline list starts with the provided expectations.
    ///
    /// This mirrors the workflow described in the inline proposal: tests only
    /// need to check the prefix of the inline list for quick sanity checks.
    pub fn starts_with(self, expectations: &[InlineExpectation]) -> Self {
        assert!(
            self.nodes.len() >= expectations.len(),
            "{}: Inline list shorter than expected (have {}, need {})",
            self.context,
            self.nodes.len(),
            expectations.len()
        );
        for (idx, expectation) in expectations.iter().enumerate() {
            let actual = &self.nodes[idx];
            expectation.assert(actual, &format!("{}:inline[{}]", self.context, idx));
        }
        self
    }

    /// Assert the total amount of inline nodes.
    pub fn length(self, expected: usize) -> Self {
        assert_eq!(
            self.nodes.len(),
            expected,
            "{}: Expected {} inline nodes, found {}",
            self.context,
            expected,
            self.nodes.len()
        );
        self
    }

    /// Exposes the raw inline nodes for custom assertions.
    pub fn nodes(&self) -> &[InlineNode] {
        &self.nodes
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct InlineExpectation {
    kind: InlineExpectationKind,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum InlineExpectationKind {
    Plain(TextMatch),
    Strong(Vec<InlineExpectation>),
    Emphasis(Vec<InlineExpectation>),
    Code(TextMatch),
    Math(TextMatch),
}

#[allow(dead_code)]
impl InlineExpectation {
    pub fn plain_text(text: impl Into<String>) -> Self {
        Self {
            kind: InlineExpectationKind::Plain(TextMatch::Exact(text.into())),
        }
    }

    pub fn plain(match_kind: TextMatch) -> Self {
        Self {
            kind: InlineExpectationKind::Plain(match_kind),
        }
    }

    pub fn strong(children: Vec<InlineExpectation>) -> Self {
        Self {
            kind: InlineExpectationKind::Strong(children),
        }
    }

    pub fn strong_text(text: impl Into<String>) -> Self {
        Self::strong(vec![InlineExpectation::plain_text(text.into())])
    }

    pub fn emphasis(children: Vec<InlineExpectation>) -> Self {
        Self {
            kind: InlineExpectationKind::Emphasis(children),
        }
    }

    pub fn emphasis_text(text: impl Into<String>) -> Self {
        Self::emphasis(vec![InlineExpectation::plain_text(text.into())])
    }

    pub fn code_text(text: impl Into<String>) -> Self {
        Self {
            kind: InlineExpectationKind::Code(TextMatch::Exact(text.into())),
        }
    }

    pub fn math_text(text: impl Into<String>) -> Self {
        Self {
            kind: InlineExpectationKind::Math(TextMatch::Exact(text.into())),
        }
    }

    fn assert(&self, actual: &InlineNode, context: &str) {
        match (&self.kind, actual) {
            (InlineExpectationKind::Plain(matcher), InlineNode::Plain(text)) => {
                matcher.assert(text, context);
            }
            (InlineExpectationKind::Strong(expect_children), InlineNode::Strong(children)) => {
                assert_inline_children(children, expect_children, context);
            }
            (InlineExpectationKind::Emphasis(expect_children), InlineNode::Emphasis(children)) => {
                assert_inline_children(children, expect_children, context);
            }
            (InlineExpectationKind::Code(matcher), InlineNode::Code(text)) => {
                matcher.assert(text, context);
            }
            (InlineExpectationKind::Math(matcher), InlineNode::Math(text)) => {
                matcher.assert(text, context);
            }
            (expected, got) => panic!("{}: Expected inline {:?}, got {:?}", context, expected, got),
        }
    }
}

#[allow(dead_code)]
fn assert_inline_children(actual: &InlineContent, expected: &[InlineExpectation], context: &str) {
    assert!(
        actual.len() >= expected.len(),
        "{}: Inline child list shorter than expected (have {}, need {})",
        context,
        actual.len(),
        expected.len()
    );
    for (idx, expectation) in expected.iter().enumerate() {
        let child_context = format!("{}:child[{}]", context, idx);
        expectation.assert(&actual[idx], &child_context);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asserts_inline_prefix() {
        let content = TextContent::from_string("Welcome to *the* party".into(), None);
        InlineAssertion::new(&content, "paragraph.lines[0]")
            .starts_with(&[
                InlineExpectation::plain_text("Welcome to "),
                InlineExpectation::strong_text("the"),
                InlineExpectation::plain_text(" party"),
            ])
            .length(3);
    }

    #[test]
    #[should_panic(expected = "paragraph.lines[0]:inline[0]")]
    fn detects_mismatched_inline() {
        let content = TextContent::from_string("*value*".into(), None);
        InlineAssertion::new(&content, "paragraph.lines[0]")
            .starts_with(&[InlineExpectation::plain_text("value")]);
    }
}
