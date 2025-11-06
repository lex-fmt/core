//! Pipeline executor that runs processing configurations

use crate::lex::lexing::base_tokenization;
use crate::lex::parsing::{builder, Document};
use crate::lex::pipeline::config::{
    AnalysisSpec, BuilderSpec, ConfigRegistry, PipelineSpec, TargetSpec,
};
use crate::lex::pipeline::mapper::walk_stream;
use crate::lex::pipeline::mappers::*;
use crate::lex::pipeline::stream::TokenStream;
use std::fmt;

/// Errors during pipeline execution
#[derive(Debug, Clone)]
pub enum ExecutionError {
    ConfigNotFound(String),
    TransformationFailed(String),
    ParsingFailed(String),
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionError::ConfigNotFound(name) => write!(f, "Config '{}' not found", name),
            ExecutionError::TransformationFailed(msg) => {
                write!(f, "Transformation failed: {}", msg)
            }
            ExecutionError::ParsingFailed(msg) => write!(f, "Parsing failed: {}", msg),
        }
    }
}

impl std::error::Error for ExecutionError {}

/// Output from pipeline execution
#[derive(Debug)]
pub enum ExecutionOutput {
    Tokens(TokenStream),
    Document(Document),
}

/// Executes processing configurations
pub struct PipelineExecutor {
    registry: ConfigRegistry,
}

impl PipelineExecutor {
    /// Create executor with default configurations
    pub fn new() -> Self {
        Self {
            registry: ConfigRegistry::with_defaults(),
        }
    }

    /// Create executor with custom registry
    pub fn with_registry(registry: ConfigRegistry) -> Self {
        Self { registry }
    }

    /// Execute a named configuration
    pub fn execute(
        &self,
        config_name: &str,
        source: &str,
    ) -> Result<ExecutionOutput, ExecutionError> {
        let config = self
            .registry
            .get(config_name)
            .ok_or_else(|| ExecutionError::ConfigNotFound(config_name.to_string()))?;

        // Step 1: Base tokenization
        let source_with_newline = crate::lex::lexing::ensure_source_ends_with_newline(source);
        let tokens = base_tokenization::tokenize(&source_with_newline);
        let mut stream = TokenStream::Flat(tokens);

        // Step 2: Apply transformations
        stream = self.apply_transformations(stream, &config.pipeline_spec)?;

        // Step 3: Process target
        match &config.target {
            TargetSpec::Tokens => Ok(ExecutionOutput::Tokens(stream)),
            TargetSpec::Ast { analyzer, builder } => {
                let doc = self.analyze_and_build(stream, source, analyzer, builder)?;
                Ok(ExecutionOutput::Document(doc))
            }
        }
    }

    fn apply_transformations(
        &self,
        mut stream: TokenStream,
        spec: &PipelineSpec,
    ) -> Result<TokenStream, ExecutionError> {
        match spec {
            PipelineSpec::Raw => {
                // No transformations, return as-is
                Ok(stream)
            }
            PipelineSpec::Indentation => {
                stream = walk_stream(stream, &mut NormalizeWhitespaceMapper::new())
                    .map_err(|e| ExecutionError::TransformationFailed(e.to_string()))?;
                stream = walk_stream(stream, &mut SemanticIndentationMapper::new())
                    .map_err(|e| ExecutionError::TransformationFailed(e.to_string()))?;
                stream = walk_stream(stream, &mut BlankLinesMapper::new())
                    .map_err(|e| ExecutionError::TransformationFailed(e.to_string()))?;
                Ok(stream)
            }
            PipelineSpec::LinebasedFlat => {
                stream = walk_stream(stream, &mut NormalizeWhitespaceMapper::new())
                    .map_err(|e| ExecutionError::TransformationFailed(e.to_string()))?;
                stream = walk_stream(stream, &mut SemanticIndentationMapper::new())
                    .map_err(|e| ExecutionError::TransformationFailed(e.to_string()))?;
                stream = walk_stream(stream, &mut BlankLinesMapper::new())
                    .map_err(|e| ExecutionError::TransformationFailed(e.to_string()))?;
                stream = walk_stream(stream, &mut ToLineTokensMapper::new())
                    .map_err(|e| ExecutionError::TransformationFailed(e.to_string()))?;
                Ok(stream)
            }
            PipelineSpec::Linebased => {
                stream = walk_stream(stream, &mut NormalizeWhitespaceMapper::new())
                    .map_err(|e| ExecutionError::TransformationFailed(e.to_string()))?;
                stream = walk_stream(stream, &mut SemanticIndentationMapper::new())
                    .map_err(|e| ExecutionError::TransformationFailed(e.to_string()))?;
                stream = walk_stream(stream, &mut BlankLinesMapper::new())
                    .map_err(|e| ExecutionError::TransformationFailed(e.to_string()))?;
                stream = walk_stream(stream, &mut ToLineTokensMapper::new())
                    .map_err(|e| ExecutionError::TransformationFailed(e.to_string()))?;

                let mut indent_mapper = IndentationToTreeMapper::new();
                stream = indent_mapper
                    .transform(stream)
                    .map_err(|e| ExecutionError::TransformationFailed(e.to_string()))?;

                Ok(stream)
            }
        }
    }

    fn analyze_and_build(
        &self,
        stream: TokenStream,
        source: &str,
        analyzer: &AnalysisSpec,
        _builder: &BuilderSpec,
    ) -> Result<Document, ExecutionError> {
        // Note: Currently only one builder (LSP) is supported, so we don't need to match on it
        match analyzer {
            AnalysisSpec::Reference => {
                let tokens = stream.unroll();
                let parse_node =
                    crate::lex::parsing::reference::parse(tokens, source).map_err(|_| {
                        ExecutionError::ParsingFailed("Reference analyzer failed".to_string())
                    })?;
                let builder = builder::AstBuilder::new(source);
                Ok(builder.build(parse_node))
            }
            AnalysisSpec::Linebased => {
                let container =
                    crate::lex::pipeline::adapters_linebased::token_stream_to_line_container(
                        stream,
                    )
                    .map_err(|e| {
                        ExecutionError::ParsingFailed(format!("Stream conversion failed: {:?}", e))
                    })?;
                crate::lex::parsing::linebased::parse_experimental_v2(container, source).map_err(
                    |e| ExecutionError::ParsingFailed(format!("Linebased analyzer failed: {}", e)),
                )
            }
        }
    }

    /// List all available configurations
    pub fn list_configs(&self) -> Vec<&crate::lex::pipeline::config::ProcessingConfig> {
        self.registry.list_all()
    }

    /// Get the registry
    pub fn registry(&self) -> &ConfigRegistry {
        &self.registry
    }
}

impl Default for PipelineExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let executor = PipelineExecutor::new();
        assert!(!executor.list_configs().is_empty());
    }

    #[test]
    fn test_executor_default() {
        let executor = PipelineExecutor::default();
        assert!(!executor.list_configs().is_empty());
    }

    #[test]
    fn test_execute_default_config() {
        let executor = PipelineExecutor::new();
        let result = executor.execute("default", "Hello world\n");

        assert!(result.is_ok());
        match result.unwrap() {
            ExecutionOutput::Document(doc) => {
                assert!(!doc.root.content.is_empty(), "Document should have content");
            }
            _ => panic!("Expected Document output"),
        }
    }

    #[test]
    fn test_execute_tokens_config() {
        let executor = PipelineExecutor::new();
        let result = executor.execute("tokens-indentation", "Hello world");

        assert!(result.is_ok());
        match result.unwrap() {
            ExecutionOutput::Tokens(stream) => {
                let tokens = stream.unroll();
                assert!(!tokens.is_empty(), "Should have tokens");
            }
            _ => panic!("Expected Tokens output"),
        }
    }

    #[test]
    fn test_execute_linebased_config() {
        let executor = PipelineExecutor::new();
        let result = executor.execute("linebased", "Hello:\n    World\n");

        assert!(result.is_ok());
        match result.unwrap() {
            ExecutionOutput::Document(doc) => {
                assert!(!doc.root.content.is_empty(), "Document should have content");
            }
            _ => panic!("Expected Document output"),
        }
    }

    #[test]
    fn test_execute_nonexistent_config() {
        let executor = PipelineExecutor::new();
        let result = executor.execute("nonexistent", "Hello");

        assert!(result.is_err());
        match result.unwrap_err() {
            ExecutionError::ConfigNotFound(name) => {
                assert_eq!(name, "nonexistent");
            }
            _ => panic!("Expected ConfigNotFound error"),
        }
    }

    #[test]
    fn test_list_configs() {
        let executor = PipelineExecutor::new();
        let configs = executor.list_configs();

        let config_names: Vec<_> = configs.iter().map(|c| c.name.as_str()).collect();
        assert!(config_names.contains(&"default"));
        assert!(config_names.contains(&"linebased"));
        assert!(config_names.contains(&"tokens-indentation"));
    }

    #[test]
    fn test_execute_tokens_raw_config() {
        let executor = PipelineExecutor::new();
        let result = executor.execute("tokens-raw", "Hello world");

        assert!(result.is_ok());
        match result.unwrap() {
            ExecutionOutput::Tokens(_) => {} // Success
            _ => panic!("Expected Tokens output"),
        }
    }

    #[test]
    fn test_execute_tokens_linebased_flat() {
        let executor = PipelineExecutor::new();
        let result = executor.execute("tokens-linebased-flat", "Hello world");

        assert!(result.is_ok());
        match result.unwrap() {
            ExecutionOutput::Tokens(_) => {} // Success
            _ => panic!("Expected Tokens output"),
        }
    }

    #[test]
    fn test_execute_tokens_linebased_tree() {
        let executor = PipelineExecutor::new();
        let result = executor.execute("tokens-linebased-tree", "Hello world");

        assert!(result.is_ok());
        match result.unwrap() {
            ExecutionOutput::Tokens(_) => {} // Success
            _ => panic!("Expected Tokens output"),
        }
    }

    #[test]
    fn test_with_custom_registry() {
        use crate::lex::pipeline::config::{PipelineSpec, ProcessingConfig, TargetSpec};

        let mut registry = ConfigRegistry::new();
        registry.register(ProcessingConfig {
            name: "custom".into(),
            description: "Custom config".into(),
            pipeline_spec: PipelineSpec::Indentation,
            target: TargetSpec::Tokens,
        });

        let executor = PipelineExecutor::with_registry(registry);
        assert!(executor.registry().has("custom"));
        assert!(!executor.registry().has("default"));
    }

    #[test]
    fn test_execution_error_display() {
        let err1 = ExecutionError::ConfigNotFound("test".into());
        assert_eq!(format!("{}", err1), "Config 'test' not found");

        let err2 = ExecutionError::TransformationFailed("error".into());
        assert_eq!(format!("{}", err2), "Transformation failed: error");

        let err3 = ExecutionError::ParsingFailed("error".into());
        assert_eq!(format!("{}", err3), "Parsing failed: error");
    }
}
