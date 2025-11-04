//! Low-level TokenStream transformation pipeline builder
//!
//! This module provides the `Pipeline` builder that chains StreamMapper transformations
//! together. This is the low-level interface for building TokenStream transformation
//! pipelines (NormalizeWhitespace, SemanticIndentation, ToLineTokens, etc.).
//!
//! For high-level lexer/parser orchestration (selecting lexer and parser combinations),
//! see the `orchestration` module which provides the `LexPipeline` API.
//!
//! # Design
//!
//! The pipeline:
//! 1. Calls base tokenization to get raw `Vec<(Token, Range)>` pairs
//! 2. Converts to `TokenStream::Flat`
//! 3. Applies each transformation in sequence via StreamMapper trait
//! 4. Returns final `TokenStream` (can be Flat or Tree depending on transformations)
//!
//! # Architecture
//!
//! This is the **internal transformation infrastructure**. It handles the low-level
//! token transformations that happen after base tokenization and before parsing.
//! For selecting which lexer/parser to use, see `LexPipeline` in the orchestration module.
//!
//! # Examples
//!
//! ```ignore
//! use lex::lex::pipeline::Pipeline;
//!
//! let mut pipeline = Pipeline::new()
//!     .add_transformation(NormalizeWhitespaceMapper::new())
//!     .add_transformation(SemanticIndentationMapper::new());
//!
//! let result = pipeline.run("hello world")?;
//! ```

use crate::lex::lexers::base_tokenization;
use crate::lex::pipeline::mapper::{StreamMapper, TransformationError};
use crate::lex::pipeline::stream::TokenStream;

/// A pipeline that chains StreamMapper transformations together.
///
/// The pipeline handles the complete flow from source text to transformed TokenStream:
/// 1. Base tokenization (lexing)
/// 2. Conversion to TokenStream
/// 3. Sequential application of transformations
///
/// Transformations are applied in the order they were added via `add_transformation`.
pub struct Pipeline {
    /// The sequence of transformations to apply
    transformations: Vec<Box<dyn StreamMapper>>,
}

impl Pipeline {
    /// Create a new empty pipeline.
    ///
    /// The pipeline starts with no transformations. Add transformations using
    /// `add_transformation()` before calling `run()`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let pipeline = Pipeline::new();
    /// ```
    pub fn new() -> Self {
        Pipeline {
            transformations: Vec::new(),
        }
    }

    /// Add a transformation to the pipeline.
    ///
    /// Transformations are applied in the order they're added. This method uses
    /// the builder pattern, allowing chaining:
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let pipeline = Pipeline::new()
    ///     .add_transformation(FirstMapper::new())
    ///     .add_transformation(SecondMapper::new());
    /// ```
    ///
    /// # Arguments
    ///
    /// * `mapper` - A StreamMapper implementation to add to the pipeline
    pub fn add_transformation<T: StreamMapper + 'static>(mut self, mapper: T) -> Self {
        self.transformations.push(Box::new(mapper));
        self
    }

    /// Run the pipeline on source text.
    ///
    /// This performs the complete transformation flow:
    /// 1. Tokenize source using base_tokenization
    /// 2. Convert tokens to TokenStream::Flat
    /// 3. Apply each transformation in sequence
    /// 4. Return final TokenStream
    ///
    /// The returned TokenStream can be either Flat or Tree depending on which
    /// transformations were applied.
    ///
    /// # Arguments
    ///
    /// * `source` - The source text to tokenize and transform
    ///
    /// # Returns
    ///
    /// The final TokenStream after all transformations, or a TransformationError
    /// if any transformation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut pipeline = Pipeline::new()
    ///     .add_transformation(SomeMapper::new());
    ///
    /// let result = pipeline.run("hello world")?;
    /// ```
    pub fn run(&mut self, source: &str) -> Result<TokenStream, TransformationError> {
        // Step 1: Base tokenization
        let tokens = base_tokenization::tokenize(source);

        // Step 2: Convert to TokenStream::Flat
        let mut stream = TokenStream::Flat(tokens);

        // Step 3: Apply each transformation in sequence
        for transformation in &mut self.transformations {
            stream = crate::lex::pipeline::mapper::walk_stream(stream, transformation.as_mut())?;
        }

        // Step 4: Return final TokenStream
        Ok(stream)
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::lexers::tokens::Token;
    use std::ops::Range as ByteRange;

    // Dummy mapper for testing - counts tokens
    struct TokenCounterMapper {
        count: usize,
    }

    impl TokenCounterMapper {
        fn new() -> Self {
            TokenCounterMapper { count: 0 }
        }
    }

    impl StreamMapper for TokenCounterMapper {
        fn map_flat(
            &mut self,
            tokens: Vec<(Token, ByteRange<usize>)>,
        ) -> Result<TokenStream, TransformationError> {
            self.count += tokens.len();
            Ok(TokenStream::Flat(tokens))
        }
    }

    // Dummy mapper that converts to uppercase
    struct UppercaseMapper;

    impl StreamMapper for UppercaseMapper {
        fn map_flat(
            &mut self,
            tokens: Vec<(Token, ByteRange<usize>)>,
        ) -> Result<TokenStream, TransformationError> {
            let transformed = tokens
                .into_iter()
                .map(|(token, range)| {
                    let new_token = match token {
                        Token::Text(s) => Token::Text(s.to_uppercase()),
                        other => other,
                    };
                    (new_token, range)
                })
                .collect();
            Ok(TokenStream::Flat(transformed))
        }
    }

    // Dummy mapper that fails
    struct FailingMapper;

    impl StreamMapper for FailingMapper {
        fn map_flat(
            &mut self,
            _tokens: Vec<(Token, ByteRange<usize>)>,
        ) -> Result<TokenStream, TransformationError> {
            Err(TransformationError::Error(
                "Intentional failure".to_string(),
            ))
        }
    }

    #[test]
    fn test_pipeline_new() {
        let pipeline = Pipeline::new();
        assert_eq!(pipeline.transformations.len(), 0);
    }

    #[test]
    fn test_pipeline_default() {
        let pipeline = Pipeline::default();
        assert_eq!(pipeline.transformations.len(), 0);
    }

    #[test]
    fn test_pipeline_add_transformation() {
        let pipeline = Pipeline::new()
            .add_transformation(TokenCounterMapper::new())
            .add_transformation(UppercaseMapper);

        assert_eq!(pipeline.transformations.len(), 2);
    }

    #[test]
    fn test_pipeline_run_empty() {
        // Empty pipeline should just tokenize
        let mut pipeline = Pipeline::new();
        let result = pipeline.run("hello world");

        assert!(result.is_ok());
        match result.unwrap() {
            TokenStream::Flat(tokens) => {
                assert!(!tokens.is_empty());
                // Should have Text("hello"), Whitespace, Text("world") at minimum
                assert!(tokens.len() >= 3);
            }
            _ => panic!("Expected Flat stream from empty pipeline"),
        }
    }

    #[test]
    fn test_pipeline_run_single_transformation() {
        let mut pipeline = Pipeline::new().add_transformation(UppercaseMapper);

        let result = pipeline.run("hello");

        assert!(result.is_ok());
        match result.unwrap() {
            TokenStream::Flat(tokens) => {
                // Find the text token and verify it's uppercase
                let text_tokens: Vec<_> = tokens
                    .iter()
                    .filter_map(|(token, _)| match token {
                        Token::Text(s) => Some(s.clone()),
                        _ => None,
                    })
                    .collect();

                assert!(!text_tokens.is_empty());
                assert_eq!(text_tokens[0], "HELLO");
            }
            _ => panic!("Expected Flat stream"),
        }
    }

    #[test]
    fn test_pipeline_run_multiple_transformations() {
        // Chain counter and uppercase
        let mut pipeline = Pipeline::new()
            .add_transformation(TokenCounterMapper::new())
            .add_transformation(UppercaseMapper);

        let result = pipeline.run("hello world");

        assert!(result.is_ok());
        match result.unwrap() {
            TokenStream::Flat(tokens) => {
                // Verify uppercase worked
                let has_uppercase = tokens.iter().any(|(token, _)| match token {
                    Token::Text(s) => s.chars().any(|c| c.is_uppercase()),
                    _ => false,
                });
                assert!(has_uppercase, "Should have uppercase text");
            }
            _ => panic!("Expected Flat stream"),
        }
    }

    #[test]
    fn test_pipeline_run_failure() {
        let mut pipeline = Pipeline::new().add_transformation(FailingMapper);

        let result = pipeline.run("hello");

        assert!(result.is_err());
        match result.unwrap_err() {
            TransformationError::Error(msg) => {
                assert_eq!(msg, "Intentional failure");
            }
        }
    }

    #[test]
    fn test_pipeline_run_empty_source() {
        let mut pipeline = Pipeline::new();
        let result = pipeline.run("");

        assert!(result.is_ok());
        match result.unwrap() {
            TokenStream::Flat(tokens) => {
                // Empty source should produce minimal/no tokens
                assert!(tokens.is_empty() || tokens.len() <= 1);
            }
            _ => panic!("Expected Flat stream"),
        }
    }

    #[test]
    fn test_pipeline_builder_pattern() {
        // Verify builder pattern works smoothly
        let pipeline = Pipeline::new()
            .add_transformation(TokenCounterMapper::new())
            .add_transformation(UppercaseMapper)
            .add_transformation(TokenCounterMapper::new());

        assert_eq!(pipeline.transformations.len(), 3);
    }

    #[test]
    fn test_pipeline_preserves_token_ranges() {
        let mut pipeline = Pipeline::new();
        let result = pipeline.run("hello");

        assert!(result.is_ok());
        match result.unwrap() {
            TokenStream::Flat(tokens) => {
                // Verify ranges are valid
                for (_, range) in tokens {
                    assert!(range.start <= range.end);
                    assert!(range.end <= "hello".len());
                }
            }
            _ => panic!("Expected Flat stream"),
        }
    }
}
