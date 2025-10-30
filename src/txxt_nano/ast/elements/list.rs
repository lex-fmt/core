//! List element definition

use super::super::location::Location;
use super::super::text_content::TextContent;
use super::super::traits::AstNode;
use super::super::traits::Container;
use super::content_item::ContentItem;
use std::fmt;

/// A list contains multiple list items
#[derive(Debug, Clone, PartialEq)]
pub struct List {
    pub items: Vec<ListItem>,
    pub span: Option<Location>,
}

/// A list item has text and optional nested content
#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    pub(crate) text: Vec<TextContent>,
    pub content: Vec<ContentItem>,
    pub span: Option<Location>,
}

impl List {
    pub fn new(items: Vec<ListItem>) -> Self {
        Self { items, span: None }
    }
    pub fn with_span(mut self, span: Option<Location>) -> Self {
        self.span = span;
        self
    }
}

impl AstNode for List {
    fn node_type(&self) -> &'static str {
        "List"
    }
    fn display_label(&self) -> String {
        format!("{} items", self.items.len())
    }
    fn location(&self) -> Option<Location> {
        self.span
    }
}

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "List({} items)", self.items.len())
    }
}

impl ListItem {
    pub fn new(text: String) -> Self {
        Self {
            text: vec![TextContent::from_string(text, None)],
            content: Vec::new(),
            span: None,
        }
    }
    pub fn with_content(text: String, content: Vec<ContentItem>) -> Self {
        Self {
            text: vec![TextContent::from_string(text, None)],
            content,
            span: None,
        }
    }
    /// Create a ListItem with TextContent that may have span information
    pub fn with_text_content(text_content: TextContent, content: Vec<ContentItem>) -> Self {
        Self {
            text: vec![text_content],
            content,
            span: None,
        }
    }
    pub fn with_span(mut self, span: Option<Location>) -> Self {
        self.span = span;
        self
    }
    pub fn text(&self) -> &str {
        self.text[0].as_string()
    }
}

impl AstNode for ListItem {
    fn node_type(&self) -> &'static str {
        "ListItem"
    }
    fn display_label(&self) -> String {
        let text = self.text();
        if text.len() > 50 {
            format!("{}...", &text[..50])
        } else {
            text.to_string()
        }
    }
    fn location(&self) -> Option<Location> {
        self.span
    }
}

impl Container for ListItem {
    fn label(&self) -> &str {
        self.text[0].as_string()
    }
    fn children(&self) -> &[ContentItem] {
        &self.content
    }
    fn children_mut(&mut self) -> &mut Vec<ContentItem> {
        &mut self.content
    }
}

impl fmt::Display for ListItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ListItem('{}')", self.text())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_with_span() {
        let span = super::super::super::location::Location::new(
            super::super::super::location::Position::new(1, 0),
            super::super::super::location::Position::new(1, 10),
        );
        let list = List::new(vec![]).with_span(Some(span));
        assert_eq!(list.span, Some(span));
    }
}
