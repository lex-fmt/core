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
    pub content: Vec<ContentItem>,
    pub location: Option<Location>,
}

/// A list item has text and optional nested content
#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    pub(crate) text: Vec<TextContent>,
    pub content: Vec<ContentItem>,
    pub location: Option<Location>,
}

impl List {
    pub fn new(items: Vec<ContentItem>) -> Self {
        Self {
            content: items,
            location: None,
        }
    }
    pub fn with_location(mut self, location: Option<Location>) -> Self {
        self.location = location;
        self
    }
}

impl AstNode for List {
    fn node_type(&self) -> &'static str {
        "List"
    }
    fn display_label(&self) -> String {
        format!("{} items", self.content.len())
    }
    fn location(&self) -> Option<Location> {
        self.location
    }
}

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "List({} items)", self.content.len())
    }
}

impl ListItem {
    pub fn new(text: String) -> Self {
        Self {
            text: vec![TextContent::from_string(text, None)],
            content: Vec::new(),
            location: None,
        }
    }
    pub fn with_content(text: String, content: Vec<ContentItem>) -> Self {
        Self {
            text: vec![TextContent::from_string(text, None)],
            content,
            location: None,
        }
    }
    /// Create a ListItem with TextContent that may have location information
    pub fn with_text_content(text_content: TextContent, content: Vec<ContentItem>) -> Self {
        Self {
            text: vec![text_content],
            content,
            location: None,
        }
    }
    pub fn with_location(mut self, location: Option<Location>) -> Self {
        self.location = location;
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
        self.location
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
    fn test_list_with_location() {
        let location = super::super::super::location::Location::new(
            super::super::super::location::Position::new(1, 0),
            super::super::super::location::Position::new(1, 10),
        );
        let list = List::new(vec![]).with_location(Some(location));
        assert_eq!(list.location, Some(location));
    }
}
