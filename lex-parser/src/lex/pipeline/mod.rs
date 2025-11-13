//! Simplified processing pipeline for the lex format
//!
//! This module provides a streamlined pipeline for transforming lex source code into AST documents.
//! The pipeline has been simplified from a complex config-driven system to a straightforward
//! sequential transformation flow.
//!
//! # Architecture
//!
//! The pipeline executes three sequential transformations:
//!
//! 1. **Base Tokenization** - Raw lexical analysis using logos
//! 2. **Semantic Indentation** - Transform Indentation tokens into Indent/Dedent pairs
//! 3. **Parsing** - Build AST using the linebased parser
//!
//! # Usage
//!
//! ## Direct Pipeline Usage
//!
//! Use [`Pipeline::run()`] when you need just the AST document:
//!
//! ```rust
//! use lex::lex::pipeline::Pipeline;
//!
//! let source = "This is a paragraph.\n\n1. Session Title\n    Content here.";
//! let pipeline = Pipeline::new();
//! let doc = pipeline.run(source).expect("Parse failed");
//! ```
//!
//! ## Using PipelineExecutor
//!
//! Use [`PipelineExecutor`] when you need format serialization (e.g., for CLI tools):
//!
//! ```rust
//! use lex::lex::pipeline::PipelineExecutor;
//!
//! let source = "This is a paragraph.";
//! let executor = PipelineExecutor::new();
//!
//! // Get AST document
//! let result = executor.execute(source).expect("Parse failed");
//!
//! // Or serialize to a specific format
//! let tag_output = executor.execute_and_serialize(source, "ast-tag")
//!     .expect("Serialization failed");
//! ```
//!
//! ## For Most Use Cases
//!
//! The convenience function [`crate::lex::parsing::parse_document()`] is the recommended
//! entry point for typical parsing needs:
//!
//! ```rust
//! use lex::lex::parsing::parse_document;
//!
//! let doc = parse_document("Hello world\n").expect("Parse failed");
//! ```

use crate::lex::parsing::Document;

pub mod executor;
pub use executor::{ExecutionError, ExecutionOutput, PipelineExecutor};

/// The core processing pipeline.
///
/// Transforms lex source code through tokenization, lexical transformations,
/// and parsing to produce an AST document.
///
/// This is a simple struct that encodes the complete transformation sequence.
/// For most use cases, prefer the convenience function [`crate::lex::parsing::parse_document()`].
pub struct Pipeline;

impl Pipeline {
    pub fn new() -> Self {
        Self
    }

    /// Execute the complete pipeline transformation on source text.
    ///
    /// # Transformation Sequence
    ///
    /// 1. Ensure source ends with newline (required for paragraph parsing at EOF)
    /// 2. Base tokenization - raw lexical tokens from logos
    /// 3. Semantic indentation - convert Indentation tokens to Indent/Dedent
    /// 4. Parse - build AST using linebased parser
    ///
    /// # Example
    ///
    /// ```rust
    /// use lex::lex::pipeline::Pipeline;
    ///
    /// let pipeline = Pipeline::new();
    /// let doc = pipeline.run("Hello world\n").expect("Parse failed");
    /// assert_eq!(doc.root.children.len(), 1);
    /// ```
    pub fn run(&self, source: &str) -> Result<Document, String> {
        let source_with_newline = crate::lex::lexing::ensure_source_ends_with_newline(source);
        let token_stream = crate::lex::lexing::base_tokenization::tokenize(&source_with_newline);
        let tokens = crate::lex::lexing::lex(token_stream);
        crate::lex::parsing::engine::parse_from_flat_tokens(tokens, source)
            .map_err(|err| format!("Parsing failed: {}", err))
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}
