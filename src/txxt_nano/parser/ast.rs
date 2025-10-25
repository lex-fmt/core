//! Abstract Syntax Tree (AST) definitions for the txxt format
//!
//! This module defines the data structures that represent the parsed
//! structure of a txxt document.
//!
//! ## Testing
//!
//! All parser tests must follow strict guidelines. See the [testing module](crate::txxt_nano::testing)
//! for comprehensive documentation on using verified txxt sources and AST assertions.

use std::fmt;

// ============================================================================
// AST Traits - Common interfaces for uniform node access
// ============================================================================
//
// These traits provide a uniform interface for working with different AST node types.
// This is crucial for generic algorithms like tree traversal, serialization, and testing.
//
// ## Design Philosophy
//
// AST nodes have semantic field names (`title`, `content`, `subject`, `item_line`) that
// describe their specific role, but generic code needs uniform access. These traits bridge
// that gap:
//
// - Struct fields: Semantic names (`Session.title`, `Session.content`)
// - Trait methods: Standard names (`session.label()`, `session.children()`)
//
// ## Usage
//
// **Testing**: The assertion library uses these traits to provide uniform assertions
// across all node types without knowing their concrete types.
//
// **Serialization**: A generic XML/JSON serializer can traverse any AST node using
// `label()` and `children()` without knowing if it's a Session, ListItem, or Definition.
//
// **Tree visualization**: Debug printers can show any tree structure using the common
// interface, automatically formatting nested structures regardless of node type.
//
// ## Example
//
// ```rust
// use txxt_nano::parser::ast::{Container, Session, ContentItem, Paragraph};
//
// let session = Session::new("Introduction".to_string(), vec![
//     ContentItem::Paragraph(Paragraph::from_line("Hello".to_string()))
// ]);
//
// // Access through trait - works for Session, ListItem, Definition, etc.
// assert_eq!(session.label(), "Introduction");
// assert_eq!(session.children().len(), 1);
// ```

/// Common interface for all AST nodes
pub trait AstNode {
    /// Get the node type name for display/debugging
    fn node_type(&self) -> &'static str;

    /// Get the display label for this node (for tree visualization)
    /// - For containers (Session, ListItem, etc): returns the title/subject
    /// - For leaf nodes (Paragraph): returns first N chars of content
    fn display_label(&self) -> String;
}

/// Trait for container nodes that have a label and children
///
/// Container nodes have a "title-like" identifier and nested content.
/// Examples: Session (has title), ListItem (has item line), Definition (has subject).
pub trait Container: AstNode {
    /// Get the label/title/subject of this container
    ///
    /// Maps to semantic field: `title`, `subject`, `item_line`, etc.
    fn label(&self) -> &str;

    /// Get the children of this container
    ///
    /// Maps to semantic field: `content`
    fn children(&self) -> &[ContentItem];

    /// Get a mutable reference to children (for tree manipulation)
    fn children_mut(&mut self) -> &mut Vec<ContentItem>;
}

/// Trait for leaf nodes that contain text
///
/// Text nodes are leaves in the AST tree - they have content but no children.
/// Example: Paragraph.
pub trait TextNode: AstNode {
    /// Get the text content of this node
    fn text(&self) -> String;

    /// Get the lines that make up this text
    fn lines(&self) -> &[String];
}

// ============================================================================
// AST Node Definitions
// ============================================================================

/// A complete txxt document
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    /// Top-level content items (paragraphs and sessions)
    pub items: Vec<ContentItem>,
}

/// A content item can be a paragraph, session, list, definition, annotation, or foreign block
#[derive(Debug, Clone, PartialEq)]
pub enum ContentItem {
    Paragraph(Paragraph),
    Session(Session),
    List(List),
    Definition(Definition),
    Annotation(Annotation),
    ForeignBlock(ForeignBlock),
}

/// A paragraph is a block of text content
#[derive(Debug, Clone, PartialEq)]
pub struct Paragraph {
    /// The text lines that make up this paragraph
    pub lines: Vec<String>,
}

/// A session is a titled section that can contain nested content
#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    /// The title of the session
    pub title: String,
    /// Content items within this session (paragraphs and nested sessions)
    pub content: Vec<ContentItem>,
}

/// A list is a sequence of list items (requires at least 2 items)
#[derive(Debug, Clone, PartialEq)]
pub struct List {
    /// The items in this list
    pub items: Vec<ListItem>,
}

/// A list item is a single entry in a list
#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    /// The text content of this list item (including the marker)
    /// Stored as a single-element vector to maintain backward compatibility
    text: Vec<String>,
    /// Nested content within this list item (paragraphs and lists)
    /// Will be empty if there's no nested content
    pub content: Vec<ContentItem>,
}

/// A definition is a subject followed by its definition content
#[derive(Debug, Clone, PartialEq)]
pub struct Definition {
    /// The subject being defined (without the trailing colon)
    pub subject: String,
    /// Content that defines the subject (paragraphs and lists, no sessions)
    pub content: Vec<ContentItem>,
}

/// A foreign block is a subject followed by unparsed content and a closing annotation
#[derive(Debug, Clone, PartialEq)]
pub struct ForeignBlock {
    /// The subject of the foreign block (without the trailing colon)
    pub subject: String,
    /// The raw, unparsed content of the block
    pub content: String,
    /// The mandatory closing annotation
    pub closing_annotation: Annotation,
}

/// A label for an annotation - can be simple (note) or namespaced (python.typing)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Label {
    /// The full label string (e.g., "note" or "python.typing")
    pub value: String,
}

/// A parameter for an annotation - key=value pair or boolean shorthand
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    /// The parameter key
    pub key: String,
    /// The parameter value (None for boolean shorthand, Some for explicit values)
    pub value: Option<String>,
}

/// An annotation is a metadata element with label, optional parameters, and optional content
#[derive(Debug, Clone, PartialEq)]
pub struct Annotation {
    /// The annotation label (e.g., "note", "warning", "python.typing")
    pub label: Label,
    /// Optional parameters (key=value pairs or boolean shorthand)
    pub parameters: Vec<Parameter>,
    /// Optional content (paragraphs and lists, no sessions or nested annotations)
    pub content: Vec<ContentItem>,
}

impl Document {
    /// Create a new empty document
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Create a document with the given items
    pub fn with_items(items: Vec<ContentItem>) -> Self {
        Self { items }
    }

    // ========================================================================
    // Iterator methods for type-specific access
    // ========================================================================

    /// Iterate over all top-level content items
    pub fn iter_items(&self) -> impl Iterator<Item = &ContentItem> {
        self.items.iter()
    }

    /// Iterate over all top-level paragraphs only
    pub fn iter_paragraphs(&self) -> impl Iterator<Item = &Paragraph> {
        self.items.iter().filter_map(|item| item.as_paragraph())
    }

    /// Iterate over all top-level sessions only
    pub fn iter_sessions(&self) -> impl Iterator<Item = &Session> {
        self.items.iter().filter_map(|item| item.as_session())
    }

    /// Iterate over all top-level lists only
    pub fn iter_lists(&self) -> impl Iterator<Item = &List> {
        self.items.iter().filter_map(|item| item.as_list())
    }

    /// Iterate over all top-level foreign blocks only
    pub fn iter_foreign_blocks(&self) -> impl Iterator<Item = &ForeignBlock> {
        self.items.iter().filter_map(|item| item.as_foreign_block())
    }

    /// Count items by type
    pub fn count_by_type(&self) -> (usize, usize, usize, usize) {
        let paragraphs = self.iter_paragraphs().count();
        let sessions = self.iter_sessions().count();
        let lists = self.iter_lists().count();
        let foreign_blocks = self.iter_foreign_blocks().count();
        (paragraphs, sessions, lists, foreign_blocks)
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Paragraph {
    /// Create a new paragraph with the given lines
    pub fn new(lines: Vec<String>) -> Self {
        Self { lines }
    }

    /// Create a paragraph from a single line
    pub fn from_line(line: String) -> Self {
        Self { lines: vec![line] }
    }

    /// Get the full text of the paragraph
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }
}

impl Session {
    /// Create a new session with the given title and content
    pub fn new(title: String, content: Vec<ContentItem>) -> Self {
        Self { title, content }
    }

    /// Create a session with just a title and no content
    pub fn with_title(title: String) -> Self {
        Self {
            title,
            content: Vec::new(),
        }
    }
}

impl List {
    /// Create a new list with the given items
    pub fn new(items: Vec<ListItem>) -> Self {
        Self { items }
    }
}

impl ListItem {
    /// Create a new list item with the given text
    pub fn new(text: String) -> Self {
        Self {
            text: vec![text],
            content: Vec::new(),
        }
    }

    /// Create a new list item with text and content
    pub fn with_content(text: String, content: Vec<ContentItem>) -> Self {
        Self {
            text: vec![text],
            content,
        }
    }

    /// Get the text content of this list item
    pub fn text(&self) -> &str {
        &self.text[0]
    }
}

impl Definition {
    /// Create a new definition with the given subject and content
    pub fn new(subject: String, content: Vec<ContentItem>) -> Self {
        Self { subject, content }
    }

    /// Create a definition with just a subject and no content
    pub fn with_subject(subject: String) -> Self {
        Self {
            subject,
            content: Vec::new(),
        }
    }
}

impl Label {
    /// Create a new label from a string
    pub fn new(value: String) -> Self {
        Self { value }
    }

    /// Create a label from a string slice
    pub fn from_string(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }
}

impl Parameter {
    /// Create a new parameter with key and value
    pub fn new(key: String, value: Option<String>) -> Self {
        Self { key, value }
    }

    /// Create a boolean parameter (shorthand, no value)
    pub fn boolean(key: String) -> Self {
        Self { key, value: None }
    }

    /// Create a parameter with a string value
    pub fn with_value(key: String, value: String) -> Self {
        Self {
            key,
            value: Some(value),
        }
    }
}

impl Annotation {
    /// Create a new annotation with all fields
    pub fn new(label: Label, parameters: Vec<Parameter>, content: Vec<ContentItem>) -> Self {
        Self {
            label,
            parameters,
            content,
        }
    }

    /// Create a marker-form annotation (label only, no parameters or content)
    pub fn marker(label: Label) -> Self {
        Self {
            label,
            parameters: Vec::new(),
            content: Vec::new(),
        }
    }

    /// Create an annotation with label and parameters only (no content)
    pub fn with_parameters(label: Label, parameters: Vec<Parameter>) -> Self {
        Self {
            label,
            parameters,
            content: Vec::new(),
        }
    }
}

impl ForeignBlock {
    /// Create a new foreign block with all fields
    pub fn new(subject: String, content: String, closing_annotation: Annotation) -> Self {
        Self {
            subject,
            content,
            closing_annotation,
        }
    }

    /// Create a foreign block with no content (marker form)
    pub fn marker(subject: String, closing_annotation: Annotation) -> Self {
        Self {
            subject,
            content: String::new(),
            closing_annotation,
        }
    }
}

impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Document({} items)", self.items.len())
    }
}

impl fmt::Display for ContentItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContentItem::Paragraph(p) => write!(f, "Paragraph({} lines)", p.lines.len()),
            ContentItem::Session(s) => {
                write!(f, "Session('{}', {} items)", s.title, s.content.len())
            }
            ContentItem::List(l) => write!(f, "List({} items)", l.items.len()),
            ContentItem::Definition(d) => {
                write!(f, "Definition('{}', {} items)", d.subject, d.content.len())
            }
            ContentItem::Annotation(a) => {
                write!(
                    f,
                    "Annotation('{}', {} params, {} items)",
                    a.label.value,
                    a.parameters.len(),
                    a.content.len()
                )
            }
            ContentItem::ForeignBlock(fb) => {
                write!(f, "ForeignBlock('{}')", fb.subject)
            }
        }
    }
}

impl fmt::Display for Paragraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Paragraph({} lines)", self.lines.len())
    }
}

impl fmt::Display for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Session('{}', {} items)", self.title, self.content.len())
    }
}

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "List({} items)", self.items.len())
    }
}

impl fmt::Display for ListItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ListItem('{}')", self.text())
    }
}

impl fmt::Display for Definition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Definition('{}', {} items)",
            self.subject,
            self.content.len()
        )
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Display for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            Some(v) => write!(f, "{}={}", self.key, v),
            None => write!(f, "{}", self.key),
        }
    }
}

impl fmt::Display for Annotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Annotation('{}', {} params, {} items)",
            self.label.value,
            self.parameters.len(),
            self.content.len()
        )
    }
}

impl fmt::Display for ForeignBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ForeignBlock('{}', {} chars, closing: {})",
            self.subject,
            self.content.len(),
            self.closing_annotation.label.value
        )
    }
}

// ============================================================================
// Trait Implementations
// ============================================================================

// Paragraph - TextNode implementation
impl AstNode for Paragraph {
    fn node_type(&self) -> &'static str {
        "Paragraph"
    }

    fn display_label(&self) -> String {
        let text = self.text();
        if text.len() > 50 {
            format!("{}...", &text[..50])
        } else {
            text
        }
    }
}

impl TextNode for Paragraph {
    fn text(&self) -> String {
        self.lines.join("\n")
    }

    fn lines(&self) -> &[String] {
        &self.lines
    }
}

// Session - Container implementation
impl AstNode for Session {
    fn node_type(&self) -> &'static str {
        "Session"
    }

    fn display_label(&self) -> String {
        self.title.clone()
    }
}

impl Container for Session {
    fn label(&self) -> &str {
        &self.title
    }

    fn children(&self) -> &[ContentItem] {
        &self.content
    }

    fn children_mut(&mut self) -> &mut Vec<ContentItem> {
        &mut self.content
    }
}

// List - AstNode implementation
impl AstNode for List {
    fn node_type(&self) -> &'static str {
        "List"
    }

    fn display_label(&self) -> String {
        format!("{} items", self.items.len())
    }
}

// ListItem - AstNode and Container implementation
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
}

impl Container for ListItem {
    fn label(&self) -> &str {
        &self.text[0]
    }

    fn children(&self) -> &[ContentItem] {
        &self.content
    }

    fn children_mut(&mut self) -> &mut Vec<ContentItem> {
        &mut self.content
    }
}

// Definition - AstNode and Container implementation
impl AstNode for Definition {
    fn node_type(&self) -> &'static str {
        "Definition"
    }

    fn display_label(&self) -> String {
        if self.subject.len() > 50 {
            format!("{}...", &self.subject[..50])
        } else {
            self.subject.clone()
        }
    }
}

impl Container for Definition {
    fn label(&self) -> &str {
        &self.subject
    }

    fn children(&self) -> &[ContentItem] {
        &self.content
    }

    fn children_mut(&mut self) -> &mut Vec<ContentItem> {
        &mut self.content
    }
}

// Annotation - AstNode and Container implementation
impl AstNode for Annotation {
    fn node_type(&self) -> &'static str {
        "Annotation"
    }

    fn display_label(&self) -> String {
        // Show label and parameter count
        if self.parameters.is_empty() {
            self.label.value.clone()
        } else {
            format!("{} ({} params)", self.label.value, self.parameters.len())
        }
    }
}

impl Container for Annotation {
    fn label(&self) -> &str {
        &self.label.value
    }

    fn children(&self) -> &[ContentItem] {
        &self.content
    }

    fn children_mut(&mut self) -> &mut Vec<ContentItem> {
        &mut self.content
    }
}

// ForeignBlock - AstNode implementation (NOT Container - content is raw string)
impl AstNode for ForeignBlock {
    fn node_type(&self) -> &'static str {
        "ForeignBlock"
    }

    fn display_label(&self) -> String {
        if self.subject.len() > 50 {
            format!("{}...", &self.subject[..50])
        } else {
            self.subject.clone()
        }
    }
}

// ContentItem - Helper methods for trait access
impl ContentItem {
    /// Get the node type name (delegates to AstNode trait)
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

    /// Get the display label (delegates to AstNode trait)
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

    /// Get the label if this is a container node
    pub fn label(&self) -> Option<&str> {
        match self {
            ContentItem::Session(s) => Some(s.label()),
            ContentItem::Definition(d) => Some(d.label()),
            ContentItem::Annotation(a) => Some(a.label()),
            ContentItem::ForeignBlock(fb) => Some(&fb.subject),
            ContentItem::Paragraph(_) => None,
            ContentItem::List(_) => None,
        }
    }

    /// Get the children if this is a container node
    pub fn children(&self) -> Option<&[ContentItem]> {
        match self {
            ContentItem::Session(s) => Some(s.children()),
            ContentItem::Definition(d) => Some(d.children()),
            ContentItem::Annotation(a) => Some(a.children()),
            ContentItem::ForeignBlock(_) => None, // ForeignBlock is NOT a container
            ContentItem::Paragraph(_) => None,
            ContentItem::List(_) => None,
        }
    }

    /// Get mutable children if this is a container node
    pub fn children_mut(&mut self) -> Option<&mut Vec<ContentItem>> {
        match self {
            ContentItem::Session(s) => Some(s.children_mut()),
            ContentItem::Definition(d) => Some(d.children_mut()),
            ContentItem::Annotation(a) => Some(a.children_mut()),
            ContentItem::ForeignBlock(_) => None, // ForeignBlock is NOT a container
            ContentItem::Paragraph(_) => None,
            ContentItem::List(_) => None,
        }
    }

    /// Get the text content if this is a text node
    pub fn text(&self) -> Option<String> {
        match self {
            ContentItem::Paragraph(p) => Some(p.text()),
            ContentItem::Session(_) => None,
            ContentItem::List(_) => None,
            ContentItem::Definition(_) => None,
            ContentItem::Annotation(_) => None,
            ContentItem::ForeignBlock(_) => None,
        }
    }

    // ========================================================================
    // Type checking methods
    // ========================================================================

    /// Check if this item is a Paragraph
    pub fn is_paragraph(&self) -> bool {
        matches!(self, ContentItem::Paragraph(_))
    }

    /// Check if this item is a Session
    pub fn is_session(&self) -> bool {
        matches!(self, ContentItem::Session(_))
    }

    /// Check if this item is a List
    pub fn is_list(&self) -> bool {
        matches!(self, ContentItem::List(_))
    }

    /// Check if this item is a Definition
    pub fn is_definition(&self) -> bool {
        matches!(self, ContentItem::Definition(_))
    }

    /// Check if this item is an Annotation
    pub fn is_annotation(&self) -> bool {
        matches!(self, ContentItem::Annotation(_))
    }

    /// Check if this item is a ForeignBlock
    pub fn is_foreign_block(&self) -> bool {
        matches!(self, ContentItem::ForeignBlock(_))
    }

    // ========================================================================
    // Safe extraction methods (Option-returning)
    // ========================================================================

    /// Get a reference to the Paragraph if this is a Paragraph variant
    pub fn as_paragraph(&self) -> Option<&Paragraph> {
        if let ContentItem::Paragraph(p) = self {
            Some(p)
        } else {
            None
        }
    }

    /// Get a reference to the Session if this is a Session variant
    pub fn as_session(&self) -> Option<&Session> {
        if let ContentItem::Session(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Get a reference to the List if this is a List variant
    pub fn as_list(&self) -> Option<&List> {
        if let ContentItem::List(l) = self {
            Some(l)
        } else {
            None
        }
    }

    /// Get a reference to the Definition if this is a Definition variant
    pub fn as_definition(&self) -> Option<&Definition> {
        if let ContentItem::Definition(d) = self {
            Some(d)
        } else {
            None
        }
    }

    /// Get a mutable reference to the Paragraph if this is a Paragraph variant
    pub fn as_paragraph_mut(&mut self) -> Option<&mut Paragraph> {
        if let ContentItem::Paragraph(p) = self {
            Some(p)
        } else {
            None
        }
    }

    /// Get a mutable reference to the Session if this is a Session variant
    pub fn as_session_mut(&mut self) -> Option<&mut Session> {
        if let ContentItem::Session(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Get a mutable reference to the List if this is a List variant
    pub fn as_list_mut(&mut self) -> Option<&mut List> {
        if let ContentItem::List(l) = self {
            Some(l)
        } else {
            None
        }
    }

    /// Get a mutable reference to the Definition if this is a Definition variant
    pub fn as_definition_mut(&mut self) -> Option<&mut Definition> {
        if let ContentItem::Definition(d) = self {
            Some(d)
        } else {
            None
        }
    }

    /// Get a reference to the Annotation if this is an Annotation variant
    pub fn as_annotation(&self) -> Option<&Annotation> {
        if let ContentItem::Annotation(a) = self {
            Some(a)
        } else {
            None
        }
    }

    /// Get a mutable reference to the Annotation if this is an Annotation variant
    pub fn as_annotation_mut(&mut self) -> Option<&mut Annotation> {
        if let ContentItem::Annotation(a) = self {
            Some(a)
        } else {
            None
        }
    }

    /// Get a reference to the ForeignBlock if this is a ForeignBlock variant
    pub fn as_foreign_block(&self) -> Option<&ForeignBlock> {
        if let ContentItem::ForeignBlock(fb) = self {
            Some(fb)
        } else {
            None
        }
    }

    /// Get a mutable reference to the ForeignBlock if this is a ForeignBlock variant
    pub fn as_foreign_block_mut(&mut self) -> Option<&mut ForeignBlock> {
        if let ContentItem::ForeignBlock(fb) = self {
            Some(fb)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paragraph_creation() {
        let para = Paragraph::new(vec!["Hello".to_string(), "World".to_string()]);
        assert_eq!(para.lines.len(), 2);
        assert_eq!(para.text(), "Hello\nWorld");
    }

    #[test]
    fn test_session_creation() {
        let session = Session::new(
            "Introduction".to_string(),
            vec![ContentItem::Paragraph(Paragraph::from_line(
                "Content".to_string(),
            ))],
        );
        assert_eq!(session.title, "Introduction");
        assert_eq!(session.content.len(), 1);
    }

    #[test]
    fn test_document_creation() {
        let doc = Document::with_items(vec![
            ContentItem::Paragraph(Paragraph::from_line("Para 1".to_string())),
            ContentItem::Session(Session::with_title("Section 1".to_string())),
        ]);
        assert_eq!(doc.items.len(), 2);
    }

    // ========================================================================
    // Trait Tests
    // ========================================================================

    #[test]
    fn test_paragraph_ast_node_trait() {
        let para = Paragraph::new(vec!["First line".to_string(), "Second line".to_string()]);

        assert_eq!(para.node_type(), "Paragraph");
        assert_eq!(para.display_label(), "First line\nSecond line");
    }

    #[test]
    fn test_paragraph_ast_node_trait_long_text() {
        let long_text = "a".repeat(60);
        let para = Paragraph::from_line(long_text);

        assert_eq!(para.node_type(), "Paragraph");
        let label = para.display_label();
        assert!(label.ends_with("..."));
        assert_eq!(label.len(), 53); // 50 chars + "..."
    }

    #[test]
    fn test_paragraph_text_node_trait() {
        let para = Paragraph::new(vec![
            "Line 1".to_string(),
            "Line 2".to_string(),
            "Line 3".to_string(),
        ]);

        assert_eq!(para.text(), "Line 1\nLine 2\nLine 3");
        assert_eq!(para.lines(), &["Line 1", "Line 2", "Line 3"]);
    }

    #[test]
    fn test_session_ast_node_trait() {
        let session = Session::new(
            "Introduction".to_string(),
            vec![ContentItem::Paragraph(Paragraph::from_line(
                "Content".to_string(),
            ))],
        );

        assert_eq!(session.node_type(), "Session");
        assert_eq!(session.display_label(), "Introduction");
    }

    #[test]
    fn test_session_container_trait() {
        let mut session = Session::new(
            "My Session".to_string(),
            vec![
                ContentItem::Paragraph(Paragraph::from_line("Para 1".to_string())),
                ContentItem::Paragraph(Paragraph::from_line("Para 2".to_string())),
            ],
        );

        // Test label
        assert_eq!(session.label(), "My Session");

        // Test children (immutable)
        assert_eq!(session.children().len(), 2);
        match &session.children()[0] {
            ContentItem::Paragraph(p) => assert_eq!(p.text(), "Para 1"),
            _ => panic!("Expected paragraph"),
        }

        // Test children_mut
        session
            .children_mut()
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Para 3".to_string(),
            )));
        assert_eq!(session.children().len(), 3);
    }

    #[test]
    fn test_content_item_node_type() {
        let para = ContentItem::Paragraph(Paragraph::from_line("Text".to_string()));
        assert_eq!(para.node_type(), "Paragraph");

        let session = ContentItem::Session(Session::with_title("Title".to_string()));
        assert_eq!(session.node_type(), "Session");
    }

    #[test]
    fn test_content_item_display_label() {
        let para = ContentItem::Paragraph(Paragraph::from_line("Hello world".to_string()));
        assert_eq!(para.display_label(), "Hello world");

        let session = ContentItem::Session(Session::with_title("My Title".to_string()));
        assert_eq!(session.display_label(), "My Title");
    }

    #[test]
    fn test_content_item_label() {
        let para = ContentItem::Paragraph(Paragraph::from_line("Text".to_string()));
        assert_eq!(para.label(), None);

        let session = ContentItem::Session(Session::with_title("Title".to_string()));
        assert_eq!(session.label(), Some("Title"));
    }

    #[test]
    fn test_content_item_children() {
        let para = ContentItem::Paragraph(Paragraph::from_line("Text".to_string()));
        assert_eq!(para.children(), None);

        let session = ContentItem::Session(Session::new(
            "Title".to_string(),
            vec![ContentItem::Paragraph(Paragraph::from_line(
                "Content".to_string(),
            ))],
        ));
        assert_eq!(session.children().unwrap().len(), 1);
    }

    #[test]
    fn test_content_item_children_mut() {
        let mut para = ContentItem::Paragraph(Paragraph::from_line("Text".to_string()));
        assert_eq!(para.children_mut(), None);

        let mut session = ContentItem::Session(Session::with_title("Title".to_string()));
        let children = session.children_mut().unwrap();
        children.push(ContentItem::Paragraph(Paragraph::from_line(
            "New content".to_string(),
        )));

        assert_eq!(session.children().unwrap().len(), 1);
    }

    #[test]
    fn test_content_item_text() {
        let para = ContentItem::Paragraph(Paragraph::new(vec![
            "Line 1".to_string(),
            "Line 2".to_string(),
        ]));
        assert_eq!(para.text(), Some("Line 1\nLine 2".to_string()));

        let session = ContentItem::Session(Session::with_title("Title".to_string()));
        assert_eq!(session.text(), None);
    }

    #[test]
    fn test_nested_session_trait_access() {
        // Create a nested structure
        let mut root_session = Session::new(
            "Root".to_string(),
            vec![
                ContentItem::Paragraph(Paragraph::from_line("Para 1".to_string())),
                ContentItem::Session(Session::new(
                    "Nested".to_string(),
                    vec![ContentItem::Paragraph(Paragraph::from_line(
                        "Nested para".to_string(),
                    ))],
                )),
            ],
        );

        // Access through traits
        assert_eq!(root_session.label(), "Root");
        assert_eq!(root_session.children().len(), 2);

        // Navigate to nested session using trait methods
        if let Some(ContentItem::Session(nested)) = root_session.children().get(1) {
            assert_eq!(nested.label(), "Nested");
            assert_eq!(nested.children().len(), 1);
        } else {
            panic!("Expected nested session");
        }

        // Mutate through traits
        root_session
            .children_mut()
            .push(ContentItem::Paragraph(Paragraph::from_line(
                "Added para".to_string(),
            )));
        assert_eq!(root_session.children().len(), 3);
    }
}
