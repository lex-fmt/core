//! TextContent facade for representing user content text
//!
//! This module provides the `TextContent` type, which serves as a stable
//! interface for user-provided text throughout the AST. The facade is designed
//! to evolve over time:
//!
//! - **Phase 1 (current):** Plain text strings with source position tracking
//! - **Phase 2 (future):** Parsed inline elements (bold, italic, links, etc.)
//!
//! By using a facade, we can evolve from Phase 1 to Phase 2 without changing
//! the AST node types. External code accesses text via stable API methods
//! (.as_string(), future: .as_inlines()), which work regardless of the
//! internal representation.

use super::location::Location;

/// Represents user-provided text content with source position tracking.
///
/// TextContent acts as a facade over different internal representations,
/// allowing the text layer to evolve without breaking the AST structure.
/// Currently stores plain text; future versions will support parsed inline nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct TextContent {
    /// Span in the source covering this text
    pub span: Option<Location>,
    /// Internal representation (evolves over time)
    inner: TextRepresentation,
}

/// Internal representation of text content.
///
/// This enum encapsulates the actual text storage format. It can evolve
/// without changing the public TextContent API.
#[derive(Debug, Clone, PartialEq)]
enum TextRepresentation {
    /// Plain text as a String.
    /// May contain formatting markers like "**bold**" or "_italic_"
    /// that will be parsed in Phase 2.
    Text(String),
    // Future variants (Phase 2):
    // Inlines(Vec<InlineNode>),
}

impl TextContent {
    /// Create TextContent from a string and optional source span.
    ///
    /// # Arguments
    /// * `text` - The raw text content
    /// * `span` - Optional source location of this text
    ///
    /// # Example
    /// ```ignore
    /// let content = TextContent::from_string("Hello world".to_string(), Some(span));
    /// ```
    pub fn from_string(text: String, span: Option<Location>) -> Self {
        Self {
            span,
            inner: TextRepresentation::Text(text),
        }
    }

    /// Create empty TextContent.
    pub fn empty() -> Self {
        Self {
            span: None,
            inner: TextRepresentation::Text(String::new()),
        }
    }

    /// Get the text content as a string slice.
    ///
    /// Works regardless of internal representation. In Phase 1, returns the
    /// stored string directly. In Phase 2, would flatten inline nodes to text.
    ///
    /// # Example
    /// ```ignore
    /// let text = content.as_string();
    /// assert_eq!(text, "Hello world");
    /// ```
    pub fn as_string(&self) -> &str {
        match &self.inner {
            TextRepresentation::Text(s) => s,
        }
    }

    /// Get mutable access to the text content.
    ///
    /// Note: Only available in Phase 1. Once inlines are parsed,
    /// you would need to reconstruct inlines after mutations.
    ///
    /// # Panics
    /// In Phase 2, this may panic or return an error if inlines have been parsed.
    pub fn as_string_mut(&mut self) -> &mut String {
        match &mut self.inner {
            TextRepresentation::Text(s) => s,
        }
    }

    /// Check if content is empty.
    pub fn is_empty(&self) -> bool {
        self.as_string().is_empty()
    }

    /// Get the length of the content in characters.
    pub fn len(&self) -> usize {
        self.as_string().len()
    }

    // Future API (Phase 2 placeholders):
    // pub fn as_inlines(&self) -> Option<&[InlineNode]> { ... }
    // pub fn parse_inlines(&mut self) -> Result<()> { ... }
}

impl Default for TextContent {
    fn default() -> Self {
        Self::empty()
    }
}

impl From<String> for TextContent {
    fn from(text: String) -> Self {
        Self::from_string(text, None)
    }
}

impl From<&str> for TextContent {
    fn from(text: &str) -> Self {
        Self::from_string(text.to_string(), None)
    }
}

impl AsRef<str> for TextContent {
    fn as_ref(&self) -> &str {
        self.as_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_from_string() {
        let content = TextContent::from_string("Hello".to_string(), None);
        assert_eq!(content.as_string(), "Hello");
    }

    #[test]
    fn test_empty() {
        let content = TextContent::empty();
        assert!(content.is_empty());
        assert_eq!(content.len(), 0);
    }

    #[test]
    fn test_from_string_trait() {
        let content = TextContent::from("Hello".to_string());
        assert_eq!(content.as_string(), "Hello");
    }

    #[test]
    fn test_from_str_trait() {
        let content = TextContent::from("Hello");
        assert_eq!(content.as_string(), "Hello");
    }

    #[test]
    fn test_as_ref() {
        let content = TextContent::from("Hello");
        let text: &str = content.as_ref();
        assert_eq!(text, "Hello");
    }

    #[test]
    fn test_with_span() {
        let span = Location::new(Position::new(0, 0), Position::new(0, 5));
        let content = TextContent::from_string("Hello".to_string(), Some(span));
        assert_eq!(content.span, Some(span));
    }

    #[test]
    fn test_mutate() {
        let mut content = TextContent::from_string("Hello".to_string(), None);
        *content.as_string_mut() = "World".to_string();
        assert_eq!(content.as_string(), "World");
    }

    use super::super::location::Position;
}
