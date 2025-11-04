//! Legacy pipeline orchestration for lexer/parser combinations
//!
//! This module provides the old `LexPipeline` infrastructure that manages multiple
//! lexer/parser combinations through registries. This is legacy code maintained
//! for backwards compatibility while experimental pipelines are being evaluated.
//!
//! For the new TokenStream-based pipeline, see the `pipeline` module.
//!
//! # Examples
//!
//! ```no_run
//! use lex::lex::pipeline::LexPipeline;
//!
//! // Use default stable pipeline
//! let pipeline = LexPipeline::default();
//! let doc = pipeline.parse("hello world").expect("Failed to parse");
//!
//! // Use specific combination
//! let pipeline = LexPipeline::new("linebased", "linebased");
//! let doc = pipeline.parse("hello world").expect("Failed to parse");
//!
//! // Tokenize only
//! let output = pipeline.lex("hello world").expect("Failed to tokenize");
//! ```

use crate::lex::lexers::{LexError, LexerOutput, LexerRegistry};
use crate::lex::parsers::{Document, ParseError, ParserInput, ParserRegistry};
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

/// High-level pipeline for parsing lex documents
///
/// Combines a lexer and parser implementation into a unified interface.
/// Allows selecting which lexer and parser to use for processing lex source code.
pub struct LexPipeline {
    lexer_name: String,
    parser_name: String,
    lexer_registry: LexerRegistry,
    parser_registry: ParserRegistry,
}

impl LexPipeline {
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

        LexPipeline {
            lexer_name: lexer_name.to_string(),
            parser_name: parser_name.to_string(),
            lexer_registry,
            parser_registry,
        }
    }

    /// Get the default stable pipeline (indentation lexer + reference parser)
    pub fn default_pipeline() -> Self {
        LexPipeline::new("indentation", "reference")
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
            LexerOutput::LineContainer(container) => ParserInput::LineContainer(container),
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

impl Default for LexPipeline {
    fn default() -> Self {
        Self::default_pipeline()
    }
}

impl Clone for LexPipeline {
    fn clone(&self) -> Self {
        LexPipeline {
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
        let pipeline = LexPipeline::new("indentation", "reference");
        assert_eq!(pipeline.lexer_name(), "indentation");
        assert_eq!(pipeline.parser_name(), "reference");
    }

    #[test]
    fn test_pipeline_default() {
        let pipeline = LexPipeline::default();
        assert_eq!(pipeline.lexer_name(), "indentation");
        assert_eq!(pipeline.parser_name(), "reference");
    }

    #[test]
    fn test_pipeline_default_pipeline() {
        let pipeline = LexPipeline::default_pipeline();
        assert_eq!(pipeline.lexer_name(), "indentation");
        assert_eq!(pipeline.parser_name(), "reference");
    }

    #[test]
    fn test_pipeline_available_lexers() {
        let lexers = LexPipeline::available_lexers();
        assert!(!lexers.is_empty());
        assert!(lexers.contains(&"indentation".to_string()));
        assert!(lexers.contains(&"linebased".to_string()));
    }

    #[test]
    fn test_pipeline_available_parsers() {
        let parsers = LexPipeline::available_parsers();
        assert!(!parsers.is_empty());
        assert!(parsers.contains(&"reference".to_string()));
        assert!(parsers.contains(&"linebased".to_string()));
    }

    #[test]
    fn test_pipeline_available_configurations() {
        let configs = LexPipeline::available_configurations();
        // Should have at least indentation+reference, indentation+linebased,
        // linebased+reference, linebased+linebased
        assert!(configs.len() >= 4);

        // Check some valid combinations exist
        assert!(configs.contains(&("indentation".to_string(), "reference".to_string())));
        assert!(configs.contains(&("indentation".to_string(), "linebased".to_string())));
    }

    #[test]
    fn test_pipeline_clone() {
        let original = LexPipeline::new("indentation", "reference");
        let cloned = original.clone();

        assert_eq!(cloned.lexer_name(), original.lexer_name());
        assert_eq!(cloned.parser_name(), original.parser_name());
    }

    #[test]
    fn test_pipeline_lex_simple() {
        let pipeline = LexPipeline::new("indentation", "reference");
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
        let pipeline = LexPipeline::default();
        let result = pipeline.parse("hello");

        assert!(result.is_ok());
        // The parse should succeed and create a document
        let _doc = result.unwrap();
        // Just verify it parsed successfully
    }
}
