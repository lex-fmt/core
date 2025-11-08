//! AST element extraction and assertion helpers
//!
//! This module provides utilities for extracting specific elements from parsed
//! documents and making assertions about their content.

use crate::lex::ast::traits::{Container, TextNode};
use crate::lex::ast::{Annotation, ContentItem, Definition, Document, List, Paragraph, Session};

// ===== Helper functions for extracting elements from AST =====

/// Get the first element of a specific type from the document
/// Now uses the new recursive iterator API
pub fn get_first_paragraph(doc: &Document) -> Option<&Paragraph> {
    doc.iter_paragraphs_recursive().next()
}

/// Get the first session from the document
/// Now uses the new recursive iterator API
pub fn get_first_session(doc: &Document) -> Option<&Session> {
    doc.iter_sessions_recursive().next()
}

/// Get the first list from the document
/// Now uses the new recursive iterator API
pub fn get_first_list(doc: &Document) -> Option<&List> {
    doc.iter_lists_recursive().next()
}

/// Get the first definition from the document
/// Now uses the new recursive iterator API
pub fn get_first_definition(doc: &Document) -> Option<&Definition> {
    doc.iter_definitions_recursive().next()
}

/// Get the first annotation from the document
/// Now uses the new recursive iterator API
pub fn get_first_annotation(doc: &Document) -> Option<&Annotation> {
    doc.iter_annotations_recursive().next()
}

/// Get the first verbatim block from the document
/// Note: Returns the boxed Verbatim from VerbatimBlock
/// Now uses the new recursive iterator API
pub fn get_first_verbatim(doc: &Document) -> Option<&crate::lex::ast::Verbatim> {
    doc.iter_verbatim_blocks_recursive().next()
}

// ===== Assertion helpers =====

/// Check if a paragraph's text starts with the given string
pub fn paragraph_text_starts_with(paragraph: &Paragraph, expected: &str) -> bool {
    paragraph.text().starts_with(expected)
}

/// Check if a paragraph's text contains the given string
pub fn paragraph_text_contains(paragraph: &Paragraph, expected: &str) -> bool {
    paragraph.text().contains(expected)
}

// ===== Document comparison utilities =====

/// Check if two documents have matching AST structure
pub fn documents_match(doc1: &Document, doc2: &Document) -> bool {
    doc1.root.children.len() == doc2.root.children.len()
        && doc1
            .root
            .children
            .iter()
            .zip(doc2.root.children.iter())
            .all(|(item1, item2)| content_items_match(item1, item2))
}

/// Check if two content items match (recursive)
pub fn content_items_match(item1: &ContentItem, item2: &ContentItem) -> bool {
    use ContentItem::*;
    match (item1, item2) {
        (Paragraph(p1), Paragraph(p2)) => p1.lines().len() == p2.lines().len(),
        (Session(s1), Session(s2)) => {
            s1.label() == s2.label()
                && s1.children().len() == s2.children().len()
                && s1
                    .children()
                    .iter()
                    .zip(s2.children().iter())
                    .all(|(c1, c2)| content_items_match(c1, c2))
        }
        (List(l1), List(l2)) => l1.items.len() == l2.items.len(),
        (Definition(d1), Definition(d2)) => {
            d1.label() == d2.label() && d1.children().len() == d2.children().len()
        }
        (Annotation(a1), Annotation(a2)) => a1.children().len() == a2.children().len(),
        (VerbatimBlock(v1), VerbatimBlock(v2)) => v1.children.len() == v2.children.len(),
        _ => false, // Different types don't match
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::pipeline::Parser;
    use crate::lex::testing::lexplore::loader::*;

    #[test]
    fn test_get_first_paragraph() {
        let source = Lexplore::get_source_for(ElementType::Paragraph, 1).unwrap();
        let doc = parse_with_parser(&source, Parser::Reference).unwrap();
        let paragraph = get_first_paragraph(&doc);
        assert!(paragraph.is_some());
    }

    #[test]
    fn test_paragraph_assertions() {
        let source = Lexplore::get_source_for(ElementType::Paragraph, 1).unwrap();
        let doc = parse_with_parser(&source, Parser::Reference).unwrap();
        let paragraph = get_first_paragraph(&doc).unwrap();

        assert!(paragraph_text_starts_with(paragraph, "This is a simple"));
    }
}
