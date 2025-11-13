//! Lexing transformation pipeline
//!
//! This module provides the low-level transformation pipeline that operates on
//! token streams after base tokenization. It is responsible for chaining
//! `StreamMapper` implementations together.
//!
//! The pipeline is intentionally focused on lexing concerns only. Higher level
//! orchestration (parsing, building, configuration loading) lives in the
//! `lex::pipeline` module.

use crate::lex::lexing::base_tokenization;
use crate::lex::pipeline::mapper::{StreamMapper, TransformationError};
use crate::lex::pipeline::stream::TokenStream;

/// A pipeline that chains StreamMapper transformations together for the lexing stage.
pub struct LexingPipeline {
    transformations: Vec<Box<dyn StreamMapper>>,
}

impl LexingPipeline {
    /// Create a new empty lexing pipeline.
    pub fn new() -> Self {
        Self {
            transformations: Vec::new(),
        }
    }

    /// Add a transformation to the pipeline.
    ///
    /// Transformations are executed in the order they are added.
    pub fn add_transformation<T: StreamMapper + 'static>(&mut self, mapper: T) {
        self.transformations.push(Box::new(mapper));
    }

    /// Number of transformations registered in the pipeline.
    pub fn transformation_count(&self) -> usize {
        self.transformations.len()
    }

    /// Run the lexing pipeline on source text, returning the transformed token stream.
    pub fn run(&mut self, source: &str) -> Result<TokenStream, TransformationError> {
        // Step 1: Base tokenization
        let tokens = base_tokenization::tokenize(source);

        // Step 2: Convert to TokenStream::Flat
        let mut stream = TokenStream::Flat(tokens);

        // Step 3: Apply transformations sequentially
        for transformation in &mut self.transformations {
            stream = crate::lex::pipeline::mapper::walk_stream(stream, transformation.as_mut())?;
        }

        Ok(stream)
    }
}

impl Default for LexingPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::token::Token;
    use crate::lex::pipeline::stream::TokenStream;
    use std::ops::Range as ByteRange;

    struct TokenCounterMapper {
        count: usize,
    }

    impl TokenCounterMapper {
        fn new() -> Self {
            Self { count: 0 }
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

    #[test]
    fn test_new_pipeline_has_no_transformations() {
        let pipeline = LexingPipeline::new();
        assert_eq!(pipeline.transformation_count(), 0);
    }

    #[test]
    fn test_add_transformation_increments_count() {
        let mut pipeline = LexingPipeline::new();
        pipeline.add_transformation(TokenCounterMapper::new());
        assert_eq!(pipeline.transformation_count(), 1);
    }

    #[test]
    fn test_run_returns_token_stream() {
        let mut pipeline = LexingPipeline::new();
        let result = pipeline.run("hello world");

        assert!(result.is_ok());
        match result.unwrap() {
            TokenStream::Flat(tokens) => assert!(!tokens.is_empty()),
            _ => panic!("expected flat stream"),
        }
    }
}
