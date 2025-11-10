//! Pipeline executor that runs processing configurations

use crate::lex::formats::{FormatError, FormatRegistry};
use crate::lex::lexing::base_tokenization;
use crate::lex::lexing::transformations::*;
use crate::lex::parsing::{builder, Document};
use crate::lex::pipeline::config::{
    AnalysisSpec, BuilderSpec, ConfigRegistry, PipelineSpec, TargetSpec,
};
use crate::lex::pipeline::mapper::walk_stream;
use crate::lex::pipeline::stream::TokenStream;
use std::fmt;

/// Errors during pipeline execution
#[derive(Debug, Clone)]
pub enum ExecutionError {
    ConfigNotFound(String),
    TransformationFailed(String),
    ParsingFailed(String),
    FormatError(FormatError),
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionError::ConfigNotFound(name) => write!(f, "Config '{}' not found", name),
            ExecutionError::TransformationFailed(msg) => {
                write!(f, "Transformation failed: {}", msg)
            }
            ExecutionError::ParsingFailed(msg) => write!(f, "Parsing failed: {}", msg),
            ExecutionError::FormatError(err) => write!(f, "Format error: {}", err),
        }
    }
}

impl std::error::Error for ExecutionError {}

/// Output from pipeline execution
#[derive(Debug)]
pub enum ExecutionOutput {
    Tokens(TokenStream),
    Document(Document),
    Serialized(String),
}

/// Executes processing configurations
pub struct PipelineExecutor {
    registry: ConfigRegistry,
    format_registry: FormatRegistry,
}

impl PipelineExecutor {
    /// Create executor with default configurations
    pub fn new() -> Self {
        Self {
            registry: ConfigRegistry::with_defaults(),
            format_registry: FormatRegistry::with_defaults(),
        }
    }

    /// Create executor with custom registry
    pub fn with_registry(registry: ConfigRegistry) -> Self {
        Self {
            registry,
            format_registry: FormatRegistry::with_defaults(),
        }
    }

    /// Create executor with custom registries
    pub fn with_registries(registry: ConfigRegistry, format_registry: FormatRegistry) -> Self {
        Self {
            registry,
            format_registry,
        }
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
            TargetSpec::Serialized {
                analyzer,
                builder,
                format,
            } => {
                // First parse to AST
                let doc = self.analyze_and_build(stream, source, analyzer, builder)?;
                // Then serialize using FormatRegistry
                let output = self
                    .format_registry
                    .serialize(&doc, format)
                    .map_err(ExecutionError::FormatError)?;
                Ok(ExecutionOutput::Serialized(output))
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
                // Indentation pipeline: no line grouping
                stream = walk_stream(stream, &mut SemanticIndentationMapper::new())
                    .map_err(|e| ExecutionError::TransformationFailed(e.to_string()))?;
                Ok(stream)
            }
            PipelineSpec::LinebasedFlat | PipelineSpec::Linebased => {
                // Linebased pipeline: add line token grouping after base transformations
                stream = walk_stream(stream, &mut SemanticIndentationMapper::new())
                    .map_err(|e| ExecutionError::TransformationFailed(e.to_string()))?;
                stream = walk_stream(stream, &mut LineTokenGroupingMapper::new())
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
                // Convert grouped tokens to line tokens and parse
                crate::lex::parsing::linebased::parse_from_grouped_stream(stream, source).map_err(
                    |e| ExecutionError::ParsingFailed(format!("Linebased analyzer failed: {}", e)),
                )
            }
        }
    }

    /// List all available configurations
    pub fn list_configs(&self) -> Vec<&crate::lex::pipeline::config::ProcessingConfig> {
        self.registry.list_all()
    }

    /// Get the config registry
    pub fn registry(&self) -> &ConfigRegistry {
        &self.registry
    }

    /// Get the format registry
    pub fn format_registry(&self) -> &FormatRegistry {
        &self.format_registry
    }

    /// Execute a configuration and serialize the result to a format
    ///
    /// This is a convenience method that combines `execute()` with format serialization.
    /// If the pipeline produces a Document, it will be serialized using the specified format.
    /// If the pipeline produces Tokens, an error will be returned.
    ///
    /// # Arguments
    ///
    /// * `config_name` - Name of the configuration to execute
    /// * `source` - Source text to process
    /// * `format` - Name of the format to serialize to (e.g., "treeviz", "tag")
    ///
    /// # Returns
    ///
    /// The serialized string output
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The configuration is not found
    /// - The pipeline fails
    /// - The format is not found
    /// - The pipeline produces Tokens instead of a Document
    pub fn execute_and_serialize(
        &self,
        config_name: &str,
        source: &str,
        format: &str,
    ) -> Result<String, ExecutionError> {
        let output = self.execute(config_name, source)?;

        match output {
            ExecutionOutput::Document(doc) => self
                .format_registry
                .serialize(&doc, format)
                .map_err(ExecutionError::FormatError),
            ExecutionOutput::Serialized(s) => Ok(s), // Already serialized
            ExecutionOutput::Tokens(_) => Err(ExecutionError::FormatError(
                FormatError::SerializationError(
                    "Cannot serialize tokens to format (pipeline must produce AST)".to_string(),
                ),
            )),
        }
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
                assert!(
                    !doc.root.children.is_empty(),
                    "Document should have content"
                );
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
                assert!(
                    !doc.root.children.is_empty(),
                    "Document should have content"
                );
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

        let err4 = ExecutionError::FormatError(crate::lex::formats::FormatError::FormatNotFound(
            "test".into(),
        ));
        assert_eq!(format!("{}", err4), "Format error: Format 'test' not found");
    }

    #[test]
    fn test_format_registry_accessor() {
        let executor = PipelineExecutor::new();
        let registry = executor.format_registry();

        assert!(registry.has("treeviz"));
        assert!(registry.has("tag"));
    }

    #[test]
    fn test_execute_and_serialize_treeviz() {
        let executor = PipelineExecutor::new();
        let result = executor.execute_and_serialize("default", "Hello world\n", "treeviz");

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("⧉")); // Document icon
        assert!(output.contains("¶")); // Paragraph icon
    }

    #[test]
    fn test_execute_and_serialize_tag() {
        let executor = PipelineExecutor::new();
        let result = executor.execute_and_serialize("default", "Hello world\n", "tag");

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("<document>"));
        assert!(output.contains("<paragraph>"));
        assert!(output.contains("</document>"));
    }

    #[test]
    fn test_execute_and_serialize_format_not_found() {
        let executor = PipelineExecutor::new();
        let result = executor.execute_and_serialize("default", "Hello world\n", "nonexistent");

        assert!(result.is_err());
        match result.unwrap_err() {
            ExecutionError::FormatError(crate::lex::formats::FormatError::FormatNotFound(name)) => {
                assert_eq!(name, "nonexistent");
            }
            _ => panic!("Expected FormatNotFound error"),
        }
    }

    #[test]
    fn test_execute_and_serialize_tokens_error() {
        let executor = PipelineExecutor::new();
        let result =
            executor.execute_and_serialize("tokens-indentation", "Hello world\n", "treeviz");

        assert!(result.is_err());
        match result.unwrap_err() {
            ExecutionError::FormatError(crate::lex::formats::FormatError::SerializationError(
                msg,
            )) => {
                assert!(msg.contains("Cannot serialize tokens"));
            }
            _ => panic!("Expected SerializationError"),
        }
    }

    #[test]
    fn test_with_registries() {
        use crate::lex::formats::FormatRegistry;
        use crate::lex::pipeline::config::{
            ConfigRegistry, PipelineSpec, ProcessingConfig, TargetSpec,
        };

        let mut config_registry = ConfigRegistry::new();
        config_registry.register(ProcessingConfig {
            name: "custom".into(),
            description: "Custom config".into(),
            pipeline_spec: PipelineSpec::Indentation,
            target: TargetSpec::Tokens,
        });

        let format_registry = FormatRegistry::with_defaults();

        let executor = PipelineExecutor::with_registries(config_registry, format_registry);
        assert!(executor.registry().has("custom"));
        assert!(executor.format_registry().has("treeviz"));
    }
}
