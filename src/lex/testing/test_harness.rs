//! Test harness for per-element testing
//!
//! This module provides utilities for testing individual element variations
//! using the per-element library in `docs/specs/v1/elements/`.
//!
//! # Usage
//!
//! ```rust,ignore
//! use lex::lex::testing::test_harness::*;
//!
//! // Load a specific element variation
//! let source = ElementSources::get_source_for(ElementType::Paragraph, 1).unwrap();
//! let doc = parse_with_parser(&source, Parser::Reference).unwrap();
//!
//! // Get first element of a type
//! let paragraph = get_first_element::<Paragraph>(&doc).unwrap();
//!
//! // Use assertions
//! assert_eq!(paragraph.lines().len(), 1);
//! assert!(paragraph_text_starts_with(&paragraph, "This is a simple"));
//! ```

use crate::lex::ast::traits::{Container, TextNode};
use crate::lex::ast::{Annotation, ContentItem, Definition, Document, List, Paragraph, Session};
use crate::lex::parsing::ParseError;
use crate::lex::pipeline::{ExecutionOutput, PipelineExecutor};
use std::fs;
use std::path::PathBuf;

/// Element types that can be loaded from the per-element library
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementType {
    Paragraph,
    List,
    Session,
    Definition,
    Annotation,
    Verbatim,
}

impl ElementType {
    /// Get the directory name for this element type
    fn dir_name(&self) -> &'static str {
        match self {
            ElementType::Paragraph => "paragraph",
            ElementType::List => "list",
            ElementType::Session => "session",
            ElementType::Definition => "definition",
            ElementType::Annotation => "annotation",
            ElementType::Verbatim => "verbatim",
        }
    }

    /// Get the element name prefix for filenames
    fn prefix(&self) -> &'static str {
        match self {
            ElementType::Paragraph => "paragraph",
            ElementType::List => "list",
            ElementType::Session => "session",
            ElementType::Definition => "definition",
            ElementType::Annotation => "annotation",
            ElementType::Verbatim => "verbatim",
        }
    }
}

/// Parser implementation to use for parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parser {
    /// Reference parser (combinator-based, stable)
    Reference,
    /// Linebased parser (grammar-based, experimental)
    Linebased,
}

impl Parser {
    /// Get the pipeline config name for this parser
    fn config_name(&self) -> &'static str {
        match self {
            Parser::Reference => "default",
            Parser::Linebased => "linebased",
        }
    }
}

/// Errors that can occur when loading element sources
#[derive(Debug, Clone)]
pub enum ElementSourceError {
    FileNotFound(String),
    IoError(String),
    ParseError(String),
    InvalidElement(String),
}

impl std::fmt::Display for ElementSourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElementSourceError::FileNotFound(msg) => write!(f, "File not found: {}", msg),
            ElementSourceError::IoError(msg) => write!(f, "IO error: {}", msg),
            ElementSourceError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ElementSourceError::InvalidElement(msg) => write!(f, "Invalid element: {}", msg),
        }
    }
}

impl std::error::Error for ElementSourceError {}

impl From<std::io::Error> for ElementSourceError {
    fn from(err: std::io::Error) -> Self {
        ElementSourceError::IoError(err.to_string())
    }
}

impl From<ParseError> for ElementSourceError {
    fn from(err: ParseError) -> Self {
        ElementSourceError::ParseError(err.to_string())
    }
}

/// Fluent API builder for loading elements
pub struct ElementLoader {
    element_type: ElementType,
    number: usize,
}

impl ElementLoader {
    /// Get the raw source string
    pub fn source(&self) -> String {
        ElementSources::must_get_source_for(self.element_type, self.number)
    }

    /// Parse with the specified parser and return a ParsedElement for further chaining
    pub fn parse_with(self, parser: Parser) -> ParsedElement {
        let doc = ElementSources::must_get_ast_for(self.element_type, self.number, parser);
        ParsedElement {
            element_type: self.element_type,
            doc,
        }
    }

    /// Parse with the Reference parser (shorthand)
    pub fn parse(self) -> ParsedElement {
        self.parse_with(Parser::Reference)
    }
}

/// A parsed element document, ready for element extraction
pub struct ParsedElement {
    element_type: ElementType,
    doc: Document,
}

impl ParsedElement {
    /// Get the underlying document
    pub fn document(&self) -> &Document {
        &self.doc
    }

    /// Get the first paragraph, panicking if not found
    pub fn expect_paragraph(&self) -> &Paragraph {
        get_first_paragraph(&self.doc)
            .unwrap_or_else(|| panic!("No paragraph found in {:?} document", self.element_type))
    }

    /// Get the first session, panicking if not found
    pub fn expect_session(&self) -> &Session {
        get_first_session(&self.doc)
            .unwrap_or_else(|| panic!("No session found in {:?} document", self.element_type))
    }

    /// Get the first list, panicking if not found
    pub fn expect_list(&self) -> &List {
        get_first_list(&self.doc)
            .unwrap_or_else(|| panic!("No list found in {:?} document", self.element_type))
    }

    /// Get the first definition, panicking if not found
    pub fn expect_definition(&self) -> &Definition {
        get_first_definition(&self.doc)
            .unwrap_or_else(|| panic!("No definition found in {:?} document", self.element_type))
    }

    /// Get the first annotation, panicking if not found
    pub fn expect_annotation(&self) -> &Annotation {
        get_first_annotation(&self.doc)
            .unwrap_or_else(|| panic!("No annotation found in {:?} document", self.element_type))
    }

    /// Get the first verbatim block, panicking if not found
    pub fn expect_verbatim(&self) -> &crate::lex::ast::Verbatim {
        get_first_verbatim(&self.doc)
            .unwrap_or_else(|| panic!("No verbatim found in {:?} document", self.element_type))
    }

    /// Get the first paragraph (returns Option)
    pub fn first_paragraph(&self) -> Option<&Paragraph> {
        get_first_paragraph(&self.doc)
    }

    /// Get the first session (returns Option)
    pub fn first_session(&self) -> Option<&Session> {
        get_first_session(&self.doc)
    }

    /// Get the first list (returns Option)
    pub fn first_list(&self) -> Option<&List> {
        get_first_list(&self.doc)
    }

    /// Get the first definition (returns Option)
    pub fn first_definition(&self) -> Option<&Definition> {
        get_first_definition(&self.doc)
    }

    /// Get the first annotation (returns Option)
    pub fn first_annotation(&self) -> Option<&Annotation> {
        get_first_annotation(&self.doc)
    }

    /// Get the first verbatim block (returns Option)
    pub fn first_verbatim(&self) -> Option<&crate::lex::ast::Verbatim> {
        get_first_verbatim(&self.doc)
    }
}

/// Interface for loading per-element test sources
pub struct ElementSources;

impl ElementSources {
    const SPEC_VERSION: &'static str = "v1";

    /// Get the path to the elements directory
    fn elements_dir() -> PathBuf {
        PathBuf::from(format!("docs/specs/{}/elements", Self::SPEC_VERSION))
    }

    // ===== Convenience "must_" methods that panic on error =====

    /// Get source string, panicking with helpful message if not found
    pub fn must_get_source_for(element_type: ElementType, number: usize) -> String {
        Self::get_source_for(element_type, number)
            .unwrap_or_else(|e| panic!("Failed to load {:?} #{}: {}", element_type, number, e))
    }

    /// Get AST document, panicking if not found or parse fails
    pub fn must_get_ast_for(element_type: ElementType, number: usize, parser: Parser) -> Document {
        Self::get_ast_for(element_type, number, parser).unwrap_or_else(|e| {
            panic!("Failed to load/parse {:?} #{}: {}", element_type, number, e)
        })
    }

    // ===== Fluent API - start a chain =====

    /// Start a fluent chain for loading and parsing an element
    ///
    /// # Example
    /// ```rust,ignore
    /// let doc = ElementSources::load(ElementType::Paragraph, 1)
    ///     .parse_with(Parser::Reference);
    /// ```
    pub fn load(element_type: ElementType, number: usize) -> ElementLoader {
        ElementLoader {
            element_type,
            number,
        }
    }

    // ===== Convenience shortcuts for specific element types =====

    /// Load a paragraph variation (fluent API)
    pub fn paragraph(number: usize) -> ElementLoader {
        Self::load(ElementType::Paragraph, number)
    }

    /// Load a list variation (fluent API)
    pub fn list(number: usize) -> ElementLoader {
        Self::load(ElementType::List, number)
    }

    /// Load a session variation (fluent API)
    pub fn session(number: usize) -> ElementLoader {
        Self::load(ElementType::Session, number)
    }

    /// Load a definition variation (fluent API)
    pub fn definition(number: usize) -> ElementLoader {
        Self::load(ElementType::Definition, number)
    }

    /// Load an annotation variation (fluent API)
    pub fn annotation(number: usize) -> ElementLoader {
        Self::load(ElementType::Annotation, number)
    }

    /// Load a verbatim variation (fluent API)
    pub fn verbatim(number: usize) -> ElementLoader {
        Self::load(ElementType::Verbatim, number)
    }

    /// Get the path to a specific element type directory
    fn element_type_dir(element_type: ElementType) -> PathBuf {
        Self::elements_dir().join(element_type.dir_name())
    }

    /// Find the file matching the element type and number
    fn find_file(element_type: ElementType, number: usize) -> Result<PathBuf, ElementSourceError> {
        let dir = Self::element_type_dir(element_type);
        let pattern = format!("{}-{:02}-", element_type.prefix(), number);

        let entries = fs::read_dir(&dir)?;
        for entry in entries {
            let entry = entry?;
            let filename = entry.file_name();
            if let Some(name) = filename.to_str() {
                if name.starts_with(&pattern) && name.ends_with(".lex") {
                    return Ok(entry.path());
                }
            }
        }

        Err(ElementSourceError::FileNotFound(format!(
            "No file found for {:?} number {} in {}",
            element_type,
            number,
            dir.display()
        )))
    }

    /// Get the source string for a specific element variation
    ///
    /// # Example
    /// ```rust,ignore
    /// let source = ElementSources::get_source_for(ElementType::Paragraph, 1).unwrap();
    /// ```
    pub fn get_source_for(
        element_type: ElementType,
        number: usize,
    ) -> Result<String, ElementSourceError> {
        let path = Self::find_file(element_type, number)?;
        let content = fs::read_to_string(&path)?;
        Ok(content)
    }

    /// Get the AST document for a specific element variation using the specified parser
    ///
    /// # Example
    /// ```rust,ignore
    /// let doc = ElementSources::get_ast_for(ElementType::Paragraph, 1, Parser::Reference).unwrap();
    /// ```
    pub fn get_ast_for(
        element_type: ElementType,
        number: usize,
        parser: Parser,
    ) -> Result<Document, ElementSourceError> {
        let source = Self::get_source_for(element_type, number)?;
        parse_with_parser(&source, parser)
    }

    /// List all available numbers for a given element type
    pub fn list_numbers_for(element_type: ElementType) -> Result<Vec<usize>, ElementSourceError> {
        let dir = Self::element_type_dir(element_type);
        let prefix = element_type.prefix();
        let mut numbers = Vec::new();

        let entries = fs::read_dir(&dir)?;
        for entry in entries {
            let entry = entry?;
            let filename = entry.file_name();
            if let Some(name) = filename.to_str() {
                if name.starts_with(prefix) && name.ends_with(".lex") {
                    // Extract number from pattern: "element-NN-hint.lex"
                    if let Some(num_str) = name.strip_prefix(&format!("{}-", prefix)) {
                        if let Some(num_part) = num_str.split('-').next() {
                            if let Ok(num) = num_part.parse::<usize>() {
                                numbers.push(num);
                            }
                        }
                    }
                }
            }
        }

        numbers.sort_unstable();
        numbers.dedup();
        Ok(numbers)
    }
}

/// Parse a source string with a specific parser
pub fn parse_with_parser(source: &str, parser: Parser) -> Result<Document, ElementSourceError> {
    let executor = PipelineExecutor::new();
    let output = executor
        .execute(parser.config_name(), source)
        .map_err(|e| ElementSourceError::ParseError(e.to_string()))?;

    match output {
        ExecutionOutput::Document(doc) => Ok(doc),
        _ => Err(ElementSourceError::ParseError(
            "Expected document output from parser".to_string(),
        )),
    }
}

/// Parse a source string with multiple parsers and compare results
pub fn parse_with_multiple_parsers(
    source: &str,
    parsers: &[Parser],
) -> Result<Vec<(Parser, Document)>, ElementSourceError> {
    let mut results = Vec::new();
    for &parser in parsers {
        let doc = parse_with_parser(source, parser)?;
        results.push((parser, doc));
    }
    Ok(results)
}

/// Compare ASTs from multiple parsers to ensure they produce the same structure
///
/// Returns Ok(()) if all ASTs match, or Err with details about the first mismatch
pub fn compare_parser_results(results: &[(Parser, Document)]) -> Result<(), String> {
    if results.len() < 2 {
        return Ok(());
    }

    let (first_parser, first_doc) = &results[0];
    for (parser, doc) in &results[1..] {
        if !documents_match(first_doc, doc) {
            return Err(format!(
                "AST mismatch between {:?} and {:?} parsers",
                first_parser, parser
            ));
        }
    }

    Ok(())
}

/// Check if two documents have matching AST structure
fn documents_match(doc1: &Document, doc2: &Document) -> bool {
    doc1.root.children.len() == doc2.root.children.len()
        && doc1
            .root
            .children
            .iter()
            .zip(doc2.root.children.iter())
            .all(|(item1, item2)| content_items_match(item1, item2))
}

/// Check if two content items match (recursive)
fn content_items_match(item1: &ContentItem, item2: &ContentItem) -> bool {
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

// ===== Helper functions for extracting elements from AST =====

/// Get the first element of a specific type from the document
pub fn get_first_paragraph(doc: &Document) -> Option<&Paragraph> {
    doc.root.children.iter().find_map(|item| match item {
        ContentItem::Paragraph(p) => Some(p),
        _ => None,
    })
}

/// Get the first session from the document
pub fn get_first_session(doc: &Document) -> Option<&Session> {
    doc.root.children.iter().find_map(|item| match item {
        ContentItem::Session(s) => Some(s),
        _ => None,
    })
}

/// Get the first list from the document
pub fn get_first_list(doc: &Document) -> Option<&List> {
    doc.root.children.iter().find_map(|item| match item {
        ContentItem::List(l) => Some(l),
        _ => None,
    })
}

/// Get the first definition from the document
pub fn get_first_definition(doc: &Document) -> Option<&Definition> {
    doc.root.children.iter().find_map(|item| match item {
        ContentItem::Definition(d) => Some(d),
        _ => None,
    })
}

/// Get the first annotation from the document
pub fn get_first_annotation(doc: &Document) -> Option<&Annotation> {
    doc.root.children.iter().find_map(|item| match item {
        ContentItem::Annotation(a) => Some(a),
        _ => None,
    })
}

/// Get the first verbatim block from the document
/// Note: Returns the boxed Verbatim from VerbatimBlock
pub fn get_first_verbatim(doc: &Document) -> Option<&crate::lex::ast::Verbatim> {
    doc.root.children.iter().find_map(|item| match item {
        ContentItem::VerbatimBlock(v) => Some(v.as_ref()),
        _ => None,
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_type_dir_names() {
        assert_eq!(ElementType::Paragraph.dir_name(), "paragraph");
        assert_eq!(ElementType::List.dir_name(), "list");
        assert_eq!(ElementType::Session.dir_name(), "session");
    }

    #[test]
    fn test_parser_config_names() {
        assert_eq!(Parser::Reference.config_name(), "default");
        assert_eq!(Parser::Linebased.config_name(), "linebased");
    }

    #[test]
    fn test_get_source_for_paragraph() {
        let source = ElementSources::get_source_for(ElementType::Paragraph, 1);
        assert!(source.is_ok(), "Should find paragraph-01 file");
        let content = source.unwrap();
        assert!(!content.is_empty());
    }

    #[test]
    fn test_list_numbers_for_paragraphs() {
        let numbers = ElementSources::list_numbers_for(ElementType::Paragraph).unwrap();
        assert!(!numbers.is_empty());
        assert!(numbers.contains(&1));
    }

    #[test]
    fn test_parse_with_reference_parser() {
        let source = ElementSources::get_source_for(ElementType::Paragraph, 1).unwrap();
        let doc = parse_with_parser(&source, Parser::Reference);
        assert!(doc.is_ok(), "Reference parser should parse successfully");
    }

    #[test]
    fn test_get_first_paragraph() {
        let source = ElementSources::get_source_for(ElementType::Paragraph, 1).unwrap();
        let doc = parse_with_parser(&source, Parser::Reference).unwrap();
        let paragraph = get_first_paragraph(&doc);
        assert!(paragraph.is_some());
    }

    #[test]
    fn test_paragraph_assertions() {
        let source = ElementSources::get_source_for(ElementType::Paragraph, 1).unwrap();
        let doc = parse_with_parser(&source, Parser::Reference).unwrap();
        let paragraph = get_first_paragraph(&doc).unwrap();

        // Note: parsers may have bugs, we're just testing the infrastructure
        // The important part is that we can get a paragraph and check its content
        assert!(paragraph_text_starts_with(paragraph, "This is a simple"));
    }

    // ===== Fluent API Tests =====

    #[test]
    fn test_fluent_api_basic() {
        // Test: ElementSources::paragraph(1).parse().expect_paragraph()
        let parsed = ElementSources::paragraph(1).parse();
        let paragraph = parsed.expect_paragraph();

        assert!(paragraph_text_starts_with(paragraph, "This is a simple"));
    }

    #[test]
    fn test_fluent_api_with_parser_selection() {
        // Test with explicit parser selection
        let parsed = ElementSources::paragraph(1).parse_with(Parser::Reference);
        let paragraph = parsed.expect_paragraph();

        assert!(paragraph_text_starts_with(paragraph, "This is a simple"));
    }

    #[test]
    fn test_fluent_api_source_only() {
        // Get just the source without parsing
        let source = ElementSources::paragraph(1).source();
        assert!(source.contains("simple"));
    }

    #[test]
    fn test_fluent_api_list() {
        let parsed = ElementSources::list(1).parse();
        let list = parsed.expect_list();

        assert!(!list.items.is_empty());
    }

    #[test]
    fn test_fluent_api_session() {
        let parsed = ElementSources::session(1).parse();
        let session = parsed.expect_session();

        assert!(!session.label().is_empty());
    }

    #[test]
    fn test_fluent_api_definition() {
        let parsed = ElementSources::definition(1).parse();
        let definition = parsed.expect_definition();

        assert!(!definition.label().is_empty());
    }

    #[test]
    fn test_must_methods() {
        // Test must_get_source_for
        let source = ElementSources::must_get_source_for(ElementType::Paragraph, 1);
        assert!(!source.is_empty());

        // Test must_get_ast_for
        let doc = ElementSources::must_get_ast_for(ElementType::Paragraph, 1, Parser::Reference);
        assert!(!doc.root.children.is_empty());
    }
}
