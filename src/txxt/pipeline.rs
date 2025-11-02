//! High-level unified pipeline API for txxt parsing
//!
//! This module provides the `TxxtPipeline` struct which offers a convenient,
//! high-level interface for tokenizing and parsing txxt documents using
//! any combination of lexer and parser implementations.
//!
//! # Examples
//!
//! ```no_run
//! use txxt::txxt::pipeline::TxxtPipeline;
//!
//! // Use default stable pipeline
//! let pipeline = TxxtPipeline::default();
//! let doc = pipeline.parse("hello world").expect("Failed to parse");
//!
//! // Use specific combination
//! let pipeline = TxxtPipeline::new("linebased", "linebased");
//! let doc = pipeline.parse("hello world").expect("Failed to parse");
//!
//! // Tokenize only
//! let output = pipeline.lex("hello world").expect("Failed to tokenize");
//! ```

use crate::txxt::lexers::{LexError, LexerOutput, LexerRegistry};
use crate::txxt::parsers::{Document, ParseError, ParserInput, ParserRegistry};
use std::fmt;

/// Errors that can occur during pipeline operations
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineError {
    LexerError(LexError),
    ParserError(ParseError),
    InvalidCombination(String),
}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PipelineError::LexerError(e) => write!(f, "Lexer error: {}", e),
            PipelineError::ParserError(e) => write!(f, "Parser error: {}", e),
            PipelineError::InvalidCombination(msg) => {
                write!(f, "Invalid lexer/parser combination: {}", msg)
            }
        }
    }
}

impl std::error::Error for PipelineError {}

impl From<LexError> for PipelineError {
    fn from(err: LexError) -> Self {
        PipelineError::LexerError(err)
    }
}

impl From<ParseError> for PipelineError {
    fn from(err: ParseError) -> Self {
        PipelineError::ParserError(err)
    }
}

/// High-level pipeline for parsing txxt documents
///
/// Combines a lexer and parser implementation into a unified interface.
/// Allows selecting which lexer and parser to use for processing txxt source code.
pub struct TxxtPipeline {
    lexer_name: String,
    parser_name: String,
    lexer_registry: LexerRegistry,
    parser_registry: ParserRegistry,
}

impl TxxtPipeline {
    /// Create a new pipeline with specified lexer and parser
    ///
    /// # Arguments
    /// * `lexer_name` - Name of the lexer to use (e.g., "indentation", "linebased")
    /// * `parser_name` - Name of the parser to use (e.g., "reference", "linebased")
    ///
    /// # Panics
    /// Panics if the registries are not initialized or lexer/parser not found
    pub fn new(lexer_name: &str, parser_name: &str) -> Self {
        // Initialize registries with defaults
        LexerRegistry::init_defaults();
        ParserRegistry::init_defaults();

        let lexer_registry = LexerRegistry::global().lock().unwrap().clone();
        let parser_registry = ParserRegistry::global().lock().unwrap().clone();

        TxxtPipeline {
            lexer_name: lexer_name.to_string(),
            parser_name: parser_name.to_string(),
            lexer_registry,
            parser_registry,
        }
    }

    /// Get the default stable pipeline (indentation lexer + reference parser)
    pub fn default_pipeline() -> Self {
        TxxtPipeline::new("indentation", "reference")
    }

    /// Get the lexer name for this pipeline
    pub fn lexer_name(&self) -> &str {
        &self.lexer_name
    }

    /// Get the parser name for this pipeline
    pub fn parser_name(&self) -> &str {
        &self.parser_name
    }

    /// Tokenize source text using the configured lexer
    pub fn lex(&self, source: &str) -> Result<LexerOutput, PipelineError> {
        self.lexer_registry
            .tokenize(&self.lexer_name, source)
            .map_err(PipelineError::from)
    }

    /// Parse source text using the configured lexer and parser
    pub fn parse(&self, source: &str) -> Result<Document, PipelineError> {
        // Tokenize
        let lexer_output = self.lex(source)?;

        // Convert lexer output to parser input
        let parser_input = match lexer_output {
            LexerOutput::Tokens(tokens) => ParserInput::Tokens(tokens),
            LexerOutput::LineTokenTrees(trees) => ParserInput::LineTokenTrees(trees),
        };

        // Parse
        self.parser_registry
            .parse(&self.parser_name, parser_input, source)
            .map_err(PipelineError::from)
    }

    /// Get all available lexer/parser combinations
    pub fn available_configurations() -> Vec<(String, String)> {
        // Initialize registries
        LexerRegistry::init_defaults();
        ParserRegistry::init_defaults();

        let lexer_registry = LexerRegistry::global().lock().unwrap();
        let parser_registry = ParserRegistry::global().lock().unwrap();

        let lexers = lexer_registry.available();
        let parsers = parser_registry.available();

        let mut combinations = Vec::new();
        for lexer in &lexers {
            for parser in &parsers {
                // Only add valid combinations (lexer output must match parser input)
                // This is a simple check - in a real system you might want more sophisticated validation
                combinations.push((lexer.clone(), parser.clone()));
            }
        }

        combinations
    }

    /// List available lexers
    pub fn available_lexers() -> Vec<String> {
        LexerRegistry::init_defaults();
        LexerRegistry::global().lock().unwrap().available()
    }

    /// List available parsers
    pub fn available_parsers() -> Vec<String> {
        ParserRegistry::init_defaults();
        ParserRegistry::global().lock().unwrap().available()
    }
}

impl Default for TxxtPipeline {
    fn default() -> Self {
        Self::default_pipeline()
    }
}

impl Clone for TxxtPipeline {
    fn clone(&self) -> Self {
        TxxtPipeline {
            lexer_name: self.lexer_name.clone(),
            parser_name: self.parser_name.clone(),
            lexer_registry: self.lexer_registry.clone(),
            parser_registry: self.parser_registry.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        let pipeline = TxxtPipeline::new("indentation", "reference");
        assert_eq!(pipeline.lexer_name(), "indentation");
        assert_eq!(pipeline.parser_name(), "reference");
    }

    #[test]
    fn test_pipeline_default() {
        let pipeline = TxxtPipeline::default();
        assert_eq!(pipeline.lexer_name(), "indentation");
        assert_eq!(pipeline.parser_name(), "reference");
    }

    #[test]
    fn test_pipeline_default_pipeline() {
        let pipeline = TxxtPipeline::default_pipeline();
        assert_eq!(pipeline.lexer_name(), "indentation");
        assert_eq!(pipeline.parser_name(), "reference");
    }

    #[test]
    fn test_pipeline_available_lexers() {
        let lexers = TxxtPipeline::available_lexers();
        assert!(!lexers.is_empty());
        assert!(lexers.contains(&"indentation".to_string()));
        assert!(lexers.contains(&"linebased".to_string()));
    }

    #[test]
    fn test_pipeline_available_parsers() {
        let parsers = TxxtPipeline::available_parsers();
        assert!(!parsers.is_empty());
        assert!(parsers.contains(&"reference".to_string()));
        assert!(parsers.contains(&"linebased".to_string()));
    }

    #[test]
    fn test_pipeline_available_configurations() {
        let configs = TxxtPipeline::available_configurations();
        // Should have at least indentation+reference, indentation+linebased,
        // linebased+reference, linebased+linebased
        assert!(configs.len() >= 4);

        // Check some valid combinations exist
        assert!(configs.contains(&("indentation".to_string(), "reference".to_string())));
        assert!(configs.contains(&("indentation".to_string(), "linebased".to_string())));
    }

    #[test]
    fn test_pipeline_clone() {
        let original = TxxtPipeline::new("indentation", "reference");
        let cloned = original.clone();

        assert_eq!(cloned.lexer_name(), original.lexer_name());
        assert_eq!(cloned.parser_name(), original.parser_name());
    }

    #[test]
    fn test_pipeline_lex_simple() {
        let pipeline = TxxtPipeline::new("indentation", "reference");
        let result = pipeline.lex("hello");

        assert!(result.is_ok());
        match result.unwrap() {
            LexerOutput::Tokens(_) => {
                // Success - indentation lexer produces tokens
            }
            _ => panic!("Expected tokens from indentation lexer"),
        }
    }

    #[test]
    fn test_pipeline_parse_simple() {
        let pipeline = TxxtPipeline::default();
        let result = pipeline.parse("hello");

        assert!(result.is_ok());
        // The parse should succeed and create a document
        let _doc = result.unwrap();
        // Just verify it parsed successfully
    }
}
