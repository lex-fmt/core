//! ContentItem enum definition

use super::super::span::Position;
use super::super::traits::{AstNode, Container, NodeStartLocation};
use super::annotation::Annotation;
use super::definition::Definition;
use super::foreign::ForeignBlock;
use super::list::List;
use super::paragraph::Paragraph;
use super::session::Session;
use std::fmt;

/// ContentItem represents any element that can appear in document content
#[derive(Debug, Clone, PartialEq)]
pub enum ContentItem {
    Paragraph(Paragraph),
    Session(Session),
    List(List),
    Definition(Definition),
    Annotation(Annotation),
    ForeignBlock(ForeignBlock),
}

impl ContentItem {
    pub fn node_type(&self) -> &'static str {
        match self {
            ContentItem::Paragraph(p) => p.node_type(),
            ContentItem::Session(s) => s.node_type(),
            ContentItem::List(l) => l.node_type(),
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
            ContentItem::ForeignBlock(fb) => Some(fb.subject.as_string()),
            _ => None,
        }
    }

    pub fn children(&self) -> Option<&[ContentItem]> {
        match self {
            ContentItem::Session(s) => Some(s.children()),
            ContentItem::Definition(d) => Some(d.children()),
            ContentItem::Annotation(a) => Some(a.children()),
            _ => None,
        }
    }

    pub fn children_mut(&mut self) -> Option<&mut Vec<ContentItem>> {
        match self {
            ContentItem::Session(s) => Some(s.children_mut()),
            ContentItem::Definition(d) => Some(d.children_mut()),
            ContentItem::Annotation(a) => Some(a.children_mut()),
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
        // Check if this item contains the position
        let span = match self {
            ContentItem::Paragraph(p) => p.span,
            ContentItem::Session(s) => s.span,
            ContentItem::List(l) => l.span,
            ContentItem::Definition(d) => d.span,
            ContentItem::Annotation(a) => a.span,
            ContentItem::ForeignBlock(fb) => fb.span,
        };

        if let Some(span) = span {
            if !span.contains(pos) {
                return None;
            }
        }

        // Position is in this item, now check children
        let mut results = vec![self];

        // Check nested items
        let children = self.children();
        if let Some(children) = children {
            for child in children {
                if let Some(mut child_results) = child.elements_at(pos) {
                    results.append(&mut child_results);
                    break; // Only one branch can contain the position
                }
            }
        }

        Some(results)
    }
}

impl NodeStartLocation for ContentItem {
    fn start_location(&self) -> Option<Position> {
        match self {
            ContentItem::Paragraph(p) => p.start_location(),
            ContentItem::Session(s) => s.start_location(),
            ContentItem::List(l) => l.start_location(),
            ContentItem::Definition(d) => d.start_location(),
            ContentItem::Annotation(a) => a.start_location(),
            ContentItem::ForeignBlock(fb) => fb.start_location(),
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
            ContentItem::List(l) => write!(f, "List({} items)", l.items.len()),
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
    use super::super::super::span::{Position, Span};
    use super::super::paragraph::Paragraph;
    use super::*;

    #[test]
    fn test_elements_at_simple_paragraph() {
        let para = Paragraph::from_line("Test".to_string())
            .with_span(Some(Span::new(Position::new(0, 0), Position::new(0, 4))));
        let item = ContentItem::Paragraph(para);

        let pos = Position::new(0, 2);
        if let Some(results) = item.elements_at(pos) {
            assert_eq!(results.len(), 1);
            assert!(results[0].is_paragraph());
        } else {
            panic!("Expected to find paragraph at position");
        }
    }

    #[test]
    fn test_elements_at_position_outside_span() {
        let para = Paragraph::from_line("Test".to_string())
            .with_span(Some(Span::new(Position::new(0, 0), Position::new(0, 4))));
        let item = ContentItem::Paragraph(para);

        let pos = Position::new(0, 10);
        let results = item.elements_at(pos);
        assert!(results.is_none());
    }

    #[test]
    fn test_elements_at_no_span() {
        // Item with no span should match any position
        let para = Paragraph::from_line("Test".to_string());
        let item = ContentItem::Paragraph(para);

        let pos = Position::new(5, 10);
        if let Some(results) = item.elements_at(pos) {
            assert_eq!(results.len(), 1);
            assert!(results[0].is_paragraph());
        } else {
            panic!("Expected to find paragraph when no span is set");
        }
    }

    #[test]
    fn test_elements_at_nested_session() {
        let para = Paragraph::from_line("Nested".to_string())
            .with_span(Some(Span::new(Position::new(1, 0), Position::new(1, 6))));
        let session = Session::new(
            super::super::super::text_content::TextContent::from_string(
                "Section".to_string(),
                None,
            ),
            vec![ContentItem::Paragraph(para)],
        )
        .with_span(Some(Span::new(Position::new(0, 0), Position::new(2, 0))));
        let item = ContentItem::Session(session);

        let pos = Position::new(1, 3);
        if let Some(results) = item.elements_at(pos) {
            assert_eq!(results.len(), 2);
            assert!(results[0].is_session());
            assert!(results[1].is_paragraph());
        } else {
            panic!("Expected to find session and paragraph");
        }
    }
}
