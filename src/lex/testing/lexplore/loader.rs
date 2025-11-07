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

    /// Find the file matching the document type and number
    fn find_document_file(
        doc_type: DocumentType,
        number: usize,
    ) -> Result<PathBuf, ElementSourceError> {
        let dir = Self::document_type_dir(doc_type);
        let pattern = format!("{:03}-", number);

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
            doc_type,
            number,
            dir.display()
        )))
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
