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
    pub items: Vec<ContentItem>,
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

// ============================================================================
// Container Types - Explicit container nodes that hold children
// ============================================================================

/// Session container - can host any element including nested sessions
#[derive(Debug, Clone, PartialEq)]
pub struct SessionContainer {
    items: Vec<ContentItem>,
}

/// Content container - can host any element except sessions
#[derive(Debug, Clone, PartialEq)]
pub struct ContentContainer {
    items: Vec<ContentItem>,
}

/// Annotation container - can host paragraphs, lists, definitions, and foreign blocks
/// Cannot host sessions or nested annotations
#[derive(Debug, Clone, PartialEq)]
pub struct AnnotationContainer {
    items: Vec<ContentItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Paragraph {
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    pub title: String,
    pub container: SessionContainer,
}

#[derive(Debug, Clone, PartialEq)]
pub struct List {
    pub items: Vec<ListItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    text: Vec<String>,
    pub container: ContentContainer,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Definition {
    pub subject: String,
    pub container: ContentContainer,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForeignBlock {
    pub subject: String,
    pub content: String,
    pub closing_annotation: Annotation,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Label {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub key: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Annotation {
    pub label: Label,
    pub parameters: Vec<Parameter>,
    pub container: AnnotationContainer,
}

impl Document {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn with_items(items: Vec<ContentItem>) -> Self {
        Self { items }
    }

    pub fn iter_items(&self) -> impl Iterator<Item = &ContentItem> {
        self.items.iter()
    }

    pub fn iter_paragraphs(&self) -> impl Iterator<Item = &Paragraph> {
        self.items.iter().filter_map(|item| item.as_paragraph())
    }

    pub fn iter_sessions(&self) -> impl Iterator<Item = &Session> {
        self.items.iter().filter_map(|item| item.as_session())
    }

    pub fn iter_lists(&self) -> impl Iterator<Item = &List> {
        self.items.iter().filter_map(|item| item.as_list())
    }

    pub fn iter_foreign_blocks(&self) -> impl Iterator<Item = &ForeignBlock> {
        self.items.iter().filter_map(|item| item.as_foreign_block())
    }

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
    pub fn new(lines: Vec<String>) -> Self {
        Self { lines }
    }
    pub fn from_line(line: String) -> Self {
        Self { lines: vec![line] }
    }
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }
}

impl Session {
    pub fn new(title: String, container: SessionContainer) -> Self {
        Self { title, container }
    }
    pub fn with_title(title: String) -> Self {
        Self {
            title,
            container: SessionContainer::new(),
        }
    }
    pub fn with_items(title: String, items: Vec<ContentItem>) -> Self {
        Self {
            title,
            container: SessionContainer::with_items(items),
        }
    }
}

impl List {
    pub fn new(items: Vec<ListItem>) -> Self {
        Self { items }
    }
}

impl ListItem {
    pub fn new(text: String) -> Self {
        Self {
            text: vec![text],
            container: ContentContainer::new(),
        }
    }
    pub fn with_container(text: String, container: ContentContainer) -> Self {
        Self {
            text: vec![text],
            container,
        }
    }
    pub fn with_items(text: String, items: Vec<ContentItem>) -> Self {
        Self {
            text: vec![text],
            container: ContentContainer::with_items(items),
        }
    }
    pub fn text(&self) -> &str {
        &self.text[0]
    }
}

impl Definition {
    pub fn new(subject: String, container: ContentContainer) -> Self {
        Self { subject, container }
    }
    pub fn with_subject(subject: String) -> Self {
        Self {
            subject,
            container: ContentContainer::new(),
        }
    }
    pub fn with_items(subject: String, items: Vec<ContentItem>) -> Self {
        Self {
            subject,
            container: ContentContainer::with_items(items),
        }
    }
}

impl ForeignBlock {
    pub fn new(subject: String, content: String, closing_annotation: Annotation) -> Self {
        Self {
            subject,
            content,
            closing_annotation,
        }
    }
    pub fn marker(subject: String, closing_annotation: Annotation) -> Self {
        Self {
            subject,
            content: String::new(),
            closing_annotation,
        }
    }
}

impl Label {
    pub fn new(value: String) -> Self {
        Self { value }
    }
    pub fn from_string(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }
}

impl Parameter {
    pub fn new(key: String, value: Option<String>) -> Self {
        Self { key, value }
    }
    pub fn boolean(key: String) -> Self {
        Self { key, value: None }
    }
    pub fn with_value(key: String, value: String) -> Self {
        Self {
            key,
            value: Some(value),
        }
    }
}

impl Annotation {
    pub fn new(label: Label, parameters: Vec<Parameter>, container: AnnotationContainer) -> Self {
        Self {
            label,
            parameters,
            container,
        }
    }
    pub fn marker(label: Label) -> Self {
        Self {
            label,
            parameters: Vec::new(),
            container: AnnotationContainer::new(),
        }
    }
    pub fn with_parameters(label: Label, parameters: Vec<Parameter>) -> Self {
        Self {
            label,
            parameters,
            container: AnnotationContainer::new(),
        }
    }
    pub fn with_items(label: Label, parameters: Vec<Parameter>, items: Vec<ContentItem>) -> Self {
        Self {
            label,
            parameters,
            container: AnnotationContainer::with_items(items),
        }
    }
}

// ============================================================================
// Container Implementations - Full Vec wrapper methods
// ============================================================================

impl SessionContainer {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn with_items(items: Vec<ContentItem>) -> Self {
        Self { items }
    }

    pub fn push(&mut self, item: ContentItem) {
        self.items.push(item)
    }

    pub fn pop(&mut self) -> Option<ContentItem> {
        self.items.pop()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn clear(&mut self) {
        self.items.clear()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ContentItem> {
        self.items.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut ContentItem> {
        self.items.iter_mut()
    }

    pub fn as_slice(&self) -> &[ContentItem] {
        &self.items
    }

    pub fn as_mut_vec(&mut self) -> &mut Vec<ContentItem> {
        &mut self.items
    }
}

impl ContentContainer {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn with_items(items: Vec<ContentItem>) -> Self {
        Self { items }
    }

    pub fn push(&mut self, item: ContentItem) {
        self.items.push(item)
    }

    pub fn pop(&mut self) -> Option<ContentItem> {
        self.items.pop()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn clear(&mut self) {
        self.items.clear()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ContentItem> {
        self.items.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut ContentItem> {
        self.items.iter_mut()
    }

    pub fn as_slice(&self) -> &[ContentItem] {
        &self.items
    }

    pub fn as_mut_vec(&mut self) -> &mut Vec<ContentItem> {
        &mut self.items
    }
}

impl AnnotationContainer {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn with_items(items: Vec<ContentItem>) -> Self {
        Self { items }
    }

    pub fn push(&mut self, item: ContentItem) {
        self.items.push(item)
    }

    pub fn pop(&mut self) -> Option<ContentItem> {
        self.items.pop()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn clear(&mut self) {
        self.items.clear()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ContentItem> {
        self.items.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut ContentItem> {
        self.items.iter_mut()
    }

    pub fn as_slice(&self) -> &[ContentItem] {
        &self.items
    }

    pub fn as_mut_vec(&mut self) -> &mut Vec<ContentItem> {
        &mut self.items
    }
}

// Index trait implementations for container[index] syntax
impl std::ops::Index<usize> for SessionContainer {
    type Output = ContentItem;
    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

impl std::ops::IndexMut<usize> for SessionContainer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.items[index]
    }
}

impl std::ops::Index<usize> for ContentContainer {
    type Output = ContentItem;
    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

impl std::ops::IndexMut<usize> for ContentContainer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.items[index]
    }
}

impl std::ops::Index<usize> for AnnotationContainer {
    type Output = ContentItem;
    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

impl std::ops::IndexMut<usize> for AnnotationContainer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.items[index]
    }
}

// IntoIterator implementations for 'for item in container' syntax
impl IntoIterator for SessionContainer {
    type Item = ContentItem;
    type IntoIter = std::vec::IntoIter<ContentItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a> IntoIterator for &'a SessionContainer {
    type Item = &'a ContentItem;
    type IntoIter = std::slice::Iter<'a, ContentItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl IntoIterator for ContentContainer {
    type Item = ContentItem;
    type IntoIter = std::vec::IntoIter<ContentItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a> IntoIterator for &'a ContentContainer {
    type Item = &'a ContentItem;
    type IntoIter = std::slice::Iter<'a, ContentItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl IntoIterator for AnnotationContainer {
    type Item = ContentItem;
    type IntoIter = std::vec::IntoIter<ContentItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a> IntoIterator for &'a AnnotationContainer {
    type Item = &'a ContentItem;
    type IntoIter = std::slice::Iter<'a, ContentItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl Default for SessionContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ContentContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AnnotationContainer {
    fn default() -> Self {
        Self::new()
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
                write!(f, "Session('{}', {} items)", s.title, s.container.len())
            }
            ContentItem::List(l) => write!(f, "List({} items)", l.items.len()),
            ContentItem::Definition(d) => {
                write!(
                    f,
                    "Definition('{}', {} items)",
                    d.subject,
                    d.container.len()
                )
            }
            ContentItem::Annotation(a) => write!(
                f,
                "Annotation('{}', {} params, {} items)",
                a.label.value,
                a.parameters.len(),
                a.container.len()
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
        write!(
            f,
            "Session('{}', {} items)",
            self.title,
            self.container.len()
        )
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
            self.container.len()
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
            self.container.len()
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
        self.container.as_slice()
    }
    fn children_mut(&mut self) -> &mut Vec<ContentItem> {
        self.container.as_mut_vec()
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
        self.container.as_slice()
    }
    fn children_mut(&mut self) -> &mut Vec<ContentItem> {
        self.container.as_mut_vec()
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
        self.container.as_slice()
    }
    fn children_mut(&mut self) -> &mut Vec<ContentItem> {
        self.container.as_mut_vec()
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
        self.container.as_slice()
    }
    fn children_mut(&mut self) -> &mut Vec<ContentItem> {
        self.container.as_mut_vec()
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
        let session = Session::with_items(
            "Introduction".to_string(),
            vec![ContentItem::Paragraph(Paragraph::from_line(
                "Content".to_string(),
            ))],
        );
        assert_eq!(session.title, "Introduction");
        assert_eq!(session.container.len(), 1);
    }

    #[test]
    fn test_document_creation() {
        let doc = Document::with_items(vec![
            ContentItem::Paragraph(Paragraph::from_line("Para 1".to_string())),
            ContentItem::Session(Session::with_title("Section 1".to_string())),
        ]);
        assert_eq!(doc.items.len(), 2);
    }
}
