//! Parsing stage pipeline
//!
//! Provides analyzer configuration and helpers for running the different
//! syntactic analyzers over token streams produced by the lexing pipeline.

use crate::lex::parsing::Document;
use crate::lex::pipeline::mapper::TransformationError;
use crate::lex::pipeline::stream::TokenStream;

/// Which analyzer to use for syntactic analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalyzerConfig {
    /// Reference combinator analyzer
    Reference,
    /// Linebased declarative grammar analyzer
    Linebased,
}

/// Run the selected analyzer over the provided token stream and build a document.
pub fn analyze(
    stream: TokenStream,
    source: &str,
    analyzer: AnalyzerConfig,
) -> Result<Document, TransformationError> {
    match analyzer {
        AnalyzerConfig::Reference => {
            let tokens = stream.unroll();
            let parse_node = crate::lex::parsing::reference::parse(tokens, source)
                .map_err(|_| TransformationError::Error("Reference analyzer failed".to_string()))?;
            Ok(crate::lex::building::pipeline::build_document(
                parse_node, source,
            ))
        }
        AnalyzerConfig::Linebased => {
            let tokens = stream.unroll();
            crate::lex::parsing::linebased::parse_from_flat_tokens(tokens, source).map_err(|e| {
                TransformationError::Error(format!("Linebased analyzer failed: {}", e))
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::lexing::pipeline::LexingPipeline;
    use crate::lex::lexing::transformations::{BlankLinesMapper, SemanticIndentationMapper};

    fn baseline_pipeline() -> LexingPipeline {
        let mut pipeline = LexingPipeline::new();
        pipeline.add_transformation(SemanticIndentationMapper::new());
        pipeline.add_transformation(BlankLinesMapper::new());
        pipeline
    }

    #[test]
    fn test_reference_analyzer_produces_document() {
        let source = "Hello world\n";
        let mut lexing = baseline_pipeline();
        let stream = lexing.run(source).expect("lexing failed");

        let result = analyze(stream, source, AnalyzerConfig::Reference);
        assert!(result.is_ok());
        assert!(!result.unwrap().root.children.is_empty());
    }

    #[test]
    fn test_linebased_analyzer_produces_document() {
        let source = "Hello:\n    World\n";
        let mut lexing = baseline_pipeline();
        let stream = lexing.run(source).expect("lexing failed");

        let result = analyze(stream, source, AnalyzerConfig::Linebased);
        assert!(result.is_ok());
        assert!(!result.unwrap().root.children.is_empty());
    }
}
