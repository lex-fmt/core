//! File loading, parsing, and tokenization for Lex test harness
//!
//! This module provides the core loading infrastructure for the Lexplore test harness,
//! handling file discovery, reading, parsing, and tokenization.

use crate::lex::ast::{Annotation, Definition, Document, List, Paragraph, Session};
use crate::lex::lexing::Token;
use crate::lex::parsing::ParseError;
use crate::lex::pipeline::DocumentLoader;
use std::fs;
use std::path::PathBuf;

// Re-export Parser from pipeline for backward compatibility
pub use crate::lex::pipeline::Parser;

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

// Parser enum is now defined in crate::lex::pipeline::loader and re-exported from pipeline module

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
    /// Get the file path for this loader
    fn get_path(&self) -> PathBuf {
        match &self.source_type {
            SourceType::Element(element_type) => Lexplore::find_file(*element_type, self.number)
                .unwrap_or_else(|e| {
                    panic!("Failed to find {:?} #{}: {}", element_type, self.number, e)
                }),
            SourceType::Document(doc_type) => Lexplore::find_document_file(*doc_type, self.number)
                .unwrap_or_else(|e| {
                    panic!("Failed to find {:?} #{}: {}", doc_type, self.number, e)
                }),
            SourceType::Path(path) => path.clone(),
        }
    }

    /// Get the raw source string
    pub fn source(&self) -> String {
        let path = self.get_path();
        fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e))
    }

    /// Parse with the specified parser and return a ParsedElement for further chaining
    pub fn parse_with(self, parser: Parser) -> ParsedElement {
        let path = self.get_path();
        let loader = DocumentLoader::new();
        let doc = loader
            .load_and_parse_with(&path, parser)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e));
        ParsedElement {
            source_type: self.source_type,
            doc,
        }
    }

    /// Parse with the Reference parser (shorthand)
    pub fn parse(self) -> ParsedElement {
        self.parse_with(crate::lex::pipeline::Parser::Reference)
    }

    /// Tokenize with the specified parser and return a ParsedTokens for further inspection
    pub fn tokenize_with(self, parser: Parser) -> ParsedTokens {
        let path = self.get_path();
        let loader = DocumentLoader::new();
        let tokens = loader
            .load_and_tokenize_with(&path, parser)
            .unwrap_or_else(|e| panic!("Failed to tokenize {}: {}", path.display(), e));
        ParsedTokens {
            source_type: self.source_type,
            tokens,
        }
    }

    /// Tokenize with the Reference parser (shorthand)
    pub fn tokenize(self) -> ParsedTokens {
        self.tokenize_with(crate::lex::pipeline::Parser::Reference)
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

/// Macro to generate element extraction methods (expect_* and first_*)
macro_rules! impl_element_extractors {
    ($($name:ident => $iter_method:ident, $type:ty, $label:literal);* $(;)?) => {
        $(
            #[doc = concat!("Get the first ", $label, ", panicking if not found")]
            pub fn $name(&self) -> &$type {
                self.doc
                    .$iter_method()
                    .next()
                    .unwrap_or_else(|| panic!(concat!("No ", $label, " found in {:?} document"), self.source_type))
            }
        )*
    };
}

/// Macro to generate optional element extraction methods (first_*)
macro_rules! impl_optional_extractors {
    ($($name:ident => $iter_method:ident, $type:ty);* $(;)?) => {
        $(
            #[doc = concat!("Get the first ", stringify!($name), " (returns Option)")]
            pub fn $name(&self) -> Option<&$type> {
                self.doc.$iter_method().next()
            }
        )*
    };
}

impl ParsedElement {
    /// Get the underlying document
    pub fn document(&self) -> &Document {
        &self.doc
    }

    impl_element_extractors! {
        expect_paragraph => iter_paragraphs_recursive, Paragraph, "paragraph";
        expect_session => iter_sessions_recursive, Session, "session";
        expect_list => iter_lists_recursive, List, "list";
        expect_definition => iter_definitions_recursive, Definition, "definition";
        expect_annotation => iter_annotations_recursive, Annotation, "annotation";
        expect_verbatim => iter_verbatim_blocks_recursive, crate::lex::ast::Verbatim, "verbatim";
    }

    impl_optional_extractors! {
        first_paragraph => iter_paragraphs_recursive, Paragraph;
        first_session => iter_sessions_recursive, Session;
        first_list => iter_lists_recursive, List;
        first_definition => iter_definitions_recursive, Definition;
        first_annotation => iter_annotations_recursive, Annotation;
        first_verbatim => iter_verbatim_blocks_recursive, crate::lex::ast::Verbatim;
    }
}

/// Macro to generate element loader shortcuts
macro_rules! element_shortcuts {
    ($($name:ident => $variant:ident, $label:literal);* $(;)?) => {
        $(
            #[doc = concat!("Load a ", $label, " variation (fluent API)")]
            pub fn $name(number: usize) -> ElementLoader {
                Self::load(ElementType::$variant, number)
            }
        )*
    };
}

/// Macro to generate document loader shortcuts
macro_rules! document_shortcuts {
    ($($name:ident => $variant:ident, $label:literal);* $(;)?) => {
        $(
            #[doc = concat!("Load a ", $label, " document (fluent API)")]
            pub fn $name(number: usize) -> ElementLoader {
                Self::load_document(DocumentType::$variant, number)
            }
        )*
    };
}

/// Interface for loading per-element test sources
pub struct Lexplore;

impl Lexplore {
    const SPEC_VERSION: &'static str = "v1";

    /// Get the path to the elements directory
    fn elements_dir() -> PathBuf {
        PathBuf::from(format!("docs/specs/{}/elements", Self::SPEC_VERSION))
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

    element_shortcuts! {
        paragraph => Paragraph, "paragraph";
        list => List, "list";
        session => Session, "session";
        definition => Definition, "definition";
        annotation => Annotation, "annotation";
        verbatim => Verbatim, "verbatim";
    }

    // ===== Convenience shortcuts for document collections =====

    document_shortcuts! {
        benchmark => Benchmark, "benchmark";
        trifecta => Trifecta, "trifecta";
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

    /// Generic file finder that searches for files matching a pattern in a directory
    ///
    /// # Panics
    ///
    /// Panics if multiple files exist with the same number. This is a critical error
    /// that indicates the test corpus has duplicate numbers.
    fn find_file_by_pattern(
        dir: PathBuf,
        pattern: &str,
        type_name: &str,
        number: usize,
        number_format: &str,
    ) -> Result<PathBuf, ElementSourceError> {
        // Collect all matching files to detect duplicates
        let mut matching_files: Vec<PathBuf> = Vec::new();
        let entries = fs::read_dir(&dir)?;
        for entry in entries {
            let entry = entry?;
            let filename = entry.file_name();
            if let Some(name) = filename.to_str() {
                if name.starts_with(pattern) && name.ends_with(".lex") {
                    matching_files.push(entry.path());
                }
            }
        }

        match matching_files.len() {
            0 => Err(ElementSourceError::FileNotFound(format!(
                "No file found for {} number {} in {}",
                type_name,
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
                    Found {} files with number {} for {}:\n\
                    {}\n\n\
                    ERROR: Test numbers must be unique within each directory.\n\
                    FIX: Rename the duplicate files to use unique numbers.\n\
                    Directory: {}",
                    matching_files.len(),
                    number_format.to_string().replace("{}", &number.to_string()),
                    type_name,
                    file_list,
                    dir.display()
                );
            }
        }
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
        Self::find_file_by_pattern(
            dir,
            &pattern,
            &format!("{:?}", element_type),
            number,
            "{:02}",
        )
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
        Self::find_file_by_pattern(dir, &pattern, &format!("{:?}", doc_type), number, "{:03}")
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::ast::traits::Container;
    use crate::lex::lexing::Token;
    use crate::lex::testing::lexplore::extraction::*;

    // Tests for the old direct API (get_source_for, etc.) have been removed.
    // Use the fluent API instead: Lexplore::paragraph(1).parse()

    #[test]
    fn test_list_numbers_for_paragraphs() {
        let numbers = Lexplore::list_numbers_for(ElementType::Paragraph).unwrap();
        assert!(!numbers.is_empty());
        assert!(numbers.contains(&1));
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

    // Removed test for deleted API: test_must_methods

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

    // Removed test for deleted API: test_get_document_source_for

    // Removed test for deleted API: test_must_get_document_source_for

    // Removed test for deleted API: test_get_document_ast_for

    // Removed test for deleted API: test_must_get_document_ast_for

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

    // Removed test for deleted API: test_get_tokens_for

    // Removed test for deleted API: test_must_get_tokens_for

    // Removed test for deleted API: test_get_document_tokens_for

    // Removed test for deleted API: test_must_get_document_tokens_for

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

    // Removed test for deleted API: test_tokenize_with_parser_function

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

    // Removed test for deleted API: test_get_source_from_path

    // Removed test for deleted API: test_must_get_source_from_path

    // Removed test for deleted API: test_get_ast_from_path

    // Removed test for deleted API: test_must_get_ast_from_path

    // Removed test for deleted API: test_get_tokens_from_path

    // Removed test for deleted API: test_must_get_tokens_from_path

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
