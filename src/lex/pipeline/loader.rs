//! Document loading and processing API
//!
//! This module provides the primary entry point for all Lex pipeline operations.
//! The `DocumentLoader` API handles both string-based and file-based processing,
//! supporting all pipeline configurations, parser selection, and output formats.
//!
//! # Architecture
//!
//! - String-based methods are the core functionality (process source text)
//! - File-based methods are thin wrappers (read file, then call string method)
//! - All operations delegate to PipelineExecutor for actual processing
//!
//! # Examples
//!
//! ```rust,ignore
//! use lex::lex::pipeline::{DocumentLoader, Parser};
//!
//! let loader = DocumentLoader::new();
//!
//! // Parse a string
//! let doc = loader.parse("Hello world\n")?;
//!
//! // Parse a file
//! let doc = loader.load_and_parse("path/to/file.lex")?;
//!
//! // Parse with specific parser
//! let doc = loader.parse_with("Hello world\n", Parser::Linebased)?;
//!
//! // Tokenize
//! let tokens = loader.tokenize("Hello world\n")?;
//!
//! // Execute any config
//! let output = loader.execute("default", "Hello world\n")?;
//! ```

use crate::lex::lexing::Token;
use crate::lex::parsing::Document;
use crate::lex::pipeline::{ExecutionError, ExecutionOutput, PipelineExecutor};
use std::fs;
use std::path::Path;

/// Parser implementation to use for parsing and tokenization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parser {
    /// Linebased parser (grammar-based, current default)
    Linebased,
}

impl Parser {
    /// Get the pipeline config name for this parser (AST output)
    pub fn config_name(&self) -> &'static str {
        match self {
            Parser::Linebased => "linebased",
        }
    }

    /// Get the pipeline config name for tokenization (Token output)
    pub fn token_config_name(&self) -> &'static str {
        match self {
            Parser::Linebased => "tokens-indentation",
        }
    }
}

/// Primary API for document loading and processing
///
/// DocumentLoader is the main entry point for all Lex pipeline operations.
/// It provides both string-based and file-based processing methods,
/// supporting all pipeline configurations and parser selections.
///
/// # Design
///
/// - String-based methods (`parse`, `tokenize`, etc.) are the core operations
/// - File-based methods (`load_and_parse`, etc.) read files then delegate to string methods
/// - All processing delegates to `PipelineExecutor` for actual execution
///
/// # Usage
///
/// ```rust,ignore
/// let loader = DocumentLoader::new();
///
/// // Process strings
/// let doc = loader.parse("Hello\n")?;
/// let tokens = loader.tokenize("Hello\n")?;
///
/// // Process files
/// let doc = loader.load_and_parse("file.lex")?;
/// let tokens = loader.load_and_tokenize("file.lex")?;
///
/// // Select parser
/// let doc = loader.parse_with("Hello\n", Parser::Linebased)?;
/// ```
pub struct DocumentLoader {
    executor: PipelineExecutor,
}

impl DocumentLoader {
    /// Create a new DocumentLoader with default pipeline configuration
    pub fn new() -> Self {
        Self {
            executor: PipelineExecutor::new(),
        }
    }

    /// Create a DocumentLoader with a custom PipelineExecutor
    pub fn with_executor(executor: PipelineExecutor) -> Self {
        Self { executor }
    }

    // ===== STRING-BASED PROCESSING (core methods) =====

    /// Execute a named pipeline configuration on source text
    ///
    /// This is the most flexible method - you can run any registered pipeline
    /// configuration by name (e.g., "default", "linebased", "tokens-indentation").
    ///
    /// # Arguments
    ///
    /// * `config_name` - Name of the pipeline configuration to execute
    /// * `source` - Source text to process
    ///
    /// # Returns
    ///
    /// `ExecutionOutput::Document` or `ExecutionOutput::Tokens` depending on the config
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let loader = DocumentLoader::new();
    /// let output = loader.execute("default", "Hello world\n")?;
    /// ```
    pub fn execute(
        &self,
        config_name: &str,
        source: &str,
    ) -> Result<ExecutionOutput, ExecutionError> {
        self.executor.execute(config_name, source)
    }

    /// Parse source text into a Document using the default parser
    ///
    /// This is a convenience method that uses the linebased parser (current default).
    /// For other analyzers, use `parse_with()` when they become available.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let loader = DocumentLoader::new();
    /// let doc = loader.parse("Hello world\n")?;
    /// ```
    pub fn parse(&self, source: &str) -> Result<Document, ExecutionError> {
        self.parse_with(source, Parser::Linebased)
    }

    /// Parse source text with a specific parser
    ///
    /// # Arguments
    ///
    /// * `source` - Source text to parse
    /// * `parser` - Which parser implementation to use
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let loader = DocumentLoader::new();
    /// let doc = loader.parse_with("Hello\n", Parser::Linebased)?;
    /// ```
    pub fn parse_with(&self, source: &str, parser: Parser) -> Result<Document, ExecutionError> {
        let output = self.executor.execute(parser.config_name(), source)?;
        match output {
            ExecutionOutput::Document(doc) => Ok(doc),
            ExecutionOutput::Tokens(_) => Err(ExecutionError::ParsingFailed(
                "Expected Document output but got Tokens".to_string(),
            )),
            ExecutionOutput::Serialized(_) => Err(ExecutionError::ParsingFailed(
                "Expected Document output but got Serialized".to_string(),
            )),
        }
    }

    /// Tokenize source text using the default tokenizer
    ///
    /// Returns a flat list of tokens with their byte ranges.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let loader = DocumentLoader::new();
    /// let tokens = loader.tokenize("Hello world\n")?;
    /// ```
    pub fn tokenize(
        &self,
        source: &str,
    ) -> Result<Vec<(Token, std::ops::Range<usize>)>, ExecutionError> {
        self.tokenize_with(source, Parser::Linebased)
    }

    /// Tokenize source text with a specific parser's tokenizer
    ///
    /// Different parsers may use different tokenization strategies.
    ///
    /// # Arguments
    ///
    /// * `source` - Source text to tokenize
    /// * `parser` - Which parser's tokenizer to use
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let loader = DocumentLoader::new();
    /// let tokens = loader.tokenize_with("Hello\n", Parser::Linebased)?;
    /// ```
    pub fn tokenize_with(
        &self,
        source: &str,
        parser: Parser,
    ) -> Result<Vec<(Token, std::ops::Range<usize>)>, ExecutionError> {
        let output = self.executor.execute(parser.token_config_name(), source)?;
        match output {
            ExecutionOutput::Tokens(stream) => Ok(stream.unroll()),
            ExecutionOutput::Document(_) => Err(ExecutionError::TransformationFailed(
                "Expected Tokens output but got Document".to_string(),
            )),
            ExecutionOutput::Serialized(_) => Err(ExecutionError::TransformationFailed(
                "Expected Tokens output but got Serialized".to_string(),
            )),
        }
    }

    // ===== FILE-BASED PROCESSING (thin wrappers over string methods) =====

    /// Load source text from a file
    ///
    /// This is a low-level method that just reads the file content.
    /// For processing, use `load_and_parse()` or `load_and_tokenize()`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let loader = DocumentLoader::new();
    /// let source = loader.load_source("path/to/file.lex")?;
    /// ```
    pub fn load_source<P: AsRef<Path>>(&self, path: P) -> Result<String, ExecutionError> {
        fs::read_to_string(path.as_ref())
            .map_err(|e| ExecutionError::TransformationFailed(e.to_string()))
    }

    /// Load a file and execute a pipeline configuration
    ///
    /// This combines file loading with pipeline execution.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to load
    /// * `config_name` - Name of the pipeline configuration to execute
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let loader = DocumentLoader::new();
    /// let output = loader.load_and_execute("file.lex", "default")?;
    /// ```
    pub fn load_and_execute<P: AsRef<Path>>(
        &self,
        path: P,
        config_name: &str,
    ) -> Result<ExecutionOutput, ExecutionError> {
        let source = self.load_source(path)?;
        self.execute(config_name, &source)
    }

    /// Load a file and parse it into a Document using the default parser
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let loader = DocumentLoader::new();
    /// let doc = loader.load_and_parse("path/to/file.lex")?;
    /// ```
    pub fn load_and_parse<P: AsRef<Path>>(&self, path: P) -> Result<Document, ExecutionError> {
        let source = self.load_source(path)?;
        self.parse(&source)
    }

    /// Load a file and parse it with a specific parser
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to load
    /// * `parser` - Which parser implementation to use
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let loader = DocumentLoader::new();
    /// let doc = loader.load_and_parse_with("file.lex", Parser::Linebased)?;
    /// ```
    pub fn load_and_parse_with<P: AsRef<Path>>(
        &self,
        path: P,
        parser: Parser,
    ) -> Result<Document, ExecutionError> {
        let source = self.load_source(path)?;
        self.parse_with(&source, parser)
    }

    /// Load a file and tokenize it using the default tokenizer
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let loader = DocumentLoader::new();
    /// let tokens = loader.load_and_tokenize("path/to/file.lex")?;
    /// ```
    pub fn load_and_tokenize<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<Vec<(Token, std::ops::Range<usize>)>, ExecutionError> {
        let source = self.load_source(path)?;
        self.tokenize(&source)
    }

    /// Load a file and tokenize it with a specific parser's tokenizer
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to load
    /// * `parser` - Which parser's tokenizer to use
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let loader = DocumentLoader::new();
    /// let tokens = loader.load_and_tokenize_with("file.lex", Parser::Linebased)?;
    /// ```
    pub fn load_and_tokenize_with<P: AsRef<Path>>(
        &self,
        path: P,
        parser: Parser,
    ) -> Result<Vec<(Token, std::ops::Range<usize>)>, ExecutionError> {
        let source = self.load_source(path)?;
        self.tokenize_with(&source, parser)
    }

    // ===== SERIALIZATION (convert to output formats) =====

    /// Convert source text to a specific format
    ///
    /// This is a convenience method for serialization. It runs the full pipeline
    /// (tokenize → parse → serialize) and returns the formatted string.
    ///
    /// # Arguments
    ///
    /// * `source` - Source text to convert
    /// * `format` - Output format name (e.g., "tag", "treeviz", "markdown")
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let loader = DocumentLoader::new();
    /// let tag_output = loader.convert("Hello\n", "tag")?;
    /// let treeviz = loader.convert("Hello\n", "treeviz")?;
    /// ```
    pub fn convert(&self, source: &str, format: &str) -> Result<String, ExecutionError> {
        let config_name = format!("lex-to-{}", format);
        let output = self.execute(&config_name, source)?;
        match output {
            ExecutionOutput::Serialized(s) => Ok(s),
            _ => Err(ExecutionError::TransformationFailed(format!(
                "Config '{}' did not produce serialized output",
                config_name
            ))),
        }
    }

    /// Load a file and convert to a specific format
    ///
    /// Combines file loading with format conversion.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to load
    /// * `format` - Output format name (e.g., "tag", "treeviz", "markdown")
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let loader = DocumentLoader::new();
    /// let tag_output = loader.load_and_convert("file.lex", "tag")?;
    /// ```
    pub fn load_and_convert<P: AsRef<Path>>(
        &self,
        path: P,
        format: &str,
    ) -> Result<String, ExecutionError> {
        let source = self.load_source(path)?;
        self.convert(&source, format)
    }

    /// Convert source text with a specific parser and format
    ///
    /// Allows choosing which parser to use before serialization.
    ///
    /// # Arguments
    ///
    /// * `source` - Source text to convert
    /// * `parser` - Which parser to use
    /// * `format` - Output format name (e.g., "tag", "treeviz")
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let loader = DocumentLoader::new();
    /// let output = loader.convert_with("Hello\n", Parser::Linebased, "tag")?;
    /// ```
    pub fn convert_with(
        &self,
        source: &str,
        _parser: Parser,
        format: &str,
    ) -> Result<String, ExecutionError> {
        let config_name = format!("lex-to-{}", format);
        let output = self.execute(&config_name, source)?;
        match output {
            ExecutionOutput::Serialized(s) => Ok(s),
            _ => Err(ExecutionError::TransformationFailed(format!(
                "Config '{}' did not produce serialized output",
                config_name
            ))),
        }
    }

    /// Load a file and convert with a specific parser and format
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to load
    /// * `parser` - Which parser to use
    /// * `format` - Output format name (e.g., "tag", "treeviz")
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let loader = DocumentLoader::new();
    /// let output = loader.load_and_convert_with("file.lex", Parser::Linebased, "treeviz")?;
    /// ```
    pub fn load_and_convert_with<P: AsRef<Path>>(
        &self,
        path: P,
        parser: Parser,
        format: &str,
    ) -> Result<String, ExecutionError> {
        let source = self.load_source(path)?;
        self.convert_with(&source, parser, format)
    }

    /// Get access to the underlying PipelineExecutor
    ///
    /// This is useful for advanced use cases that need direct executor access.
    pub fn executor(&self) -> &PipelineExecutor {
        &self.executor
    }
}

impl Default for DocumentLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loader_creation() {
        let loader = DocumentLoader::new();
        assert!(!loader.executor().list_configs().is_empty());
    }

    #[test]
    fn test_loader_default() {
        let loader = DocumentLoader::default();
        assert!(!loader.executor().list_configs().is_empty());
    }

    #[test]
    fn test_parse_simple_source() {
        let loader = DocumentLoader::new();
        let doc = loader.parse("Hello world\n");

        assert!(doc.is_ok());
        let doc = doc.unwrap();
        assert!(!doc.root.children.is_empty());
    }

    #[test]
    fn test_parse_with_linebased_parser() {
        let loader = DocumentLoader::new();
        let doc = loader.parse_with("Hello:\n    World\n", Parser::Linebased);

        assert!(doc.is_ok());
        let doc = doc.unwrap();
        assert!(!doc.root.children.is_empty());
    }

    #[test]
    fn test_tokenize_simple_source() {
        let loader = DocumentLoader::new();
        let tokens = loader.tokenize("Hello world");

        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_tokenize_with_linebased() {
        let loader = DocumentLoader::new();
        let tokens = loader.tokenize_with("Hello world", Parser::Linebased);

        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_execute_default_config() {
        let loader = DocumentLoader::new();
        let result = loader.execute("default", "Hello world\n");

        assert!(result.is_ok());
        match result.unwrap() {
            ExecutionOutput::Document(doc) => {
                assert!(!doc.root.children.is_empty());
            }
            _ => panic!("Expected Document output"),
        }
    }

    #[test]
    fn test_execute_tokens_config() {
        let loader = DocumentLoader::new();
        let result = loader.execute("tokens-indentation", "Hello world");

        assert!(result.is_ok());
        match result.unwrap() {
            ExecutionOutput::Tokens(stream) => {
                let tokens = stream.unroll();
                assert!(!tokens.is_empty());
            }
            _ => panic!("Expected Tokens output"),
        }
    }

    #[test]
    fn test_execute_linebased_config() {
        let loader = DocumentLoader::new();
        let result = loader.execute("linebased", "Hello:\n    World\n");

        assert!(result.is_ok());
        match result.unwrap() {
            ExecutionOutput::Document(doc) => {
                assert!(!doc.root.children.is_empty());
            }
            _ => panic!("Expected Document output"),
        }
    }

    #[test]
    fn test_execute_nonexistent_config() {
        let loader = DocumentLoader::new();
        let result = loader.execute("nonexistent", "Hello");

        assert!(result.is_err());
    }

    #[test]
    fn test_load_and_parse_file() {
        let loader = DocumentLoader::new();
        let result =
            loader.load_and_parse("docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex");

        assert!(result.is_ok());
        let doc = result.unwrap();
        assert!(!doc.root.children.is_empty());
    }

    #[test]
    fn test_load_and_parse_with_parser() {
        let loader = DocumentLoader::new();
        let result = loader.load_and_parse_with(
            "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex",
            Parser::Linebased,
        );

        assert!(result.is_ok());
        let doc = result.unwrap();
        assert!(!doc.root.children.is_empty());
    }

    #[test]
    fn test_load_and_tokenize_file() {
        let loader = DocumentLoader::new();
        let result = loader
            .load_and_tokenize("docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex");

        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_load_and_tokenize_with_parser() {
        let loader = DocumentLoader::new();
        let result = loader.load_and_tokenize_with(
            "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex",
            Parser::Linebased,
        );

        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_load_source_file() {
        let loader = DocumentLoader::new();
        let result =
            loader.load_source("docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex");

        assert!(result.is_ok());
        let source = result.unwrap();
        assert!(!source.is_empty());
    }

    #[test]
    fn test_load_source_nonexistent_file() {
        let loader = DocumentLoader::new();
        let result = loader.load_source("nonexistent-file.lex");

        assert!(result.is_err());
    }

    #[test]
    fn test_load_and_execute_default() {
        let loader = DocumentLoader::new();
        let result = loader.load_and_execute(
            "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex",
            "default",
        );

        assert!(result.is_ok());
        match result.unwrap() {
            ExecutionOutput::Document(doc) => {
                assert!(!doc.root.children.is_empty());
            }
            _ => panic!("Expected Document output"),
        }
    }

    #[test]
    fn test_load_and_execute_tokens() {
        let loader = DocumentLoader::new();
        let result = loader.load_and_execute(
            "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex",
            "tokens-indentation",
        );

        assert!(result.is_ok());
        match result.unwrap() {
            ExecutionOutput::Tokens(stream) => {
                let tokens = stream.unroll();
                assert!(!tokens.is_empty());
            }
            _ => panic!("Expected Tokens output"),
        }
    }

    #[test]
    fn test_parser_config_names() {
        assert_eq!(Parser::Linebased.config_name(), "linebased");
        assert_eq!(Parser::Linebased.token_config_name(), "tokens-indentation");
    }

    // ===== Serialization tests =====

    #[test]
    fn test_convert_to_tag() {
        let loader = DocumentLoader::new();
        let result = loader.convert("Hello world\n", "tag");

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("<paragraph>"));
        assert!(output.contains("Hello world"));
    }

    #[test]
    fn test_convert_to_treeviz() {
        let loader = DocumentLoader::new();
        let result = loader.convert("Hello world\n", "treeviz");

        assert!(result.is_ok());
        let output = result.unwrap();
        // Treeviz format contains node type information
        assert!(!output.is_empty());
    }

    #[test]
    fn test_convert_with_parser() {
        let loader = DocumentLoader::new();
        let result = loader.convert_with("Hello world\n", Parser::Linebased, "tag");

        assert!(result.is_ok());
        let output = result.unwrap();
        // Linebased parser should also produce output
        assert!(!output.is_empty());
        assert!(output.contains("<document>"));
    }

    #[test]
    fn test_load_and_convert() {
        let loader = DocumentLoader::new();
        let result = loader.load_and_convert(
            "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex",
            "tag",
        );

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("<paragraph>"));
    }

    #[test]
    fn test_load_and_convert_with_parser() {
        let loader = DocumentLoader::new();
        let result = loader.load_and_convert_with(
            "docs/specs/v1/elements/paragraph/paragraph-01-flat-oneline.lex",
            Parser::Linebased,
            "treeviz",
        );

        assert!(result.is_ok());
        let output = result.unwrap();
        // Treeviz format should produce output
        assert!(!output.is_empty());
    }

    #[test]
    fn test_execute_serialized_config() {
        let loader = DocumentLoader::new();
        let result = loader.execute("lex-to-tag", "Hello world\n");

        assert!(result.is_ok());
        match result.unwrap() {
            crate::lex::pipeline::ExecutionOutput::Serialized(output) => {
                assert!(output.contains("<paragraph>"));
            }
            _ => panic!("Expected Serialized output"),
        }
    }
}
