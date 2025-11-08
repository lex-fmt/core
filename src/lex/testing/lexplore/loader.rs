//! File loading, parsing, and tokenization for Lex test harness
//!
//! This module provides the core loading infrastructure for the Lexplore test harness,
//! handling file discovery, reading, parsing, and tokenization.

use crate::lex::ast::{Annotation, Definition, Document, List, Paragraph, Session};
use crate::lex::lexing::Token;
use crate::lex::parsing::ParseError;
use crate::lex::pipeline::{ExecutionOutput, PipelineExecutor};
use std::fs;
use std::path::PathBuf;

use super::extraction::*;

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

/// Document collection types for comprehensive testing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentType {
    Benchmark,
    Trifecta,
}

impl DocumentType {
    /// Get the directory name for this document type
    fn dir_name(&self) -> &'static str {
        match self {
            DocumentType::Benchmark => "benchmark",
            DocumentType::Trifecta => "trifecta",
        }
    }
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
    /// Get the pipeline config name for this parser (AST output)
    fn config_name(&self) -> &'static str {
        match self {
            Parser::Reference => "default",
            Parser::Linebased => "linebased",
        }
    }

    /// Get the pipeline config name for tokenization (Token output)
    fn token_config_name(&self) -> &'static str {
        match self {
            Parser::Reference => "tokens-indentation",
            Parser::Linebased => "tokens-linebased-flat",
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

/// Fluent API builder for loading elements or documents
pub struct ElementLoader {
    source_type: SourceType,
    number: usize,
}

/// Enum to represent either an element type, document type, or arbitrary file path
#[derive(Debug)]
enum SourceType {
    Element(ElementType),
    Document(DocumentType),
    Path(PathBuf),
}

impl ElementLoader {
    /// Get the raw source string
    pub fn source(&self) -> String {
        match &self.source_type {
            SourceType::Element(element_type) => {
                Lexplore::must_get_source_for(*element_type, self.number)
            }
            SourceType::Document(doc_type) => {
                Lexplore::must_get_document_source_for(*doc_type, self.number)
            }
            SourceType::Path(path) => Lexplore::must_get_source_from_path(path),
        }
    }

    /// Parse with the specified parser and return a ParsedElement for further chaining
    pub fn parse_with(self, parser: Parser) -> ParsedElement {
        let doc = match &self.source_type {
            SourceType::Element(element_type) => {
                Lexplore::must_get_ast_for(*element_type, self.number, parser)
            }
            SourceType::Document(doc_type) => {
                Lexplore::must_get_document_ast_for(*doc_type, self.number, parser)
            }
            SourceType::Path(path) => Lexplore::must_get_ast_from_path(path, parser),
        };
        ParsedElement {
            source_type: self.source_type,
            doc,
        }
    }

    /// Parse with the Reference parser (shorthand)
    pub fn parse(self) -> ParsedElement {
        self.parse_with(Parser::Reference)
    }

    /// Tokenize with the specified parser and return a ParsedTokens for further inspection
    pub fn tokenize_with(self, parser: Parser) -> ParsedTokens {
        let tokens = match &self.source_type {
            SourceType::Element(element_type) => {
                Lexplore::must_get_tokens_for(*element_type, self.number, parser)
            }
            SourceType::Document(doc_type) => {
                Lexplore::must_get_document_tokens_for(*doc_type, self.number, parser)
            }
            SourceType::Path(path) => Lexplore::must_get_tokens_from_path(path, parser),
        };
        ParsedTokens {
            source_type: self.source_type,
            tokens,
        }
    }

    /// Tokenize with the Reference parser (shorthand)
    pub fn tokenize(self) -> ParsedTokens {
        self.tokenize_with(Parser::Reference)
    }
}

/// Tokenized source, ready for token inspection
pub struct ParsedTokens {
    #[allow(dead_code)] // Kept for symmetry with ParsedElement, may be used for debugging
    source_type: SourceType,
    tokens: Vec<(Token, std::ops::Range<usize>)>,
}

impl ParsedTokens {
    /// Get the underlying token stream
    pub fn tokens(&self) -> &[(Token, std::ops::Range<usize>)] {
        &self.tokens
    }

    /// Get the count of tokens
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    /// Check if there are no tokens
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    /// Find first token matching a predicate
    pub fn find_token<F>(&self, predicate: F) -> Option<&Token>
    where
        F: Fn(&Token) -> bool,
    {
        self.tokens
            .iter()
            .find(|(t, _)| predicate(t))
            .map(|(t, _)| t)
    }

    /// Count tokens matching a predicate
    pub fn count_tokens<F>(&self, predicate: F) -> usize
    where
        F: Fn(&Token) -> bool,
    {
        self.tokens.iter().filter(|(t, _)| predicate(t)).count()
    }

    /// Check if any token matches a predicate
    pub fn has_token<F>(&self, predicate: F) -> bool
    where
        F: Fn(&Token) -> bool,
    {
        self.tokens.iter().any(|(t, _)| predicate(t))
    }

    /// Detokenize the token stream back to source text
    ///
    /// This is a convenience method that extracts the tokens and uses the
    /// detokenizer to convert them back to source text.
    ///
    /// # Example
    /// ```rust,ignore
    /// use lex::lex::testing::lexplore::Lexplore;
    ///
    /// let source = Lexplore::from_path("docs/specs/v1/benchmark/010-kitchensink.lex")
    ///     .tokenize()
    ///     .detokenize();
    /// ```
    pub fn detokenize(&self) -> String {
        let tokens: Vec<_> = self.tokens.iter().map(|(t, _)| t.clone()).collect();
        crate::lex::formats::detokenize(&tokens)
    }
}

/// A parsed element document, ready for element extraction
pub struct ParsedElement {
    source_type: SourceType,
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
            .unwrap_or_else(|| panic!("No paragraph found in {:?} document", self.source_type))
    }

    /// Get the first session, panicking if not found
    pub fn expect_session(&self) -> &Session {
        get_first_session(&self.doc)
            .unwrap_or_else(|| panic!("No session found in {:?} document", self.source_type))
    }

    /// Get the first list, panicking if not found
    pub fn expect_list(&self) -> &List {
        get_first_list(&self.doc)
            .unwrap_or_else(|| panic!("No list found in {:?} document", self.source_type))
    }

    /// Get the first definition, panicking if not found
    pub fn expect_definition(&self) -> &Definition {
        get_first_definition(&self.doc)
            .unwrap_or_else(|| panic!("No definition found in {:?} document", self.source_type))
    }

    /// Get the first annotation, panicking if not found
    pub fn expect_annotation(&self) -> &Annotation {
        get_first_annotation(&self.doc)
            .unwrap_or_else(|| panic!("No annotation found in {:?} document", self.source_type))
    }

    /// Get the first verbatim block, panicking if not found
    pub fn expect_verbatim(&self) -> &crate::lex::ast::Verbatim {
        get_first_verbatim(&self.doc)
            .unwrap_or_else(|| panic!("No verbatim found in {:?} document", self.source_type))
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
pub struct Lexplore;

impl Lexplore {
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

    /// Get tokens, panicking if not found or tokenization fails
    pub fn must_get_tokens_for(
        element_type: ElementType,
        number: usize,
        parser: Parser,
    ) -> Vec<(Token, std::ops::Range<usize>)> {
        Self::get_tokens_for(element_type, number, parser).unwrap_or_else(|e| {
            panic!(
                "Failed to load/tokenize {:?} #{}: {}",
                element_type, number, e
            )
        })
    }

    // ===== Fluent API - start a chain =====

    /// Start a fluent chain for loading and parsing an element
    ///
    /// # Example
    /// ```rust,ignore
    /// let doc = Lexplore::load(ElementType::Paragraph, 1)
    ///     .parse_with(Parser::Reference);
    /// ```
    pub fn load(element_type: ElementType, number: usize) -> ElementLoader {
        ElementLoader {
            source_type: SourceType::Element(element_type),
            number,
        }
    }

    /// Start a fluent chain for loading and parsing a document collection
    ///
    /// # Example
    /// ```rust,ignore
    /// let doc = Lexplore::load_document(DocumentType::Benchmark, 10)
    ///     .parse_with(Parser::Reference);
    /// ```
    pub fn load_document(doc_type: DocumentType, number: usize) -> ElementLoader {
        ElementLoader {
            source_type: SourceType::Document(doc_type),
            number,
        }
    }

    /// Start a fluent chain for loading and parsing from an arbitrary file path
    ///
    /// # Example
    /// ```rust,ignore
    /// let doc = Lexplore::from_path("path/to/file.lex")
    ///     .parse_with(Parser::Reference);
    /// ```
    pub fn from_path<P: Into<PathBuf>>(path: P) -> ElementLoader {
        ElementLoader {
            source_type: SourceType::Path(path.into()),
            number: 0, // Dummy value, not used for Path variant
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

    // ===== Convenience shortcuts for document collections =====

    /// Load a benchmark document (fluent API)
    pub fn benchmark(number: usize) -> ElementLoader {
        Self::load_document(DocumentType::Benchmark, number)
    }

    /// Load a trifecta document (fluent API)
    pub fn trifecta(number: usize) -> ElementLoader {
        Self::load_document(DocumentType::Trifecta, number)
    }

    /// Get the path to a specific element type directory
    fn element_type_dir(element_type: ElementType) -> PathBuf {
        Self::elements_dir().join(element_type.dir_name())
    }

    /// Get the path to a specific document type directory
    fn document_type_dir(doc_type: DocumentType) -> PathBuf {
        Self::elements_dir()
            .parent()
            .unwrap()
            .join(doc_type.dir_name())
    }

    /// Find the file matching the element type and number
    ///
    /// # Panics
    ///
    /// Panics if multiple files exist with the same number. This is a critical error
    /// that indicates the test corpus has duplicate numbers, which violates the design
    /// where each number uniquely identifies a test case.
    fn find_file(element_type: ElementType, number: usize) -> Result<PathBuf, ElementSourceError> {
        let dir = Self::element_type_dir(element_type);
        let pattern = format!("{}-{:02}-", element_type.prefix(), number);

        // Collect all matching files to detect duplicates
        let mut matching_files: Vec<PathBuf> = Vec::new();
        let entries = fs::read_dir(&dir)?;
        for entry in entries {
            let entry = entry?;
            let filename = entry.file_name();
            if let Some(name) = filename.to_str() {
                if name.starts_with(&pattern) && name.ends_with(".lex") {
                    matching_files.push(entry.path());
                }
            }
        }

        match matching_files.len() {
            0 => Err(ElementSourceError::FileNotFound(format!(
                "No file found for {:?} number {} in {}",
                element_type,
                number,
                dir.display()
            ))),
            1 => Ok(matching_files[0].clone()),
            _ => {
                // Multiple files with the same number - this is a critical error
                let file_list = matching_files
                    .iter()
                    .map(|p| format!("  - {}", p.file_name().unwrap().to_string_lossy()))
                    .collect::<Vec<_>>()
                    .join("\n");
                panic!(
                    "DUPLICATE TEST NUMBERS DETECTED!\n\
                    Found {} files with number {:02} for {:?}:\n\
                    {}\n\n\
                    ERROR: Test numbers must be unique within each element directory.\n\
                    FIX: Rename the duplicate files to use unique numbers.\n\
                    Directory: {}",
                    matching_files.len(),
                    number,
                    element_type,
                    file_list,
                    dir.display()
                );
            }
        }
    }

    /// Find the file matching the document type and number
    ///
    /// # Panics
    ///
    /// Panics if multiple files exist with the same number. This is a critical error
    /// that indicates the test corpus has duplicate numbers, which violates the design
    /// where each number uniquely identifies a test case.
    fn find_document_file(
        doc_type: DocumentType,
        number: usize,
    ) -> Result<PathBuf, ElementSourceError> {
        let dir = Self::document_type_dir(doc_type);
        let pattern = format!("{:03}-", number);

        // Collect all matching files to detect duplicates
        let mut matching_files: Vec<PathBuf> = Vec::new();
        let entries = fs::read_dir(&dir)?;
        for entry in entries {
            let entry = entry?;
            let filename = entry.file_name();
            if let Some(name) = filename.to_str() {
                if name.starts_with(&pattern) && name.ends_with(".lex") {
                    matching_files.push(entry.path());
                }
            }
        }

        match matching_files.len() {
            0 => Err(ElementSourceError::FileNotFound(format!(
                "No file found for {:?} number {} in {}",
                doc_type,
                number,
                dir.display()
            ))),
            1 => Ok(matching_files[0].clone()),
            _ => {
                // Multiple files with the same number - this is a critical error
                let file_list = matching_files
                    .iter()
                    .map(|p| format!("  - {}", p.file_name().unwrap().to_string_lossy()))
                    .collect::<Vec<_>>()
                    .join("\n");
                panic!(
                    "DUPLICATE TEST NUMBERS DETECTED!\n\
                    Found {} files with number {:03} for {:?}:\n\
                    {}\n\n\
                    ERROR: Test numbers must be unique within each document directory.\n\
                    FIX: Rename the duplicate files to use unique numbers.\n\
                    Directory: {}",
                    matching_files.len(),
                    number,
                    doc_type,
                    file_list,
                    dir.display()
                );
            }
        }
    }

    /// Get the source string for a specific element variation
    ///
    /// # Example
    /// ```rust,ignore
    /// let source = Lexplore::get_source_for(ElementType::Paragraph, 1).unwrap();
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
    /// let doc = Lexplore::get_ast_for(ElementType::Paragraph, 1, Parser::Reference).unwrap();
    /// ```
    pub fn get_ast_for(
        element_type: ElementType,
        number: usize,
        parser: Parser,
    ) -> Result<Document, ElementSourceError> {
        let source = Self::get_source_for(element_type, number)?;
        parse_with_parser(&source, parser)
    }

    /// Get the tokens for a specific element variation using the specified parser
    ///
    /// # Example
    /// ```rust,ignore
    /// let tokens = Lexplore::get_tokens_for(ElementType::Paragraph, 1, Parser::Reference).unwrap();
    /// ```
    pub fn get_tokens_for(
        element_type: ElementType,
        number: usize,
        parser: Parser,
    ) -> Result<Vec<(Token, std::ops::Range<usize>)>, ElementSourceError> {
        let source = Self::get_source_for(element_type, number)?;
        tokenize_with_parser(&source, parser)
    }

    /// Get the source string for a specific document collection
    ///
    /// # Example
    /// ```rust,ignore
    /// let source = Lexplore::get_document_source_for(DocumentType::Benchmark, 10).unwrap();
    /// ```
    pub fn get_document_source_for(
        doc_type: DocumentType,
        number: usize,
    ) -> Result<String, ElementSourceError> {
        let path = Self::find_document_file(doc_type, number)?;
        let content = fs::read_to_string(&path)?;
        Ok(content)
    }

    /// Get source string for a document, panicking with helpful message if not found
    pub fn must_get_document_source_for(doc_type: DocumentType, number: usize) -> String {
        Self::get_document_source_for(doc_type, number)
            .unwrap_or_else(|e| panic!("Failed to load {:?} #{}: {}", doc_type, number, e))
    }

    /// Get the AST document for a specific document collection using the specified parser
    ///
    /// # Example
    /// ```rust,ignore
    /// let doc = Lexplore::get_document_ast_for(DocumentType::Benchmark, 10, Parser::Reference).unwrap();
    /// ```
    pub fn get_document_ast_for(
        doc_type: DocumentType,
        number: usize,
        parser: Parser,
    ) -> Result<Document, ElementSourceError> {
        let source = Self::get_document_source_for(doc_type, number)?;
        parse_with_parser(&source, parser)
    }

    /// Get AST document for a document collection, panicking if not found or parse fails
    pub fn must_get_document_ast_for(
        doc_type: DocumentType,
        number: usize,
        parser: Parser,
    ) -> Document {
        Self::get_document_ast_for(doc_type, number, parser)
            .unwrap_or_else(|e| panic!("Failed to load/parse {:?} #{}: {}", doc_type, number, e))
    }

    /// Get the tokens for a specific document collection using the specified parser
    ///
    /// # Example
    /// ```rust,ignore
    /// let tokens = Lexplore::get_document_tokens_for(DocumentType::Benchmark, 10, Parser::Reference).unwrap();
    /// ```
    pub fn get_document_tokens_for(
        doc_type: DocumentType,
        number: usize,
        parser: Parser,
    ) -> Result<Vec<(Token, std::ops::Range<usize>)>, ElementSourceError> {
        let source = Self::get_document_source_for(doc_type, number)?;
        tokenize_with_parser(&source, parser)
    }

    /// Get tokens for a document collection, panicking if not found or tokenization fails
    pub fn must_get_document_tokens_for(
        doc_type: DocumentType,
        number: usize,
        parser: Parser,
    ) -> Vec<(Token, std::ops::Range<usize>)> {
        Self::get_document_tokens_for(doc_type, number, parser)
            .unwrap_or_else(|e| panic!("Failed to load/tokenize {:?} #{}: {}", doc_type, number, e))
    }

    // ===== Path-based loading methods =====

    /// Get the source string from an arbitrary file path
    ///
    /// # Example
    /// ```rust,ignore
    /// let source = Lexplore::get_source_from_path("path/to/file.lex").unwrap();
    /// ```
    pub fn get_source_from_path<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<String, ElementSourceError> {
        let content = fs::read_to_string(path.as_ref())?;
        Ok(content)
    }

    /// Get source string from path, panicking with helpful message if not found
    pub fn must_get_source_from_path<P: AsRef<std::path::Path>>(path: P) -> String {
        Self::get_source_from_path(&path)
            .unwrap_or_else(|e| panic!("Failed to load {:?}: {}", path.as_ref().display(), e))
    }

    /// Get the AST document from a file path using the specified parser
    ///
    /// # Example
    /// ```rust,ignore
    /// let doc = Lexplore::get_ast_from_path("path/to/file.lex", Parser::Reference).unwrap();
    /// ```
    pub fn get_ast_from_path<P: AsRef<std::path::Path>>(
        path: P,
        parser: Parser,
    ) -> Result<Document, ElementSourceError> {
        let source = Self::get_source_from_path(&path)?;
        parse_with_parser(&source, parser)
    }

    /// Get AST document from path, panicking if not found or parse fails
    pub fn must_get_ast_from_path<P: AsRef<std::path::Path>>(path: P, parser: Parser) -> Document {
        Self::get_ast_from_path(&path, parser)
            .unwrap_or_else(|e| panic!("Failed to load/parse {:?}: {}", path.as_ref().display(), e))
    }

    /// Get the tokens from a file path using the specified parser
    ///
    /// # Example
    /// ```rust,ignore
    /// let tokens = Lexplore::get_tokens_from_path("path/to/file.lex", Parser::Reference).unwrap();
    /// ```
    pub fn get_tokens_from_path<P: AsRef<std::path::Path>>(
        path: P,
        parser: Parser,
    ) -> Result<Vec<(Token, std::ops::Range<usize>)>, ElementSourceError> {
        let source = Self::get_source_from_path(&path)?;
        tokenize_with_parser(&source, parser)
    }

    /// Get tokens from path, panicking if not found or tokenization fails
    pub fn must_get_tokens_from_path<P: AsRef<std::path::Path>>(
        path: P,
        parser: Parser,
    ) -> Vec<(Token, std::ops::Range<usize>)> {
        Self::get_tokens_from_path(&path, parser).unwrap_or_else(|e| {
            panic!(
                "Failed to load/tokenize {:?}: {}",
                path.as_ref().display(),
                e
            )
        })
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

/// Tokenize a source string with a specific parser
pub fn tokenize_with_parser(
    source: &str,
    parser: Parser,
) -> Result<Vec<(Token, std::ops::Range<usize>)>, ElementSourceError> {
    let executor = PipelineExecutor::new();
    let output = executor
        .execute(parser.token_config_name(), source)
        .map_err(|e| ElementSourceError::ParseError(e.to_string()))?;

    match output {
        ExecutionOutput::Tokens(stream) => Ok(stream.unroll()),
        _ => Err(ElementSourceError::ParseError(
            "Expected tokens output from tokenizer".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::ast::traits::Container;
    use crate::lex::lexing::Token;

    #[test]
    fn test_get_source_for_paragraph() {
        let source = Lexplore::get_source_for(ElementType::Paragraph, 1);
        assert!(source.is_ok(), "Should find paragraph-01 file");
        let content = source.unwrap();
        assert!(!content.is_empty());
    }

    #[test]
    fn test_list_numbers_for_paragraphs() {
        let numbers = Lexplore::list_numbers_for(ElementType::Paragraph).unwrap();
        assert!(!numbers.is_empty());
        assert!(numbers.contains(&1));
    }

    #[test]
    fn test_parse_with_reference_parser() {
        let source = Lexplore::get_source_for(ElementType::Paragraph, 1).unwrap();
        let doc = parse_with_parser(&source, Parser::Reference);
        assert!(doc.is_ok(), "Reference parser should parse successfully");
    }

    // ===== Fluent API Tests =====

    #[test]
    fn test_fluent_api_basic() {
        let parsed = Lexplore::paragraph(1).parse();
        let paragraph = parsed.expect_paragraph();

        assert!(paragraph_text_starts_with(paragraph, "This is a simple"));
    }

    #[test]
    fn test_fluent_api_with_parser_selection() {
        let parsed = Lexplore::paragraph(1).parse_with(Parser::Reference);
        let paragraph = parsed.expect_paragraph();

        assert!(paragraph_text_starts_with(paragraph, "This is a simple"));
    }

    #[test]
    fn test_fluent_api_source_only() {
        let source = Lexplore::paragraph(1).source();
        assert!(source.contains("simple"));
    }

    #[test]
    fn test_fluent_api_list() {
        let parsed = Lexplore::list(1).parse();
        let list = parsed.expect_list();

        assert!(!list.items.is_empty());
    }

    #[test]
    fn test_fluent_api_session() {
        let parsed = Lexplore::session(1).parse();
        let session = parsed.expect_session();

        assert!(!session.label().is_empty());
    }

    #[test]
    fn test_fluent_api_definition() {
        let parsed = Lexplore::definition(1).parse();
        let definition = parsed.expect_definition();

        assert!(!definition.label().is_empty());
    }

    #[test]
    fn test_must_methods() {
        let source = Lexplore::must_get_source_for(ElementType::Paragraph, 1);
        assert!(!source.is_empty());

        let doc = Lexplore::must_get_ast_for(ElementType::Paragraph, 1, Parser::Reference);
        assert!(!doc.root.children.is_empty());
    }

    // ===== Document Collection Tests =====

    #[test]
    fn test_benchmark_fluent_api() {
        let parsed = Lexplore::benchmark(10).parse();
        let doc = parsed.document();

        assert!(!doc.root.children.is_empty());
    }

    #[test]
    fn test_trifecta_fluent_api() {
        let parsed = Lexplore::trifecta(0).parse();
        let doc = parsed.document();

        assert!(!doc.root.children.is_empty());
    }

    #[test]
    fn test_benchmark_source_only() {
        let source = Lexplore::benchmark(10).source();
        assert!(!source.is_empty());
    }

    #[test]
    fn test_trifecta_source_only() {
        let source = Lexplore::trifecta(0).source();
        assert!(!source.is_empty());
    }

    #[test]
    fn test_get_document_source_for() {
        let source = Lexplore::get_document_source_for(DocumentType::Benchmark, 10);
        assert!(source.is_ok(), "Should find benchmark-010 file");
        let content = source.unwrap();
        assert!(!content.is_empty());
    }

    #[test]
    fn test_must_get_document_source_for() {
        let source = Lexplore::must_get_document_source_for(DocumentType::Trifecta, 0);
        assert!(!source.is_empty());
    }

    #[test]
    fn test_get_document_ast_for() {
        let doc = Lexplore::get_document_ast_for(DocumentType::Benchmark, 10, Parser::Reference);
        assert!(doc.is_ok(), "Should parse benchmark document");
        assert!(!doc.unwrap().root.children.is_empty());
    }

    #[test]
    fn test_must_get_document_ast_for() {
        let doc = Lexplore::must_get_document_ast_for(DocumentType::Trifecta, 0, Parser::Reference);
        assert!(!doc.root.children.is_empty());
    }

    // ===== Tokenization Tests =====

    #[test]
    fn test_tokenize_paragraph() {
        let parsed_tokens = Lexplore::paragraph(1).tokenize();

        assert!(!parsed_tokens.is_empty());
    }

    #[test]
    fn test_tokenize_with_parser() {
        let parsed_tokens = Lexplore::paragraph(1).tokenize_with(Parser::Reference);

        assert!(!parsed_tokens.is_empty());
        assert!(parsed_tokens.has_token(|t| matches!(t, Token::Text(_))));
    }

    #[test]
    fn test_tokenize_list() {
        let parsed_tokens = Lexplore::list(1).tokenize();

        assert!(
            parsed_tokens.has_token(|t| matches!(t, Token::Dash))
                || parsed_tokens.has_token(|t| matches!(t, Token::Number(_)))
        );
    }

    #[test]
    fn test_tokenize_benchmark() {
        let parsed_tokens = Lexplore::benchmark(10).tokenize();

        assert!(!parsed_tokens.is_empty());
        assert!(parsed_tokens.len() > 10);
    }

    #[test]
    fn test_tokenize_trifecta() {
        let parsed_tokens = Lexplore::trifecta(0).tokenize();

        assert!(!parsed_tokens.is_empty());
        assert!(parsed_tokens.has_token(|t| matches!(t, Token::Text(_))));
    }

    #[test]
    fn test_get_tokens_for() {
        let tokens = Lexplore::get_tokens_for(ElementType::Paragraph, 1, Parser::Reference);
        assert!(tokens.is_ok());
        assert!(!tokens.unwrap().is_empty());
    }

    #[test]
    fn test_must_get_tokens_for() {
        let tokens = Lexplore::must_get_tokens_for(ElementType::Paragraph, 1, Parser::Reference);
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_get_document_tokens_for() {
        let tokens =
            Lexplore::get_document_tokens_for(DocumentType::Benchmark, 10, Parser::Reference);
        assert!(tokens.is_ok());
        assert!(!tokens.unwrap().is_empty());
    }

    #[test]
    fn test_must_get_document_tokens_for() {
        let tokens =
            Lexplore::must_get_document_tokens_for(DocumentType::Trifecta, 0, Parser::Reference);
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_parsed_tokens_methods() {
        let parsed_tokens = Lexplore::paragraph(1).tokenize();

        assert!(!parsed_tokens.is_empty());

        let tokens = parsed_tokens.tokens();
        assert!(!tokens.is_empty());

        let text_token = parsed_tokens.find_token(|t| matches!(t, Token::Text(_)));
        assert!(text_token.is_some());

        let text_count = parsed_tokens.count_tokens(|t| matches!(t, Token::Text(_)));
        assert!(text_count > 0);

        assert!(parsed_tokens.has_token(|t| matches!(t, Token::Text(_))));
        assert!(parsed_tokens.has_token(|t| matches!(t, Token::Newline)));
    }

    #[test]
    fn test_tokenize_with_parser_function() {
        let source = Lexplore::must_get_source_for(ElementType::Paragraph, 1);
        let tokens = tokenize_with_parser(&source, Parser::Reference);

        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_tokenize_linebased_parser() {
        let parsed_tokens = Lexplore::paragraph(1).tokenize_with(Parser::Linebased);

        assert!(!parsed_tokens.is_empty());
        assert!(parsed_tokens.has_token(|t| matches!(t, Token::Text(_))));
    }

    // ===== Path-based Loading Tests =====

    #[test]
    fn test_from_path_parse() {
        let path = "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex";
        let parsed = Lexplore::from_path(path).parse();

        let paragraph = parsed.expect_paragraph();
        assert!(!paragraph.text().is_empty());
    }

    #[test]
    fn test_from_path_tokenize() {
        let path = "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex";
        let tokens = Lexplore::from_path(path).tokenize();

        assert!(!tokens.is_empty());
        assert!(tokens.has_token(|t| matches!(t, Token::Text(_))));
    }

    #[test]
    fn test_from_path_source() {
        let path = "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex";
        let source = Lexplore::from_path(path).source();

        assert!(!source.is_empty());
    }

    #[test]
    fn test_from_path_with_parser() {
        let path = "docs/specs/v1/elements/list/list-01-flat-simple-dash.lex";
        let parsed = Lexplore::from_path(path).parse_with(Parser::Reference);

        let list = parsed.expect_list();
        assert!(!list.items.is_empty());
    }

    #[test]
    fn test_from_path_tokenize_with_parser() {
        let path = "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex";
        let tokens = Lexplore::from_path(path).tokenize_with(Parser::Linebased);

        assert!(!tokens.is_empty());
        assert!(tokens.has_token(|t| matches!(t, Token::Text(_))));
    }

    #[test]
    fn test_get_source_from_path() {
        let path = "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex";
        let source = Lexplore::get_source_from_path(path);

        assert!(source.is_ok());
        assert!(!source.unwrap().is_empty());
    }

    #[test]
    fn test_must_get_source_from_path() {
        let path = "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex";
        let source = Lexplore::must_get_source_from_path(path);

        assert!(!source.is_empty());
    }

    #[test]
    fn test_get_ast_from_path() {
        let path = "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex";
        let doc = Lexplore::get_ast_from_path(path, Parser::Reference);

        assert!(doc.is_ok());
        assert!(!doc.unwrap().root.children.is_empty());
    }

    #[test]
    fn test_must_get_ast_from_path() {
        let path = "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex";
        let doc = Lexplore::must_get_ast_from_path(path, Parser::Reference);

        assert!(!doc.root.children.is_empty());
    }

    #[test]
    fn test_get_tokens_from_path() {
        let path = "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex";
        let tokens = Lexplore::get_tokens_from_path(path, Parser::Reference);

        assert!(tokens.is_ok());
        assert!(!tokens.unwrap().is_empty());
    }

    #[test]
    fn test_must_get_tokens_from_path() {
        let path = "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex";
        let tokens = Lexplore::must_get_tokens_from_path(path, Parser::Reference);

        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_from_path_with_benchmark() {
        let path = "docs/specs/v1/benchmark/010-kitchensink.lex";
        let parsed = Lexplore::from_path(path).parse();

        let doc = parsed.document();
        assert!(!doc.root.children.is_empty());
    }

    #[test]
    fn test_from_path_with_trifecta() {
        let path = "docs/specs/v1/trifecta/000-paragraphs.lex";
        let parsed = Lexplore::from_path(path).parse();

        let doc = parsed.document();
        assert!(!doc.root.children.is_empty());
    }

    // ===== Detokenization Tests =====

    #[test]
    fn test_detokenize_paragraph() {
        let source = Lexplore::paragraph(1).source();
        let detokenized = Lexplore::paragraph(1).tokenize().detokenize();

        // Detokenized should match original source
        assert_eq!(detokenized, source);
    }

    #[test]
    fn test_detokenize_benchmark() {
        let detokenized = Lexplore::benchmark(10).tokenize().detokenize();

        // Verify detokenization produces non-empty output with expected content
        assert!(!detokenized.is_empty());
        assert!(detokenized.contains("Kitchensink Test Document"));
        assert!(detokenized.contains("1. Primary Session"));
        assert!(detokenized.contains("2. Second Root Session"));
    }

    #[test]
    fn test_detokenize_from_path() {
        let path = "docs/specs/v1/benchmark/010-kitchensink.lex";
        let detokenized = Lexplore::from_path(path).tokenize().detokenize();

        // Verify detokenization produces expected content
        assert!(!detokenized.is_empty());
        assert!(detokenized.contains("Kitchensink Test Document"));
        assert!(detokenized.contains("Root Definition:"));
        assert!(detokenized.contains("Nested Session"));
    }

    #[test]
    fn test_detokenize_with_semantic_tokens() {
        let source = Lexplore::session(1).source();
        let detokenized = Lexplore::session(1).tokenize().detokenize();

        // Detokenized should match original source (handles Indent/Dedent tokens)
        assert_eq!(detokenized, source);
    }

    #[test]
    fn test_detokenize_trifecta() {
        let source = Lexplore::trifecta(0).source();
        let detokenized = Lexplore::trifecta(0).tokenize().detokenize();

        // Detokenized should match original source
        assert_eq!(detokenized, source);
    }

    #[test]
    fn test_detokenize_fluent_api() {
        // Demonstrate fluent API usage
        let detokenized = Lexplore::from_path("docs/specs/v1/benchmark/010-kitchensink.lex")
            .tokenize()
            .detokenize();

        assert!(detokenized.contains("Kitchensink"));
    }
}
