//! File loading, parsing, and tokenization for Lex test harness
//!
//! This module provides the core loading infrastructure for the Lexplore test harness,
//! handling file discovery, reading, parsing, and tokenization.

use crate::lex::ast::elements::{Annotation, Definition, List, Paragraph, Session, Verbatim};
use crate::lex::ast::Document;
use crate::lex::lexing::Token;
use crate::lex::parsing::ParseError;
use crate::lex::pipeline::DocumentLoader;
use crate::lex::testing::lexplore::specfile_finder;
use std::fs;
use std::path::PathBuf;

// Re-export Parser from pipeline for backward compatibility
pub use crate::lex::pipeline::Parser;

// Re-export types from specfile_finder for public API
pub use specfile_finder::{DocumentType, ElementType};

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

impl From<specfile_finder::SpecFileError> for ElementSourceError {
    fn from(err: specfile_finder::SpecFileError) -> Self {
        match err {
            specfile_finder::SpecFileError::FileNotFound(msg) => {
                ElementSourceError::FileNotFound(msg)
            }
            specfile_finder::SpecFileError::IoError(msg) => ElementSourceError::IoError(msg),
            specfile_finder::SpecFileError::DuplicateNumber(msg) => {
                ElementSourceError::IoError(msg)
            }
        }
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

    /// Parse with the specified parser and return a Document
    pub fn parse_with(self, parser: Parser) -> Document {
        let path = self.get_path();
        let loader = DocumentLoader::new();
        loader
            .load_and_parse_with(&path, parser)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e))
    }

    /// Parse with the Reference parser (shorthand)
    pub fn parse(self) -> Document {
        self.parse_with(crate::lex::pipeline::Parser::Reference)
    }

    /// Tokenize with the specified parser and return tokens with their byte ranges
    pub fn tokenize_with(self, parser: Parser) -> Vec<(Token, std::ops::Range<usize>)> {
        let path = self.get_path();
        let loader = DocumentLoader::new();
        loader
            .load_and_tokenize_with(&path, parser)
            .unwrap_or_else(|e| panic!("Failed to tokenize {}: {}", path.display(), e))
    }

    /// Tokenize with the Reference parser (shorthand)
    pub fn tokenize(self) -> Vec<(Token, std::ops::Range<usize>)> {
        self.tokenize_with(crate::lex::pipeline::Parser::Reference)
    }
}

/// Loader for isolated element test files
///
/// This struct is specifically for loading single-element test files from
/// docs/specs/v1/elements/ and extracting the element directly as a typed AST node.
///
/// It orchestrates:
/// 1. Path resolution via specfile_finder
/// 2. File parsing via DocumentLoader
/// 3. Element extraction via Document::expect_*() methods
///
/// # Example
/// ```ignore
/// let element = Lexplore::get_element(ElementType::Paragraph, 3);
/// let paragraph = element.as_paragraph();
/// // Returns &Paragraph directly, panics if not found
/// ```
pub struct IsolatedElementLoader {
    doc: Document,
}

impl IsolatedElementLoader {
    /// Create a new isolated element loader by loading and parsing the element file
    fn load(element_type: ElementType, number: usize) -> Self {
        let path = specfile_finder::find_element_file(element_type, number)
            .unwrap_or_else(|e| panic!("Failed to find {:?} #{}: {}", element_type, number, e));

        let loader = DocumentLoader::new();
        let doc = loader
            .load_and_parse_with(&path, Parser::Reference)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e));

        Self { doc }
    }

    /// Get the loaded document
    pub fn document(&self) -> &Document {
        &self.doc
    }

    /// Extract element as a Paragraph (panics if not found)
    pub fn as_paragraph(&self) -> &Paragraph {
        self.doc.expect_paragraph()
    }

    /// Extract element as a List (panics if not found)
    pub fn as_list(&self) -> &List {
        self.doc.expect_list()
    }

    /// Extract element as a Session (panics if not found)
    pub fn as_session(&self) -> &Session {
        self.doc.expect_session()
    }

    /// Extract element as a Definition (panics if not found)
    pub fn as_definition(&self) -> &Definition {
        self.doc.expect_definition()
    }

    /// Extract element as an Annotation (panics if not found)
    pub fn as_annotation(&self) -> &Annotation {
        self.doc.expect_annotation()
    }

    /// Extract element as a Verbatim block (panics if not found)
    pub fn as_verbatim(&self) -> &Verbatim {
        self.doc.expect_verbatim()
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

// ============================================================================
// FLUENT API - Delegates to specfile_finder for file resolution
// ============================================================================

/// Interface for loading per-element test sources
pub struct Lexplore;

impl Lexplore {
    // ===== Fluent API - start a chain =====

    /// Start a fluent chain for loading and parsing an element
    pub fn load(element_type: ElementType, number: usize) -> ElementLoader {
        ElementLoader {
            source_type: SourceType::Element(element_type),
            number,
        }
    }

    /// Start a fluent chain for loading and parsing a document collection
    pub fn load_document(doc_type: DocumentType, number: usize) -> ElementLoader {
        ElementLoader {
            source_type: SourceType::Document(doc_type),
            number,
        }
    }

    /// Start a fluent chain for loading and parsing from an arbitrary file path
    pub fn from_path<P: Into<PathBuf>>(path: P) -> ElementLoader {
        ElementLoader {
            source_type: SourceType::Path(path.into()),
            number: 0, // Dummy value, not used for Path variant
        }
    }

    // ===== Isolated element loading (returns AST node directly) =====

    /// Load an isolated element file and return an element loader with the parsed AST
    ///
    /// This is for single-element test files in docs/specs/v1/elements/
    /// Use .as_paragraph(), .as_list(), etc. to extract the specific element type.
    ///
    /// # Example
    /// ```ignore
    /// let element = Lexplore::get_element(ElementType::Paragraph, 3);
    /// let paragraph = element.as_paragraph();
    /// ```
    pub fn get_element(element_type: ElementType, number: usize) -> IsolatedElementLoader {
        IsolatedElementLoader::load(element_type, number)
    }

    /// Convenience: Load a paragraph element file and return the paragraph directly
    pub fn get_paragraph(number: usize) -> IsolatedElementLoader {
        Self::get_element(ElementType::Paragraph, number)
    }

    /// Convenience: Load a list element file and return the list directly
    pub fn get_list(number: usize) -> IsolatedElementLoader {
        Self::get_element(ElementType::List, number)
    }

    /// Convenience: Load a session element file and return the session directly
    pub fn get_session(number: usize) -> IsolatedElementLoader {
        Self::get_element(ElementType::Session, number)
    }

    /// Convenience: Load a definition element file and return the definition directly
    pub fn get_definition(number: usize) -> IsolatedElementLoader {
        Self::get_element(ElementType::Definition, number)
    }

    /// Convenience: Load an annotation element file and return the annotation directly
    pub fn get_annotation(number: usize) -> IsolatedElementLoader {
        Self::get_element(ElementType::Annotation, number)
    }

    /// Convenience: Load a verbatim element file and return the verbatim block directly
    pub fn get_verbatim(number: usize) -> IsolatedElementLoader {
        Self::get_element(ElementType::Verbatim, number)
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

    // ===== Core resolution wrappers - delegate to specfile_finder =====

    /// Find the file for an element type and number
    fn find_file(element_type: ElementType, number: usize) -> Result<PathBuf, ElementSourceError> {
        Ok(specfile_finder::find_element_file(element_type, number)?)
    }

    /// Find the file for a document type and number
    fn find_document_file(
        doc_type: DocumentType,
        number: usize,
    ) -> Result<PathBuf, ElementSourceError> {
        Ok(specfile_finder::find_document_file(doc_type, number)?)
    }

    /// List all available numbers for a given element type
    pub fn list_numbers_for(element_type: ElementType) -> Result<Vec<usize>, ElementSourceError> {
        Ok(specfile_finder::list_element_numbers(element_type)?)
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
        let element = Lexplore::get_paragraph(1);
        let paragraph = element.as_paragraph();

        assert!(paragraph_text_starts_with(paragraph, "This is a simple"));
    }

    #[test]
    fn test_fluent_api_with_parser_selection() {
        // The new isolated element API always uses Reference parser
        // For explicit parser selection, use the full pipeline: Lexplore::paragraph(1).parse_with(Parser::X)
        let doc = Lexplore::paragraph(1).parse_with(Parser::Reference);
        let paragraph = doc.expect_paragraph();

        assert!(paragraph_text_starts_with(paragraph, "This is a simple"));
    }

    #[test]
    fn test_fluent_api_source_only() {
        let source = Lexplore::paragraph(1).source();
        assert!(source.contains("simple"));
    }

    #[test]
    fn test_fluent_api_list() {
        let element = Lexplore::get_list(1);
        let list = element.as_list();

        assert!(!list.items.is_empty());
    }

    #[test]
    fn test_fluent_api_session() {
        let element = Lexplore::get_session(1);
        let session = element.as_session();

        assert!(!session.label().is_empty());
    }

    #[test]
    fn test_fluent_api_definition() {
        let element = Lexplore::get_definition(1);
        let definition = element.as_definition();

        assert!(!definition.label().is_empty());
    }

    // Removed test for deleted API: test_must_methods

    // ===== Document Collection Tests =====

    #[test]
    fn test_benchmark_fluent_api() {
        let doc = Lexplore::benchmark(10).parse();

        assert!(!doc.root.children.is_empty());
    }

    #[test]
    fn test_trifecta_fluent_api() {
        let doc = Lexplore::trifecta(0).parse();

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
        let tokens = Lexplore::paragraph(1).tokenize();

        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_tokenize_with_parser() {
        let tokens = Lexplore::paragraph(1).tokenize_with(Parser::Reference);

        assert!(!tokens.is_empty());
        assert!(tokens.iter().any(|(t, _)| matches!(t, Token::Text(_))));
    }

    #[test]
    fn test_tokenize_list() {
        let tokens = Lexplore::list(1).tokenize();

        assert!(
            tokens.iter().any(|(t, _)| matches!(t, Token::Dash))
                || tokens.iter().any(|(t, _)| matches!(t, Token::Number(_)))
        );
    }

    #[test]
    fn test_tokenize_benchmark() {
        let tokens = Lexplore::benchmark(10).tokenize();

        assert!(!tokens.is_empty());
        assert!(tokens.len() > 10);
    }

    #[test]
    fn test_tokenize_trifecta() {
        let tokens = Lexplore::trifecta(0).tokenize();

        assert!(!tokens.is_empty());
        assert!(tokens.iter().any(|(t, _)| matches!(t, Token::Text(_))));
    }

    #[test]
    fn test_tokenize_linebased_parser() {
        let tokens = Lexplore::paragraph(1).tokenize_with(Parser::Linebased);

        assert!(!tokens.is_empty());
        assert!(tokens.iter().any(|(t, _)| matches!(t, Token::Text(_))));
    }

    // ===== Path-based Loading Tests =====

    #[test]
    fn test_from_path_parse() {
        let path = "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex";
        let doc = Lexplore::from_path(path).parse();

        let paragraph = doc.expect_paragraph();
        assert!(!paragraph.text().is_empty());
    }

    #[test]
    fn test_from_path_tokenize() {
        let path = "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex";
        let tokens = Lexplore::from_path(path).tokenize();

        assert!(!tokens.is_empty());
        assert!(tokens.iter().any(|(t, _)| matches!(t, Token::Text(_))));
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
        let doc = Lexplore::from_path(path).parse_with(Parser::Reference);

        let list = doc.expect_list();
        assert!(!list.items.is_empty());
    }

    #[test]
    fn test_from_path_tokenize_with_parser() {
        let path = "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex";
        let tokens = Lexplore::from_path(path).tokenize_with(Parser::Linebased);

        assert!(!tokens.is_empty());
        assert!(tokens.iter().any(|(t, _)| matches!(t, Token::Text(_))));
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
        let doc = Lexplore::from_path(path).parse();

        assert!(!doc.root.children.is_empty());
    }

    #[test]
    fn test_from_path_with_trifecta() {
        let path = "docs/specs/v1/trifecta/000-paragraphs.lex";
        let doc = Lexplore::from_path(path).parse();

        assert!(!doc.root.children.is_empty());
    }

    // ===== Detokenization Tests =====

    #[test]
    fn test_detokenize_paragraph() {
        let source = Lexplore::paragraph(1).source();
        let tokens = Lexplore::paragraph(1).tokenize();
        let token_only: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        let detokenized = crate::lex::formats::detokenize(&token_only);

        // Detokenized should match original source
        assert_eq!(detokenized, source);
    }

    #[test]
    fn test_detokenize_benchmark() {
        let tokens = Lexplore::benchmark(10).tokenize();
        let token_only: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        let detokenized = crate::lex::formats::detokenize(&token_only);

        // Verify detokenization produces non-empty output with expected content
        assert!(!detokenized.is_empty());
        assert!(detokenized.contains("Kitchensink Test Document"));
        assert!(detokenized.contains("1. Primary Session"));
        assert!(detokenized.contains("2. Second Root Session"));
    }

    #[test]
    fn test_detokenize_from_path() {
        let path = "docs/specs/v1/benchmark/010-kitchensink.lex";
        let tokens = Lexplore::from_path(path).tokenize();
        let token_only: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        let detokenized = crate::lex::formats::detokenize(&token_only);

        // Verify detokenization produces expected content
        assert!(!detokenized.is_empty());
        assert!(detokenized.contains("Kitchensink Test Document"));
        assert!(detokenized.contains("Root Definition:"));
        assert!(detokenized.contains("Nested Session"));
    }

    #[test]
    fn test_detokenize_with_semantic_tokens() {
        let source = Lexplore::session(1).source();
        let tokens = Lexplore::session(1).tokenize();
        let token_only: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        let detokenized = crate::lex::formats::detokenize(&token_only);

        // Detokenized should match original source (handles Indent/Dedent tokens)
        assert_eq!(detokenized, source);
    }

    #[test]
    fn test_detokenize_trifecta() {
        let source = Lexplore::trifecta(0).source();
        let tokens = Lexplore::trifecta(0).tokenize();
        let token_only: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        let detokenized = crate::lex::formats::detokenize(&token_only);

        // Detokenized should match original source
        assert_eq!(detokenized, source);
    }

    #[test]
    fn test_detokenize_fluent_api() {
        // Demonstrate fluent API usage
        let tokens = Lexplore::from_path("docs/specs/v1/benchmark/010-kitchensink.lex").tokenize();
        let token_only: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        let detokenized = crate::lex::formats::detokenize(&token_only);

        assert!(detokenized.contains("Kitchensink"));
    }

    // ===== Isolated Element Loading Tests =====

    #[test]
    fn test_get_element_paragraph() {
        let element = Lexplore::get_element(ElementType::Paragraph, 1);
        let paragraph = element.as_paragraph();

        assert!(paragraph_text_starts_with(paragraph, "This is a simple"));
    }

    #[test]
    fn test_get_paragraph_shortcut() {
        let element = Lexplore::get_paragraph(1);
        let paragraph = element.as_paragraph();

        assert!(paragraph_text_starts_with(paragraph, "This is a simple"));
    }

    #[test]
    fn test_get_list_shortcut() {
        let element = Lexplore::get_list(1);
        let list = element.as_list();

        assert!(!list.items.is_empty());
    }

    #[test]
    fn test_get_session_shortcut() {
        let element = Lexplore::get_session(1);
        let session = element.as_session();

        assert!(!session.label().is_empty());
    }

    #[test]
    fn test_get_definition_shortcut() {
        let element = Lexplore::get_definition(1);
        let definition = element.as_definition();

        assert!(!definition.label().is_empty());
    }

    #[test]
    fn test_get_annotation_shortcut() {
        let element = Lexplore::get_annotation(1);
        let _annotation = element.as_annotation();

        // Annotations should have at least a marker
        assert!(element.document().iter_annotations_recursive().count() > 0);
    }

    #[test]
    fn test_get_verbatim_shortcut() {
        let element = Lexplore::get_verbatim(1);
        let _verbatim = element.as_verbatim();

        // Just verify it doesn't panic - verbatim blocks can be empty or have content
    }

    #[test]
    fn test_isolated_element_loader_document_access() {
        let element = Lexplore::get_paragraph(1);
        let doc = element.document();

        // Should be able to access the underlying document
        assert!(doc.first_paragraph().is_some());
    }
}
