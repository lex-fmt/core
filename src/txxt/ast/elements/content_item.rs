//! Content item
//!
//! `ContentItem` is the common wrapper for all elements that can
//! appear in document content. It lets tooling operate uniformly on
//! mixed structures (paragraphs, sessions, lists, definitions, etc.).
//!
//! Examples:
//! - A session containing paragraphs and a list
//! - A paragraph followed by a definition and an annotation

use super::super::location::{Location, Position};
use super::super::traits::{AstNode, Container, Visitor};
use super::annotation::Annotation;
use super::definition::Definition;
use super::foreign::ForeignBlock;
use super::list::{List, ListItem};
use super::paragraph::{Paragraph, TextLine};
use super::session::Session;
use std::fmt;

/// ContentItem represents any element that can appear in document content
#[derive(Debug, Clone, PartialEq)]
pub enum ContentItem {
    Paragraph(Paragraph),
    Session(Session),
    List(List),
    ListItem(ListItem),
    TextLine(TextLine),
    Definition(Definition),
    Annotation(Annotation),
    ForeignBlock(ForeignBlock),
}

impl AstNode for ContentItem {
    fn node_type(&self) -> &'static str {
        match self {
            ContentItem::Paragraph(p) => p.node_type(),
            ContentItem::Session(s) => s.node_type(),
            ContentItem::List(l) => l.node_type(),
            ContentItem::ListItem(li) => li.node_type(),
            ContentItem::TextLine(tl) => tl.node_type(),
            ContentItem::Definition(d) => d.node_type(),
            ContentItem::Annotation(a) => a.node_type(),
            ContentItem::ForeignBlock(fb) => fb.node_type(),
        }
    }

    fn display_label(&self) -> String {
        match self {
            ContentItem::Paragraph(p) => p.display_label(),
            ContentItem::Session(s) => s.display_label(),
            ContentItem::List(l) => l.display_label(),
            ContentItem::ListItem(li) => li.display_label(),
            ContentItem::TextLine(tl) => tl.display_label(),
            ContentItem::Definition(d) => d.display_label(),
            ContentItem::Annotation(a) => a.display_label(),
            ContentItem::ForeignBlock(fb) => fb.display_label(),
        }
    }

    fn location(&self) -> Option<Location> {
        match self {
            ContentItem::Paragraph(p) => p.location(),
            ContentItem::Session(s) => s.location(),
            ContentItem::List(l) => l.location(),
            ContentItem::ListItem(li) => li.location(),
            ContentItem::TextLine(tl) => tl.location(),
            ContentItem::Definition(d) => d.location(),
            ContentItem::Annotation(a) => a.location(),
            ContentItem::ForeignBlock(fb) => fb.location(),
        }
    }

    fn accept(&self, visitor: &mut dyn Visitor) {
        match self {
            ContentItem::Paragraph(p) => p.accept(visitor),
            ContentItem::Session(s) => s.accept(visitor),
            ContentItem::List(l) => l.accept(visitor),
            ContentItem::ListItem(li) => li.accept(visitor),
            ContentItem::TextLine(tl) => tl.accept(visitor),
            ContentItem::Definition(d) => d.accept(visitor),
            ContentItem::Annotation(a) => a.accept(visitor),
            ContentItem::ForeignBlock(fb) => fb.accept(visitor),
        }
    }
}

impl ContentItem {
    pub fn node_type(&self) -> &'static str {
        match self {
            ContentItem::Paragraph(p) => p.node_type(),
            ContentItem::Session(s) => s.node_type(),
            ContentItem::List(l) => l.node_type(),
            ContentItem::ListItem(li) => li.node_type(),
            ContentItem::TextLine(tl) => tl.node_type(),
            ContentItem::Definition(d) => d.node_type(),
            ContentItem::Annotation(a) => a.node_type(),
            ContentItem::ForeignBlock(fb) => fb.node_type(),
        }
    }

    pub fn display_label(&self) -> String {
        match self {
            ContentItem::Paragraph(p) => p.display_label(),
            ContentItem::Session(s) => s.display_label(),
            ContentItem::List(l) => l.display_label(),
            ContentItem::ListItem(li) => li.display_label(),
            ContentItem::TextLine(tl) => tl.display_label(),
            ContentItem::Definition(d) => d.display_label(),
            ContentItem::Annotation(a) => a.display_label(),
            ContentItem::ForeignBlock(fb) => fb.display_label(),
        }
    }

    pub fn label(&self) -> Option<&str> {
        match self {
            ContentItem::Session(s) => Some(s.label()),
            ContentItem::Definition(d) => Some(d.label()),
            ContentItem::Annotation(a) => Some(a.label()),
            ContentItem::ListItem(li) => Some(li.label()),
            ContentItem::ForeignBlock(fb) => Some(fb.subject.as_string()),
            _ => None,
        }
    }

    pub fn children(&self) -> Option<&[ContentItem]> {
        match self {
            ContentItem::Session(s) => Some(s.children()),
            ContentItem::Definition(d) => Some(d.children()),
            ContentItem::Annotation(a) => Some(a.children()),
            ContentItem::List(l) => Some(&l.content),
            ContentItem::ListItem(li) => Some(li.children()),
            ContentItem::Paragraph(p) => Some(&p.lines),
            ContentItem::TextLine(_) => None,
            _ => None,
        }
    }

    pub fn children_mut(&mut self) -> Option<&mut Vec<ContentItem>> {
        match self {
            ContentItem::Session(s) => Some(s.children_mut()),
            ContentItem::Definition(d) => Some(d.children_mut()),
            ContentItem::Annotation(a) => Some(a.children_mut()),
            ContentItem::List(l) => Some(&mut l.content),
            ContentItem::ListItem(li) => Some(li.children_mut()),
            ContentItem::Paragraph(p) => Some(&mut p.lines),
            ContentItem::TextLine(_) => None,
            _ => None,
        }
    }

    pub fn text(&self) -> Option<String> {
        match self {
            ContentItem::Paragraph(p) => Some(p.text()),
            _ => None,
        }
    }

    pub fn is_paragraph(&self) -> bool {
        matches!(self, ContentItem::Paragraph(_))
    }
    pub fn is_session(&self) -> bool {
        matches!(self, ContentItem::Session(_))
    }
    pub fn is_list(&self) -> bool {
        matches!(self, ContentItem::List(_))
    }
    pub fn is_list_item(&self) -> bool {
        matches!(self, ContentItem::ListItem(_))
    }
    pub fn is_text_line(&self) -> bool {
        matches!(self, ContentItem::TextLine(_))
    }
    pub fn is_definition(&self) -> bool {
        matches!(self, ContentItem::Definition(_))
    }
    pub fn is_annotation(&self) -> bool {
        matches!(self, ContentItem::Annotation(_))
    }
    pub fn is_foreign_block(&self) -> bool {
        matches!(self, ContentItem::ForeignBlock(_))
    }

    pub fn as_paragraph(&self) -> Option<&Paragraph> {
        if let ContentItem::Paragraph(p) = self {
            Some(p)
        } else {
            None
        }
    }
    pub fn as_session(&self) -> Option<&Session> {
        if let ContentItem::Session(s) = self {
            Some(s)
        } else {
            None
        }
    }
    pub fn as_list(&self) -> Option<&List> {
        if let ContentItem::List(l) = self {
            Some(l)
        } else {
            None
        }
    }
    pub fn as_list_item(&self) -> Option<&ListItem> {
        if let ContentItem::ListItem(li) = self {
            Some(li)
        } else {
            None
        }
    }
    pub fn as_definition(&self) -> Option<&Definition> {
        if let ContentItem::Definition(d) = self {
            Some(d)
        } else {
            None
        }
    }
    pub fn as_annotation(&self) -> Option<&Annotation> {
        if let ContentItem::Annotation(a) = self {
            Some(a)
        } else {
            None
        }
    }
    pub fn as_foreign_block(&self) -> Option<&ForeignBlock> {
        if let ContentItem::ForeignBlock(fb) = self {
            Some(fb)
        } else {
            None
        }
    }

    pub fn as_paragraph_mut(&mut self) -> Option<&mut Paragraph> {
        if let ContentItem::Paragraph(p) = self {
            Some(p)
        } else {
            None
        }
    }
    pub fn as_session_mut(&mut self) -> Option<&mut Session> {
        if let ContentItem::Session(s) = self {
            Some(s)
        } else {
            None
        }
    }
    pub fn as_list_mut(&mut self) -> Option<&mut List> {
        if let ContentItem::List(l) = self {
            Some(l)
        } else {
            None
        }
    }
    pub fn as_list_item_mut(&mut self) -> Option<&mut ListItem> {
        if let ContentItem::ListItem(li) = self {
            Some(li)
        } else {
            None
        }
    }
    pub fn as_definition_mut(&mut self) -> Option<&mut Definition> {
        if let ContentItem::Definition(d) = self {
            Some(d)
        } else {
            None
        }
    }
    pub fn as_annotation_mut(&mut self) -> Option<&mut Annotation> {
        if let ContentItem::Annotation(a) = self {
            Some(a)
        } else {
            None
        }
    }
    pub fn as_foreign_block_mut(&mut self) -> Option<&mut ForeignBlock> {
        if let ContentItem::ForeignBlock(fb) = self {
            Some(fb)
        } else {
            None
        }
    }

    /// Find all elements at the given position in this item and its children
    /// Returns elements in order from deepest to shallowest nesting
    pub fn elements_at(&self, pos: Position) -> Option<Vec<&ContentItem>> {
        // Check nested items first - even if parent location doesn't contain position,
        // nested elements might. This is important because parent locations (like sessions)
        // may only cover their title, not their nested content.
        let mut results = Vec::new();
        if let Some(children) = self.children() {
            for child in children {
                if let Some(mut child_results) = child.elements_at(pos) {
                    results.append(&mut child_results);
                    break; // Only one branch can contain the position
                }
            }
        }

        // Now, check the current item. An item is considered to be at the position if its
        // location contains the position.
        // If nested results were found, this item is a parent and should be added to the
        // list to form the stack of elements.
        // If no nested results were found, this item is the innermost element at the position.
        if self.location().map_or(false, |l| l.contains(pos)) {
            results.push(self);
        }

        if results.is_empty() {
            None
        } else {
            Some(results)
        }
    }
}

impl fmt::Display for ContentItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContentItem::Paragraph(p) => write!(f, "Paragraph({} lines)", p.lines.len()),
            ContentItem::Session(s) => {
                write!(
                    f,
                    "Session('{}', {} items)",
                    s.title.as_string(),
                    s.content.len()
                )
            }
            ContentItem::List(l) => write!(f, "List({} items)", l.content.len()),
            ContentItem::ListItem(li) => {
                write!(f, "ListItem('{}', {} items)", li.text(), li.content.len())
            }
            ContentItem::TextLine(tl) => {
                write!(f, "TextLine('{}')", tl.text())
            }
            ContentItem::Definition(d) => {
                write!(
                    f,
                    "Definition('{}', {} items)",
                    d.subject.as_string(),
                    d.content.len()
                )
            }
            ContentItem::Annotation(a) => write!(
                f,
                "Annotation('{}', {} params, {} items)",
                a.label.value,
                a.parameters.len(),
                a.content.len()
            ),
            ContentItem::ForeignBlock(fb) => {
                write!(f, "ForeignBlock('{}')", fb.subject.as_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::location::{Location, Position};
    use super::super::paragraph::Paragraph;
    use super::*;

    #[test]
    fn test_elements_at_simple_paragraph() {
        let para = Paragraph::from_line("Test".to_string()).with_location(Location::new(
            Position::new(0, 0),
            Position::new(0, 4),
        ));
        let item = ContentItem::Paragraph(para);

        let pos = Position::new(0, 2);
        if let Some(results) = item.elements_at(pos) {
            // Paragraphs now expose their TextLine children, so we get both the paragraph and the line
            assert_eq!(results.len(), 2);
            assert!(results[0].is_text_line()); // Innermost should be the TextLine
            assert!(results[1].is_paragraph()); // Outer should be the paragraph
        } else {
            panic!("Expected to find paragraph at position");
        }
    }

    #[test]
    fn test_elements_at_position_outside_location() {
        let para = Paragraph::from_line("Test".to_string()).with_location(Location::new(
            Position::new(0, 0),
            Position::new(0, 4),
        ));
        let item = ContentItem::Paragraph(para);

        let pos = Position::new(0, 10);
        let results = item.elements_at(pos);
        assert!(results.is_none());
    }

    #[test]
    fn test_elements_at_no_location() {
        // Item with no location should not match any position
        let para = Paragraph::from_line("Test".to_string());
        let item = ContentItem::Paragraph(para);

        let pos = Position::new(5, 10);
        assert!(item.elements_at(pos).is_none());
    }

    #[test]
    fn test_elements_at_nested_session() {
        let para = Paragraph::from_line("Nested".to_string()).with_location(Location::new(
            Position::new(1, 0),
            Position::new(1, 6),
        ));
        let session = Session::new(
            super::super::super::text_content::TextContent::from_string(
                "Section".to_string(),
                None,
            ),
            vec![ContentItem::Paragraph(para)],
        )
        .with_location(Location::new(
            Position::new(0, 0),
            Position::new(2, 0),
        ));
        let item = ContentItem::Session(session);

        let pos = Position::new(1, 3);
        if let Some(results) = item.elements_at(pos) {
            // Now we get: TextLine (deepest), Paragraph, Session (shallowest)
            assert_eq!(results.len(), 3);
            assert!(results[0].is_text_line());
            assert!(results[1].is_paragraph());
            assert!(results[2].is_session());
        } else {
            panic!("Expected to find session and paragraph");
        }
    }
}
