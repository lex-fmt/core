# Implementation Plan: Unified Pipeline Control

## Overview

This plan breaks down the unified pipeline refactoring into small, incremental steps that can be implemented, tested, and reviewed independently. Each step is non-breaking and adds value on its own.

## Phase 1: Extend Pipeline to Support Parsing (Foundation)

**Goal**: Make the low-level `Pipeline` capable of handling parsing, not just lexing.

**Status**: Non-breaking addition

### Step 1.1: Add Parser Support to Pipeline Builder

**File**: `src/lex/pipeline/builder.rs`

**Changes**:
```rust
// Add to Pipeline struct
pub struct Pipeline {
    transformations: Vec<Box<dyn StreamMapper>>,
    // NEW: Optional parser to run after transformations
    parser: Option<ParserConfig>,
}

// NEW: Parser configuration
pub enum ParserConfig {
    Reference,
    Linebased,
}

// NEW: Extended output type
pub enum PipelineOutput {
    Tokens(TokenStream),
    Document(Document),
}

impl Pipeline {
    // NEW: Method to add parser stage
    pub fn with_parser(mut self, parser: ParserConfig) -> Self {
        self.parser = Some(parser);
        self
    }

    // MODIFY: Change return type
    pub fn run(&mut self, source: &str) -> Result<PipelineOutput, TransformationError> {
        // Existing transformation logic...
        let stream = /* ... apply transformations ... */;

        // NEW: If parser configured, continue to parsing
        match &self.parser {
            None => Ok(PipelineOutput::Tokens(stream)),
            Some(ParserConfig::Reference) => {
                let tokens = stream.unroll();
                let doc = parsers::reference::parse(tokens, source)
                    .map_err(|_| TransformationError::Error("Parse failed".into()))?;
                Ok(PipelineOutput::Document(doc))
            }
            Some(ParserConfig::Linebased) => {
                let container = adapters_linebased::token_stream_to_line_container(stream)
                    .map_err(|e| TransformationError::Error(format!("{:?}", e)))?;
                let doc = parsers::linebased::parse_experimental_v2(container, source)
                    .map_err(|e| TransformationError::Error(e))?;
                Ok(PipelineOutput::Document(doc))
            }
        }
    }
}
```

**Tests**: Add tests in `builder.rs`:
```rust
#[test]
fn test_pipeline_with_parser() {
    let mut pipeline = Pipeline::new()
        .add_transformation(NormalizeWhitespaceMapper::new())
        .add_transformation(SemanticIndentationMapper::new())
        .add_transformation(BlankLinesMapper::new())
        .with_parser(ParserConfig::Reference);

    let result = pipeline.run("Hello world").unwrap();
    assert!(matches!(result, PipelineOutput::Document(_)));
}

#[test]
fn test_pipeline_without_parser() {
    let mut pipeline = Pipeline::new()
        .add_transformation(NormalizeWhitespaceMapper::new());

    let result = pipeline.run("Hello world").unwrap();
    assert!(matches!(result, PipelineOutput::Tokens(_)));
}
```

**Validation**:
- ✅ All existing tests pass (Pipeline without parser still works)
- ✅ New tests demonstrate parsing capability
- ✅ No changes to external APIs yet

**Estimated effort**: 2-3 hours

---

## Phase 2: Introduce ProcessingConfig (Parallel System)

**Goal**: Create the new config-based system alongside existing code.

**Status**: Completely parallel, zero impact on existing code

### Step 2.1: Create Config Data Structures

**File**: `src/lex/pipeline/config.rs` (NEW)

**Changes**:
```rust
//! Processing configuration system for Lex pipelines

use std::collections::HashMap;

/// A named configuration specifying transformation pipeline and target
#[derive(Debug, Clone)]
pub struct ProcessingConfig {
    pub name: String,
    pub description: String,
    pub pipeline_spec: PipelineSpec,
    pub target: TargetSpec,
}

/// Which transformation pipeline to use
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineSpec {
    /// Standard indentation-based transformations
    Indentation,
    /// Linebased transformations (full, with tree)
    Linebased,
    /// Linebased up to LineTokens (no tree)
    LinebasedFlat,
    /// Raw tokens only (minimal transformations)
    Raw,
}

/// What to produce from the pipeline
#[derive(Debug, Clone, PartialEq)]
pub enum TargetSpec {
    /// Stop at tokens
    Tokens,
    /// Continue to AST with specified parser
    Ast { parser: ParserSpec },
}

/// Which parser to use
#[derive(Debug, Clone, PartialEq)]
pub enum ParserSpec {
    Reference,
    Linebased,
}

/// Registry of processing configurations
pub struct ConfigRegistry {
    configs: HashMap<String, ProcessingConfig>,
}

impl ConfigRegistry {
    pub fn new() -> Self {
        ConfigRegistry {
            configs: HashMap::new(),
        }
    }

    pub fn register(&mut self, config: ProcessingConfig) {
        self.configs.insert(config.name.clone(), config);
    }

    pub fn get(&self, name: &str) -> Option<&ProcessingConfig> {
        self.configs.get(name)
    }

    pub fn list_all(&self) -> Vec<&ProcessingConfig> {
        let mut configs: Vec<_> = self.configs.values().collect();
        configs.sort_by(|a, b| a.name.cmp(&b.name));
        configs
    }

    /// Create registry with standard configurations
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        registry.register(ProcessingConfig {
            name: "default".into(),
            description: "Stable: Indentation lexer + Reference parser".into(),
            pipeline_spec: PipelineSpec::Indentation,
            target: TargetSpec::Ast {
                parser: ParserSpec::Reference,
            },
        });

        registry.register(ProcessingConfig {
            name: "linebased".into(),
            description: "Experimental: Linebased lexer + Linebased parser".into(),
            pipeline_spec: PipelineSpec::Linebased,
            target: TargetSpec::Ast {
                parser: ParserSpec::Linebased,
            },
        });

        registry.register(ProcessingConfig {
            name: "tokens-indentation".into(),
            description: "Indentation transformations, output tokens".into(),
            pipeline_spec: PipelineSpec::Indentation,
            target: TargetSpec::Tokens,
        });

        registry.register(ProcessingConfig {
            name: "tokens-linebased-flat".into(),
            description: "Linebased up to LineTokens, output tokens".into(),
            pipeline_spec: PipelineSpec::LinebasedFlat,
            target: TargetSpec::Tokens,
        });

        registry.register(ProcessingConfig {
            name: "tokens-linebased-tree".into(),
            description: "Full linebased with tree, output tokens".into(),
            pipeline_spec: PipelineSpec::Linebased,
            target: TargetSpec::Tokens,
        });

        registry.register(ProcessingConfig {
            name: "tokens-raw".into(),
            description: "Raw tokens from base tokenization".into(),
            pipeline_spec: PipelineSpec::Raw,
            target: TargetSpec::Tokens,
        });

        registry
    }
}

impl Default for ConfigRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}
```

**Tests**: Add tests in same file:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = ConfigRegistry::new();
        assert_eq!(registry.configs.len(), 0);
    }

    #[test]
    fn test_registry_with_defaults() {
        let registry = ConfigRegistry::with_defaults();
        assert!(registry.get("default").is_some());
        assert!(registry.get("linebased").is_some());
        assert!(registry.get("tokens-indentation").is_some());
    }

    #[test]
    fn test_registry_get() {
        let registry = ConfigRegistry::with_defaults();
        let config = registry.get("default").unwrap();
        assert_eq!(config.name, "default");
        assert_eq!(config.pipeline_spec, PipelineSpec::Indentation);
    }

    #[test]
    fn test_registry_list_all() {
        let registry = ConfigRegistry::with_defaults();
        let configs = registry.list_all();
        assert!(configs.len() >= 5); // Should have at least 5 default configs
    }
}
```

**Update**: `src/lex/pipeline/mod.rs`
```rust
pub mod config;  // NEW

pub use config::{ConfigRegistry, ProcessingConfig, PipelineSpec, TargetSpec, ParserSpec};
```

**Validation**:
- ✅ Config module compiles
- ✅ All tests pass
- ✅ No existing code uses it yet (zero impact)

**Estimated effort**: 2 hours

### Step 2.2: Create Pipeline Executor

**File**: `src/lex/pipeline/executor.rs` (NEW)

**Changes**:
```rust
//! Pipeline executor that runs processing configurations

use crate::lex::lexers::base_tokenization;
use crate::lex::parsers::Document;
use crate::lex::pipeline::config::{ConfigRegistry, ParserSpec, PipelineSpec, TargetSpec};
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
            ExecutionError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
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
    pub fn new() -> Self {
        Self {
            registry: ConfigRegistry::with_defaults(),
        }
    }

    pub fn with_registry(registry: ConfigRegistry) -> Self {
        Self { registry }
    }

    /// Execute a named configuration
    pub fn execute(&self, config_name: &str, source: &str) -> Result<ExecutionOutput, ExecutionError> {
        let config = self.registry.get(config_name)
            .ok_or_else(|| ExecutionError::ConfigNotFound(config_name.to_string()))?;

        // Step 1: Base tokenization
        let source_with_newline = crate::lex::lexers::ensure_source_ends_with_newline(source);
        let tokens = base_tokenization::tokenize(&source_with_newline);
        let mut stream = TokenStream::Flat(tokens);

        // Step 2: Apply transformations
        stream = self.apply_transformations(stream, &config.pipeline_spec)?;

        // Step 3: Process target
        match &config.target {
            TargetSpec::Tokens => Ok(ExecutionOutput::Tokens(stream)),
            TargetSpec::Ast { parser } => {
                let doc = self.parse(stream, source, parser)?;
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
                stream = indent_mapper.transform(stream)
                    .map_err(|e| ExecutionError::TransformationFailed(e.to_string()))?;

                Ok(stream)
            }
        }
    }

    fn parse(
        &self,
        stream: TokenStream,
        source: &str,
        parser: &ParserSpec,
    ) -> Result<Document, ExecutionError> {
        match parser {
            ParserSpec::Reference => {
                let tokens = stream.unroll();
                crate::lex::parsers::reference::parse(tokens, source)
                    .map_err(|_| ExecutionError::ParsingFailed("Reference parser failed".to_string()))
            }
            ParserSpec::Linebased => {
                let container = crate::lex::pipeline::adapters_linebased::token_stream_to_line_container(stream)
                    .map_err(|e| ExecutionError::ParsingFailed(format!("Stream conversion failed: {:?}", e)))?;
                crate::lex::parsers::linebased::parse_experimental_v2(container, source)
                    .map_err(|e| ExecutionError::ParsingFailed(format!("Linebased parser failed: {}", e)))
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
```

**Tests**: Add comprehensive tests:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let executor = PipelineExecutor::new();
        assert!(executor.list_configs().len() > 0);
    }

    #[test]
    fn test_execute_default_config() {
        let executor = PipelineExecutor::new();
        let result = executor.execute("default", "Hello world");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), ExecutionOutput::Document(_)));
    }

    #[test]
    fn test_execute_tokens_config() {
        let executor = PipelineExecutor::new();
        let result = executor.execute("tokens-indentation", "Hello world");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), ExecutionOutput::Tokens(_)));
    }

    #[test]
    fn test_execute_linebased_config() {
        let executor = PipelineExecutor::new();
        let result = executor.execute("linebased", "Hello:\n    World");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), ExecutionOutput::Document(_)));
    }

    #[test]
    fn test_execute_nonexistent_config() {
        let executor = PipelineExecutor::new();
        let result = executor.execute("nonexistent", "Hello");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExecutionError::ConfigNotFound(_)));
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
}
```

**Update**: `src/lex/pipeline/mod.rs`
```rust
pub mod executor;  // NEW

pub use executor::{PipelineExecutor, ExecutionOutput, ExecutionError};
```

**Validation**:
- ✅ Executor works end-to-end
- ✅ Can execute all default configs
- ✅ Tests demonstrate functionality
- ✅ Still no impact on existing code

**Estimated effort**: 3-4 hours

---

## Phase 3: Add Integration Tests (Validate New System)

**Goal**: Prove the new system works correctly and matches existing behavior.

**Status**: Tests only, no production code changes

### Step 3.1: Comparison Tests

**File**: `tests/integration/pipeline_executor_tests.rs` (NEW)

**Changes**:
```rust
//! Integration tests for PipelineExecutor
//! These tests validate that the new executor produces correct results

use lex::lex::pipeline::{PipelineExecutor, ExecutionOutput};
use lex::lex::processor::lex_sources::LexSources;

#[test]
fn test_executor_against_sample_files() {
    let executor = PipelineExecutor::new();

    // Test with multiple sample files
    for sample in &["000-paragraphs.lex", "040-lists.lex", "090-definitions-simple.lex"] {
        let source = LexSources::get_string(sample).unwrap();

        // Should successfully parse with default config
        let result = executor.execute("default", &source);
        assert!(result.is_ok(), "Failed to parse {}", sample);

        match result.unwrap() {
            ExecutionOutput::Document(doc) => {
                assert!(!doc.root.content.is_empty(), "Document should have content for {}", sample);
            }
            _ => panic!("Expected document output"),
        }
    }
}

#[test]
fn test_executor_vs_existing_pipeline() {
    // Compare new executor with existing LexPipeline
    let executor = PipelineExecutor::new();
    let old_pipeline = lex::lex::pipeline::LexPipeline::default();

    let source = "Hello:\n    World\n";

    // Execute with new system
    let new_result = executor.execute("default", source).unwrap();
    let new_doc = match new_result {
        ExecutionOutput::Document(doc) => doc,
        _ => panic!("Expected document"),
    };

    // Execute with old system
    let old_doc = old_pipeline.parse(source).unwrap();

    // Compare ASTs (should be identical)
    assert_eq!(
        new_doc.root.content.len(),
        old_doc.root.content.len(),
        "ASTs should have same structure"
    );
}

#[test]
fn test_executor_token_output() {
    let executor = PipelineExecutor::new();
    let source = "Hello world";

    // Test tokens-indentation config
    let result = executor.execute("tokens-indentation", source).unwrap();
    let stream = match result {
        ExecutionOutput::Tokens(s) => s,
        _ => panic!("Expected tokens"),
    };

    let tokens = stream.unroll();
    assert!(tokens.len() > 0, "Should have tokens");
}

#[test]
fn test_all_default_configs_work() {
    let executor = PipelineExecutor::new();
    let source = "Test:\n    Content\n";

    // Every default config should work without error
    for config in executor.list_configs() {
        let result = executor.execute(&config.name, source);
        assert!(
            result.is_ok(),
            "Config '{}' failed: {:?}",
            config.name,
            result.err()
        );
    }
}
```

**Validation**:
- ✅ New system produces correct results
- ✅ Matches existing system behavior
- ✅ All configs work on real sample files

**Estimated effort**: 2 hours

---

## Phase 4: Migrate processor.rs (Start Using New System)

**Goal**: Make `processor.rs` use the new executor internally while keeping external API unchanged.

**Status**: Internal refactor, external API unchanged

### Step 4.1: Update processor.rs to use PipelineExecutor

**File**: `src/lex/processor.rs`

**Changes**:
```rust
// MODIFY process_file_with_extras function
pub fn process_file_with_extras<P: AsRef<Path>>(
    file_path: P,
    spec: &ProcessingSpec,
    extras: HashMap<String, String>,
) -> Result<String, ProcessingError> {
    let file_path = file_path.as_ref();
    let content = fs::read_to_string(file_path)
        .map_err(|e| ProcessingError::IoError(e.to_string()))?;

    match &spec.format {
        // Special formats still handled directly
        OutputFormat::TokenLine => { /* existing code */ }
        OutputFormat::TokenTree => { /* existing code */ }

        // NEW: Use executor for standard processing
        _ => {
            use crate::lex::pipeline::{PipelineExecutor, ExecutionOutput};

            let executor = PipelineExecutor::new();

            // Map spec to config name
            let config_name = match (&spec.stage, &spec.format) {
                (ProcessingStage::Token, OutputFormat::Simple) => "tokens-indentation",
                (ProcessingStage::Token, OutputFormat::Json) => "tokens-indentation",
                (ProcessingStage::Ast, OutputFormat::AstTag) => "default",
                (ProcessingStage::Ast, OutputFormat::AstTreeviz) => "default",
                (ProcessingStage::Ast, OutputFormat::AstLinebasedTag) => "linebased",
                (ProcessingStage::Ast, OutputFormat::AstLinebasedTreeviz) => "linebased",
                _ => return Err(ProcessingError::InvalidFormatType(format!("{:?}", spec.format))),
            };

            // Execute using new system
            let output = executor.execute(config_name, &content)
                .map_err(|e| ProcessingError::IoError(e.to_string()))?;

            // Format output
            match (output, &spec.format) {
                (ExecutionOutput::Tokens(stream), OutputFormat::Simple) => {
                    format_tokenss(&stream.unroll(), &spec.format)
                }
                (ExecutionOutput::Tokens(stream), OutputFormat::Json) => {
                    format_tokenss(&stream.unroll(), &spec.format)
                }
                (ExecutionOutput::Document(doc), OutputFormat::AstTag) => {
                    Ok(crate::lex::parsers::serialize_ast_tag(&doc))
                }
                (ExecutionOutput::Document(doc), OutputFormat::AstTreeviz) => {
                    Ok(crate::lex::parsers::to_treeviz_str(&doc))
                }
                (ExecutionOutput::Document(doc), OutputFormat::AstLinebasedTag) => {
                    Ok(crate::lex::parsers::serialize_ast_tag(&doc))
                }
                (ExecutionOutput::Document(doc), OutputFormat::AstLinebasedTreeviz) => {
                    Ok(crate::lex::parsers::to_treeviz_str(&doc))
                }
                _ => Err(ProcessingError::InvalidFormatType("Mismatched output".into())),
            }
        }
    }
}
```

**Tests**: Ensure all existing processor tests still pass:
```bash
cargo test processor::tests
```

**Validation**:
- ✅ All existing processor tests pass
- ✅ processor.rs now uses executor internally
- ✅ External API unchanged (backwards compatible)

**Estimated effort**: 2-3 hours

---

## Phase 5: Update Binary (User-Facing)

**Goal**: Update the `lex` binary to expose the new config-based interface.

**Status**: User-visible change, but additive

### Step 5.1: Add new subcommands to binary

**File**: `src/bin/lex.rs`

**Changes**:
```rust
// ADD new subcommand
.subcommand(
    Command::new("execute")
        .about("Execute a processing configuration")
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .help("Configuration name (e.g., 'default', 'linebased')")
                .required(true)
        )
        .arg(
            Arg::new("path")
                .help("Path to lex file")
                .required(true)
                .index(1)
        )
        .arg(
            Arg::new("format")
                .long("format")
                .short('f')
                .help("Output format (token-json, ast-tag, etc.)")
                .default_value("ast-tag")
        )
)
.subcommand(
    Command::new("list-configs")
        .about("List available processing configurations")
)

// ADD handler
Some(("execute", m)) => {
    let config = m.get_one::<String>("config").unwrap();
    let path = m.get_one::<String>("path").unwrap();
    let format = m.get_one::<String>("format").unwrap();

    handle_execute_command(config, path, format);
}
Some(("list-configs", _)) => {
    handle_list_configs_command();
}

// ADD new handlers
fn handle_execute_command(config: &str, path: &str, format: &str) {
    use lex::lex::pipeline::{PipelineExecutor, ExecutionOutput};

    let source = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Error reading file: {}", e);
        std::process::exit(1);
    });

    let executor = PipelineExecutor::new();
    let output = executor.execute(config, &source).unwrap_or_else(|e| {
        eprintln!("Execution error: {}", e);
        std::process::exit(1);
    });

    // Format and print output
    match (output, format) {
        (ExecutionOutput::Document(doc), "ast-tag") => {
            println!("{}", lex::lex::parsers::serialize_ast_tag(&doc));
        }
        (ExecutionOutput::Document(doc), "ast-treeviz") => {
            println!("{}", lex::lex::parsers::to_treeviz_str(&doc));
        }
        (ExecutionOutput::Tokens(stream), "token-json") => {
            let tokens = stream.unroll();
            println!("{}", serde_json::to_string_pretty(&tokens).unwrap());
        }
        _ => {
            eprintln!("Unsupported format: {}", format);
            std::process::exit(1);
        }
    }
}

fn handle_list_configs_command() {
    use lex::lex::pipeline::PipelineExecutor;

    let executor = PipelineExecutor::new();
    println!("Available configurations:\n");

    for config in executor.list_configs() {
        println!("  {}", config.name);
        println!("    {}", config.description);
        println!();
    }
}
```

**Documentation**: Update CLI help:
```bash
# New usage examples:
lex execute --config default document.lex
lex execute --config linebased document.lex --format ast-treeviz
lex execute --config tokens-indentation document.lex --format token-json
lex list-configs
```

**Validation**:
- ✅ Old commands still work (backwards compatible)
- ✅ New commands provide cleaner interface
- ✅ Help text is clear

**Estimated effort**: 2 hours

---

## Phase 6: Deprecation (Mark Old Systems)

**Goal**: Mark the old registry-based systems as deprecated.

**Status**: Warnings only, code still works

### Step 6.1: Add deprecation warnings

**Files to mark deprecated**:
- `src/lex/lexers/common/interface.rs` - LexerRegistry
- `src/lex/parsers/common/interface.rs` - ParserRegistry
- `src/lex/pipeline/orchestration.rs` - LexPipeline

**Changes**:
```rust
#[deprecated(
    since = "0.2.0",
    note = "Use PipelineExecutor with ProcessingConfig instead"
)]
pub struct LexerRegistry { /* ... */ }

#[deprecated(
    since = "0.2.0",
    note = "Use PipelineExecutor with ProcessingConfig instead"
)]
pub struct ParserRegistry { /* ... */ }

#[deprecated(
    since = "0.2.0",
    note = "Use PipelineExecutor with ProcessingConfig instead"
)]
pub struct LexPipeline { /* ... */ }
```

**Documentation**: Add migration guide:
```markdown
# Migration Guide: LexPipeline → PipelineExecutor

## Old way:
```rust
let pipeline = LexPipeline::new("indentation", "reference");
let doc = pipeline.parse(source)?;
```

## New way:
```rust
let executor = PipelineExecutor::new();
let output = executor.execute("default", source)?;
let doc = match output {
    ExecutionOutput::Document(d) => d,
    _ => unreachable!(),
};
```
```

**Validation**:
- ✅ Deprecation warnings appear
- ✅ Code still compiles and works
- ✅ Migration path is clear

**Estimated effort**: 1 hour

---

## Phase 7: Cleanup (Remove Old Code)

**Goal**: Remove deprecated systems after migration period.

**Status**: Breaking change (but with deprecation warnings)

### Step 7.1: Remove old registry systems

**Files to remove**:
- `src/lex/lexers/common/interface.rs` (entire file)
- `src/lex/parsers/common/interface.rs` (entire file)
- `src/lex/pipeline/orchestration.rs` (entire file)

**Files to update**:
- `src/lex/lexers/common/mod.rs` - Remove interface re-exports
- `src/lex/parsers/common/mod.rs` - Remove interface re-exports
- `src/lex/pipeline/mod.rs` - Remove orchestration re-exports

**Update processor.rs**: Simplify now that we only use executor

**Validation**:
- ✅ Code compiles without old systems
- ✅ All tests pass
- ✅ Binary works correctly
- ✅ ~700 lines of code removed

**Estimated effort**: 2 hours

---

## Summary Timeline

| Phase | Description | Breaking? | Effort | Dependencies |
|-------|-------------|-----------|--------|--------------|
| **1** | Extend Pipeline with parsing | ❌ No | 2-3h | None |
| **2.1** | Create ProcessingConfig | ❌ No | 2h | Phase 1 |
| **2.2** | Create PipelineExecutor | ❌ No | 3-4h | Phase 2.1 |
| **3** | Integration tests | ❌ No | 2h | Phase 2.2 |
| **4** | Migrate processor.rs | ❌ No | 2-3h | Phase 3 |
| **5** | Update binary | ⚠️ Additive | 2h | Phase 4 |
| **6** | Add deprecations | ⚠️ Warnings | 1h | Phase 5 |
| **7** | Remove old code | ✅ Yes | 2h | Phase 6 + wait period |

**Total estimated effort**: 16-19 hours of focused work

**Total elapsed time**: 2-3 weeks (allowing for review, testing, and deprecation period)

---

## Alternative: Even Smaller Steps

If you want even more incremental steps, here's a variant:

### Ultra-Incremental Approach

**Phase 1a**: Add parser support to Pipeline (2h)
**Phase 1b**: Add tests for Pipeline with parser (1h)

**Phase 2a**: Create just the config data structures (1h)
**Phase 2b**: Create ConfigRegistry with defaults (1h)
**Phase 2c**: Add tests for ConfigRegistry (1h)

**Phase 3a**: Create basic PipelineExecutor (without all transformations) (2h)
**Phase 3b**: Add all transformation implementations (2h)
**Phase 3c**: Add parser integration (1h)
**Phase 3d**: Add tests for PipelineExecutor (1h)

**Phase 4**: Integration tests (2h)
**Phase 5**: Migrate processor.rs (2-3h)
**Phase 6**: Update binary (2h)
**Phase 7**: Deprecate (1h)
**Phase 8**: Remove (2h)

This breaks it down into ~1-2 hour chunks for even easier review cycles.

---

## Rollback Plan

At each phase, if issues arise:

- **Phase 1-3**: Simply don't use the new code (zero impact)
- **Phase 4**: Revert processor.rs changes (old path still exists)
- **Phase 5**: Old binary commands still work
- **Phase 6**: Deprecation is just warnings
- **Phase 7**: This is the only risky step, but by then system is well-tested

---

## Success Criteria

After completion:
- ✅ Single entry point (PipelineExecutor)
- ✅ Config-based processing selection
- ✅ ~700 lines of duplicate code removed
- ✅ All tests passing
- ✅ Binary exposes clean interface
- ✅ processor.rs simplified
- ✅ Clear migration path documented

---

## Questions?

1. **Which approach do you prefer**: Standard phased or ultra-incremental?
2. **Deprecation period**: How long before Phase 7 (removal)?
3. **Binary interface**: Keep old commands working alongside new?
4. **Testing coverage**: Any specific edge cases to focus on?
