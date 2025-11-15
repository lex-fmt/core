//! Fluent assertion API for AST nodes

mod assertions;

pub use assertions::{
    AnnotationAssertion, ChildrenAssertion, DefinitionAssertion, DocumentAssertion,
    InlineAssertion, InlineExpectation, ListAssertion, ListItemAssertion, ParagraphAssertion,
    ReferenceExpectation, SessionAssertion, VerbatimBlockkAssertion,
};

use crate::lex::ast::traits::AstNode;
use crate::lex::ast::{ContentItem, Document};

// ============================================================================
// Entry Point
// ============================================================================

/// Create an assertion builder for a document
pub fn assert_ast(doc: &Document) -> DocumentAssertion<'_> {
    DocumentAssertion { doc }
}

// ============================================================================
// ContentItem Assertions
// ============================================================================

pub struct ContentItemAssertion<'a> {
    pub(crate) item: &'a ContentItem,
    pub(crate) context: String,
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

    /// Assert this item is a VerbatimBlock and return verbatim block-specific assertions
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
// Tests for Document-level Assertions (location tests)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::ast::range::{Position, Range};
    use crate::lex::ast::{Document, Session};

    #[test]
    fn test_root_location_starts_at() {
        let location = Range::new(0..0, Position::new(0, 0), Position::new(0, 10));
        let mut session = Session::with_title(String::new());
        session.location = location;
        let doc = Document {
            annotations: Vec::new(),
            root: session,
        };

        assert_ast(&doc).root_location_starts_at(0, 0);
    }

    #[test]
    #[should_panic(expected = "Expected root session location start line 5, found 0")]
    fn test_root_location_starts_at_fails_wrong_line() {
        let location = Range::new(0..0, Position::new(0, 0), Position::new(0, 10));
        let mut session = Session::with_title(String::new());
        session.location = location;
        let doc = Document {
            annotations: Vec::new(),
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
            annotations: Vec::new(),
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
            annotations: Vec::new(),
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
            annotations: Vec::new(),
            root: session,
        };

        assert_ast(&doc).root_location_contains(2, 5);
    }

    #[test]
    #[should_panic(expected = "Expected root session location")]
    fn test_root_location_contains_fails() {
        let location = Range::new(0..0, Position::new(1, 0), Position::new(3, 10));
        let mut session = Session::with_title(String::new());
        session.location = location;
        let doc = Document {
            annotations: Vec::new(),
            root: session,
        };

        assert_ast(&doc).root_location_contains(5, 5);
    }

    #[test]
    fn test_root_location_excludes() {
        let location = Range::new(0..0, Position::new(1, 0), Position::new(3, 10));
        let mut session = Session::with_title(String::new());
        session.location = location;
        let doc = Document {
            annotations: Vec::new(),
            root: session,
        };

        assert_ast(&doc).root_location_excludes(5, 5);
    }

    #[test]
    #[should_panic(expected = "Expected root session location")]
    fn test_root_location_excludes_fails() {
        let location = Range::new(0..0, Position::new(1, 0), Position::new(3, 10));
        let mut session = Session::with_title(String::new());
        session.location = location;
        let doc = Document {
            annotations: Vec::new(),
            root: session,
        };

        assert_ast(&doc).root_location_excludes(2, 5);
    }

    #[test]
    fn test_location_assertions_are_fluent() {
        let location = Range::new(0..0, Position::new(0, 0), Position::new(5, 20));
        let mut session = Session::with_title(String::new());
        session.location = location;
        let doc = Document {
            annotations: Vec::new(),
            root: session,
        };

        assert_ast(&doc)
            .root_location_starts_at(0, 0)
            .root_location_ends_at(5, 20)
            .root_location_contains(2, 10)
            .root_location_excludes(10, 0)
            .item_count(0);
    }
}
