//! AST node type definitions and trait implementations
//!
//! This module defines all the node types that represent the parsed
//! structure of a txxt document, along with the traits for uniform
//! access to node information.

use super::span::{Position, Span};
use std::fmt;

// ============================================================================
// AST Traits - Common interfaces for uniform node access
// ============================================================================

/// Common interface for all AST nodes
pub trait AstNode {
    fn node_type(&self) -> &'static str;
    fn display_label(&self) -> String;
}

/// Trait for container nodes that have a label and children
pub trait Container: AstNode {
    fn label(&self) -> &str;
    fn children(&self) -> &[ContentItem];
    fn children_mut(&mut self) -> &mut Vec<ContentItem>;
}

/// Trait for leaf nodes that contain text
pub trait TextNode: AstNode {
    fn text(&self) -> String;
    fn lines(&self) -> &[String];
}

// ============================================================================
// AST Node Definitions
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub metadata: Vec<Annotation>,
    pub content: Vec<ContentItem>,
    pub span: Option<Span>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContentItem {
    Paragraph(Paragraph),
    Session(Session),
    List(List),
    Definition(Definition),
    Annotation(Annotation),
    ForeignBlock(ForeignBlock),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Paragraph {
    pub lines: Vec<String>,
    pub span: Option<Span>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    pub title: String,
    pub content: Vec<ContentItem>,
    pub span: Option<Span>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct List {
    pub items: Vec<ListItem>,
    pub span: Option<Span>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    text: Vec<String>,
    pub content: Vec<ContentItem>,
    pub span: Option<Span>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Definition {
    pub subject: String,
    pub content: Vec<ContentItem>,
    pub span: Option<Span>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForeignBlock {
    pub subject: String,
    pub content: String,
    pub closing_annotation: Annotation,
    pub span: Option<Span>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Label {
    pub value: String,
    pub span: Option<Span>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub key: String,
    pub value: Option<String>,
    pub span: Option<Span>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Annotation {
    pub label: Label,
    pub parameters: Vec<Parameter>,
    pub content: Vec<ContentItem>,
    pub span: Option<Span>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            metadata: Vec::new(),
            content: Vec::new(),
            span: None,
        }
    }

    pub fn with_content(content: Vec<ContentItem>) -> Self {
        Self {
            metadata: Vec::new(),
            content,
            span: None,
        }
    }

    pub fn with_metadata_and_content(metadata: Vec<Annotation>, content: Vec<ContentItem>) -> Self {
        Self {
            metadata,
            content,
            span: None,
        }
    }

    pub fn with_span(mut self, span: Option<Span>) -> Self {
        self.span = span;
        self
    }

    pub fn iter_items(&self) -> impl Iterator<Item = &ContentItem> {
        self.content.iter()
    }

    pub fn iter_paragraphs(&self) -> impl Iterator<Item = &Paragraph> {
        self.content.iter().filter_map(|item| item.as_paragraph())
    }

    pub fn iter_sessions(&self) -> impl Iterator<Item = &Session> {
        self.content.iter().filter_map(|item| item.as_session())
    }

    pub fn iter_lists(&self) -> impl Iterator<Item = &List> {
        self.content.iter().filter_map(|item| item.as_list())
    }

    pub fn iter_foreign_blocks(&self) -> impl Iterator<Item = &ForeignBlock> {
        self.content
            .iter()
            .filter_map(|item| item.as_foreign_block())
    }

    pub fn count_by_type(&self) -> (usize, usize, usize, usize) {
        let paragraphs = self.iter_paragraphs().count();
        let sessions = self.iter_sessions().count();
        let lists = self.iter_lists().count();
        let foreign_blocks = self.iter_foreign_blocks().count();
        (paragraphs, sessions, lists, foreign_blocks)
    }

    /// Find all elements at the given position, returning them in order from deepest to shallowest
    pub fn elements_at(&self, pos: Position) -> Vec<&ContentItem> {
        let mut results = Vec::new();
        for item in &self.content {
            if let Some(mut items) = item.elements_at(pos) {
                results.append(&mut items);
            }
        }
        results
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Paragraph {
    pub fn new(lines: Vec<String>) -> Self {
        Self { lines, span: None }
    }
    pub fn from_line(line: String) -> Self {
        Self {
            lines: vec![line],
            span: None,
        }
    }
    pub fn with_span(mut self, span: Option<Span>) -> Self {
        self.span = span;
        self
    }
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }
}

impl Session {
    pub fn new(title: String, content: Vec<ContentItem>) -> Self {
        Self {
            title,
            content,
            span: None,
        }
    }
    pub fn with_title(title: String) -> Self {
        Self {
            title,
            content: Vec::new(),
            span: None,
        }
    }
    pub fn with_span(mut self, span: Option<Span>) -> Self {
        self.span = span;
        self
    }
}

impl List {
    pub fn new(items: Vec<ListItem>) -> Self {
        Self { items, span: None }
    }
    pub fn with_span(mut self, span: Option<Span>) -> Self {
        self.span = span;
        self
    }
}

impl ListItem {
    pub fn new(text: String) -> Self {
        Self {
            text: vec![text],
            content: Vec::new(),
            span: None,
        }
    }
    pub fn with_content(text: String, content: Vec<ContentItem>) -> Self {
        Self {
            text: vec![text],
            content,
            span: None,
        }
    }
    pub fn with_span(mut self, span: Option<Span>) -> Self {
        self.span = span;
        self
    }
    pub fn text(&self) -> &str {
        &self.text[0]
    }
}

impl Definition {
    pub fn new(subject: String, content: Vec<ContentItem>) -> Self {
        Self {
            subject,
            content,
            span: None,
        }
    }
    pub fn with_subject(subject: String) -> Self {
        Self {
            subject,
            content: Vec::new(),
            span: None,
        }
    }
    pub fn with_span(mut self, span: Option<Span>) -> Self {
        self.span = span;
        self
    }
}

impl ForeignBlock {
    pub fn new(subject: String, content: String, closing_annotation: Annotation) -> Self {
        Self {
            subject,
            content,
            closing_annotation,
            span: None,
        }
    }
    pub fn marker(subject: String, closing_annotation: Annotation) -> Self {
        Self {
            subject,
            content: String::new(),
            closing_annotation,
            span: None,
        }
    }
    pub fn with_span(mut self, span: Option<Span>) -> Self {
        self.span = span;
        self
    }
}

impl Label {
    pub fn new(value: String) -> Self {
        Self { value, span: None }
    }
    pub fn from_string(value: &str) -> Self {
        Self {
            value: value.to_string(),
            span: None,
        }
    }
    pub fn with_span(mut self, span: Option<Span>) -> Self {
        self.span = span;
        self
    }
}

impl Parameter {
    pub fn new(key: String, value: Option<String>) -> Self {
        Self {
            key,
            value,
            span: None,
        }
    }
    pub fn boolean(key: String) -> Self {
        Self {
            key,
            value: None,
            span: None,
        }
    }
    pub fn with_value(key: String, value: String) -> Self {
        Self {
            key,
            value: Some(value),
            span: None,
        }
    }
    pub fn with_span(mut self, span: Option<Span>) -> Self {
        self.span = span;
        self
    }
}

impl Annotation {
    pub fn new(label: Label, parameters: Vec<Parameter>, content: Vec<ContentItem>) -> Self {
        Self {
            label,
            parameters,
            content,
            span: None,
        }
    }
    pub fn marker(label: Label) -> Self {
        Self {
            label,
            parameters: Vec::new(),
            content: Vec::new(),
            span: None,
        }
    }
    pub fn with_parameters(label: Label, parameters: Vec<Parameter>) -> Self {
        Self {
            label,
            parameters,
            content: Vec::new(),
            span: None,
        }
    }
    pub fn with_span(mut self, span: Option<Span>) -> Self {
        self.span = span;
        self
    }
}

impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Document({} metadata, {} items)",
            self.metadata.len(),
            self.content.len()
        )
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
            ContentItem::Annotation(a) => write!(
                f,
                "Annotation('{}', {} params, {} items)",
                a.label.value,
                a.parameters.len(),
                a.content.len()
            ),
            ContentItem::ForeignBlock(fb) => write!(f, "ForeignBlock('{}')", fb.subject),
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

impl AstNode for List {
    fn node_type(&self) -> &'static str {
        "List"
    }
    fn display_label(&self) -> String {
        format!("{} items", self.items.len())
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

impl AstNode for Annotation {
    fn node_type(&self) -> &'static str {
        "Annotation"
    }
    fn display_label(&self) -> String {
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
            ContentItem::ForeignBlock(fb) => Some(&fb.subject),
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
        let doc = Document::with_content(vec![
            ContentItem::Paragraph(Paragraph::from_line("Para 1".to_string())),
            ContentItem::Session(Session::with_title("Section 1".to_string())),
        ]);
        assert_eq!(doc.content.len(), 2);
        assert_eq!(doc.metadata.len(), 0);
    }

    // ========================================================================
    // Position Tracking Tests
    // ========================================================================

    #[test]
    fn test_position_creation() {
        let pos = Position::new(5, 10);
        assert_eq!(pos.line, 5);
        assert_eq!(pos.column, 10);
    }

    #[test]
    fn test_position_comparison() {
        let pos1 = Position::new(1, 5);
        let pos2 = Position::new(1, 5);
        let pos3 = Position::new(2, 3);

        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);
        assert!(pos1 < pos3);
    }

    #[test]
    fn test_span_creation() {
        let start = Position::new(0, 0);
        let end = Position::new(2, 5);
        let span = Span::new(start, end);

        assert_eq!(span.start, start);
        assert_eq!(span.end, end);
    }

    #[test]
    fn test_span_contains_single_line() {
        let span = Span::new(Position::new(0, 0), Position::new(0, 10));

        assert!(span.contains(Position::new(0, 0)));
        assert!(span.contains(Position::new(0, 5)));
        assert!(span.contains(Position::new(0, 10)));

        assert!(!span.contains(Position::new(0, 11)));
        assert!(!span.contains(Position::new(1, 0)));
    }

    #[test]
    fn test_span_contains_multiline() {
        let span = Span::new(Position::new(1, 5), Position::new(2, 10));

        // Before span
        assert!(!span.contains(Position::new(1, 4)));
        assert!(!span.contains(Position::new(0, 5)));

        // In span
        assert!(span.contains(Position::new(1, 5)));
        assert!(span.contains(Position::new(1, 10)));
        assert!(span.contains(Position::new(2, 0)));
        assert!(span.contains(Position::new(2, 10)));

        // After span
        assert!(!span.contains(Position::new(2, 11)));
        assert!(!span.contains(Position::new(3, 0)));
    }

    #[test]
    fn test_span_overlaps() {
        let span1 = Span::new(Position::new(0, 0), Position::new(1, 5));
        let span2 = Span::new(Position::new(1, 0), Position::new(2, 5));
        let span3 = Span::new(Position::new(3, 0), Position::new(4, 5));

        assert!(span1.overlaps(span2));
        assert!(span2.overlaps(span1));
        assert!(!span1.overlaps(span3));
        assert!(!span3.overlaps(span1));
    }

    #[test]
    fn test_position_display() {
        let pos = Position::new(5, 10);
        assert_eq!(format!("{}", pos), "5:10");
    }

    #[test]
    fn test_span_display() {
        let span = Span::new(Position::new(1, 0), Position::new(2, 5));
        assert_eq!(format!("{}", span), "1:0..2:5");
    }

    // ========================================================================
    // Query API Tests
    // ========================================================================

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
        let session = Session::new("Section".to_string(), vec![ContentItem::Paragraph(para)])
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

    #[test]
    fn test_document_elements_at() {
        let para1 = Paragraph::from_line("First".to_string())
            .with_span(Some(Span::new(Position::new(0, 0), Position::new(0, 5))));
        let para2 = Paragraph::from_line("Second".to_string())
            .with_span(Some(Span::new(Position::new(1, 0), Position::new(1, 6))));

        let doc = Document::with_content(vec![
            ContentItem::Paragraph(para1),
            ContentItem::Paragraph(para2),
        ]);

        let results = doc.elements_at(Position::new(1, 3));
        assert_eq!(results.len(), 1);
        assert!(results[0].is_paragraph());
    }

    #[test]
    fn test_paragraph_with_span() {
        let span = Span::new(Position::new(0, 0), Position::new(0, 5));
        let para = Paragraph::from_line("Hello".to_string()).with_span(Some(span));

        assert_eq!(para.span, Some(span));
    }

    #[test]
    fn test_builder_methods() {
        let span = Span::new(Position::new(1, 0), Position::new(1, 10));

        let para = Paragraph::new(vec!["Test".to_string()]).with_span(Some(span));
        assert_eq!(para.span, Some(span));

        let session = Session::with_title("Title".to_string()).with_span(Some(span));
        assert_eq!(session.span, Some(span));

        let list = List::new(vec![]).with_span(Some(span));
        assert_eq!(list.span, Some(span));

        let definition = Definition::with_subject("Subject".to_string()).with_span(Some(span));
        assert_eq!(definition.span, Some(span));

        let label = Label::new("test".to_string()).with_span(Some(span));
        assert_eq!(label.span, Some(span));

        let param =
            Parameter::new("key".to_string(), Some("value".to_string())).with_span(Some(span));
        assert_eq!(param.span, Some(span));

        let annotation = Annotation::marker(Label::new("test".to_string())).with_span(Some(span));
        assert_eq!(annotation.span, Some(span));
    }
}
