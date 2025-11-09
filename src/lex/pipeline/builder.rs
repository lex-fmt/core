//! Unified pipeline builder orchestrating lexing and parsing stages.
//!
//! This module wires the stage-specific pipelines (lexing, parsing, building)
//! together while keeping the ergonomic builder-style API that internal tools
//! and tests rely on.

use crate::lex::lexing::pipeline::LexingPipeline;
use crate::lex::parsing::pipeline::{self, AnalyzerConfig};
use crate::lex::parsing::Document;
use crate::lex::pipeline::mapper::{StreamMapper, TransformationError};
use crate::lex::pipeline::stream::TokenStream;

/// Output from pipeline execution
#[derive(Debug)]
pub enum PipelineOutput {
    /// Token stream output (lexing only)
    Tokens(TokenStream),
    /// Document AST output (full parsing)
    Document(Document),
}

/// Pipeline orchestrator that delegates to stage-specific pipelines.
pub struct Pipeline {
    lexing: LexingPipeline,
    analyzer: Option<AnalyzerConfig>,
}

impl Pipeline {
    /// Create a new empty pipeline.
    pub fn new() -> Self {
        Self {
            lexing: LexingPipeline::new(),
            analyzer: None,
        }
    }

    /// Add a lexing transformation. Transformations execute in insertion order.
    pub fn add_transformation<T: StreamMapper + 'static>(mut self, mapper: T) -> Self {
        self.lexing.add_transformation(mapper);
        self
    }

    /// Configure the pipeline to run an analyzer after lexing.
    pub fn with_analyzer(mut self, analyzer: AnalyzerConfig) -> Self {
        self.analyzer = Some(analyzer);
        self
    }

    /// Run the pipeline on the provided source.
    pub fn run(&mut self, source: &str) -> Result<PipelineOutput, TransformationError> {
        let stream = self.lexing.run(source)?;
        match self.analyzer {
            None => Ok(PipelineOutput::Tokens(stream)),
            Some(analyzer) => {
                let doc = pipeline::analyze(stream, source, analyzer)?;
                Ok(PipelineOutput::Document(doc))
            }
        }
    }

    /// Number of transformations registered in the underlying lexing pipeline.
    pub fn transformation_count(&self) -> usize {
        self.lexing.transformation_count()
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
    use crate::lex::lexing::tokens_core::Token;
    use crate::lex::pipeline::stream::TokenStream;
    use std::ops::Range as ByteRange;

    // Dummy mapper for testing - counts tokens
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
        assert_eq!(pipeline.transformation_count(), 0);
    }

    #[test]
    fn test_pipeline_default() {
        let pipeline = Pipeline::default();
        assert_eq!(pipeline.transformation_count(), 0);
    }

    #[test]
    fn test_pipeline_add_transformation() {
        let pipeline = Pipeline::new()
            .add_transformation(TokenCounterMapper::new())
            .add_transformation(UppercaseMapper);

        assert_eq!(pipeline.transformation_count(), 2);
    }

    #[test]
    fn test_pipeline_run_empty() {
        // Empty pipeline should just tokenize
        let mut pipeline = Pipeline::new();
        let result = pipeline.run("hello world");

        assert!(result.is_ok());
        match result.unwrap() {
            PipelineOutput::Tokens(TokenStream::Flat(tokens)) => {
                assert!(!tokens.is_empty());
                assert!(tokens.len() >= 3);
            }
            _ => panic!("Expected Tokens output with Flat stream from empty pipeline"),
        }
    }

    #[test]
    fn test_pipeline_run_single_transformation() {
        let mut pipeline = Pipeline::new().add_transformation(UppercaseMapper);

        let result = pipeline.run("hello");

        assert!(result.is_ok());
        match result.unwrap() {
            PipelineOutput::Tokens(TokenStream::Flat(tokens)) => {
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
            _ => panic!("Expected Tokens output with Flat stream"),
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
            PipelineOutput::Tokens(TokenStream::Flat(tokens)) => {
                let has_uppercase = tokens.iter().any(|(token, _)| match token {
                    Token::Text(s) => s.chars().any(|c| c.is_uppercase()),
                    _ => false,
                });
                assert!(has_uppercase, "Should have uppercase text");
            }
            _ => panic!("Expected Tokens output with Flat stream"),
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
            PipelineOutput::Tokens(TokenStream::Flat(tokens)) => {
                assert!(tokens.is_empty() || tokens.len() <= 1);
            }
            _ => panic!("Expected Tokens output with Flat stream"),
        }
    }

    #[test]
    fn test_pipeline_builder_pattern() {
        // Verify builder pattern works smoothly
        let pipeline = Pipeline::new()
            .add_transformation(TokenCounterMapper::new())
            .add_transformation(UppercaseMapper)
            .add_transformation(TokenCounterMapper::new());

        assert_eq!(pipeline.transformation_count(), 3);
    }

    #[test]
    fn test_pipeline_preserves_token_ranges() {
        let mut pipeline = Pipeline::new();
        let result = pipeline.run("hello");

        assert!(result.is_ok());
        match result.unwrap() {
            PipelineOutput::Tokens(TokenStream::Flat(tokens)) => {
                for (_, range) in tokens {
                    assert!(range.start <= range.end);
                    assert!(range.end <= "hello".len());
                }
            }
            _ => panic!("Expected Tokens output with Flat stream"),
        }
    }

    #[test]
    fn test_pipeline_with_reference_analyzer() {
        use crate::lex::lexing::transformations::*;

        let mut pipeline = Pipeline::new()
            .add_transformation(SemanticIndentationMapper::new())
            .add_transformation(BlankLinesMapper::new())
            .with_analyzer(AnalyzerConfig::Reference);

        let result = pipeline.run("Hello world\n");

        match result {
            Ok(PipelineOutput::Document(doc)) => {
                assert!(
                    !doc.root.children.is_empty(),
                    "Document should have content"
                );
            }
            Ok(_) => panic!("Expected Document output"),
            Err(e) => panic!("Pipeline failed: {:?}", e),
        }
    }

    #[test]
    fn test_pipeline_with_linebased_analyzer() {
        use crate::lex::lexing::transformations::*;

        let mut pipeline = Pipeline::new()
            .add_transformation(SemanticIndentationMapper::new())
            .add_transformation(BlankLinesMapper::new())
            .with_analyzer(AnalyzerConfig::Linebased);

        let result = pipeline.run("Hello:\n    World\n");

        assert!(result.is_ok());
        match result.unwrap() {
            PipelineOutput::Document(doc) => {
                assert!(
                    !doc.root.children.is_empty(),
                    "Document should have content"
                );
            }
            _ => panic!("Expected Document output"),
        }
    }

    #[test]
    fn test_pipeline_without_analyzer_returns_tokens() {
        let mut pipeline = Pipeline::new();
        let result = pipeline.run("Hello world");

        assert!(result.is_ok());
        match result.unwrap() {
            PipelineOutput::Tokens(_) => {} // Success
            PipelineOutput::Document(_) => panic!("Should return tokens, not document"),
        }
    }
}
