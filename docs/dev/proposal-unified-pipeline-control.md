# Proposal: Unified Pipeline Control for Lex Processing

## Executive Summary

This document proposes a unified, simplified architecture for controlling Lex document processing. The goal is to eliminate redundant abstractions while preserving the excellent pipeline infrastructure and providing a clear, intuitive interface for specifying lexing and parsing operations.

## Current State Analysis

### What Works Well ✅

1. **Pipeline Infrastructure** (`src/lex/pipeline/`)
   - `TokenStream` abstraction with Flat/Tree variants
   - `StreamMapper` trait for chainable transformations
   - `walk_stream()` for recursive traversal
   - Clean separation of transformation logic

2. **Transformation Implementations**
   - Well-defined mappers: NormalizeWhitespace, SemanticIndentation, BlankLines, ToLineTokens, IndentationToTree
   - Each mapper focuses on one concern
   - Composable via the pipeline builder

3. **AST Building** (`src/lex/parsers/ast/`)
   - Shared by both parsers
   - Three-layer architecture (normalize → extract → create)
   - Consistent location tracking

### Current Problems ❌

1. **Three Overlapping Control Mechanisms**
   - `LexerRegistry` + `ParserRegistry` (src/lex/lexers/common/interface.rs, src/lex/parsers/common/interface.rs)
   - `LexPipeline` (src/lex/pipeline/orchestration.rs)
   - `processor` module (src/lex/processor.rs)

2. **Conceptual Confusion**
   - Registries handle names → implementations
   - LexPipeline also handles names → implementations (duplicating registries)
   - Processor handles format strings and output formatting
   - No clear single source of truth for "what processing to do"

3. **Limited Pipeline Expressiveness**
   - Can't easily specify: "Run these specific transformations, then stop at tokens"
   - Can't say: "Use linebased transformations but only up to LineTokens stage"
   - Pipeline builder exists but isn't integrated with naming/selection

4. **Format String Complexity** (processor.rs)
   - Mixes stage (token/ast), format (json/simple), and pipeline (linebased) concerns
   - Examples: "token-simple", "token-line", "ast-linebased-tag"
   - Hard to extend for new transformation combinations

## Proposed Solution

### Core Concept: Named Processing Configurations

Replace the three overlapping systems with a single **Processing Configuration** concept that unifies:
- Which transformations to run (lexing pipeline)
- How far to go (stop at tokens, or continue to AST)
- Which parser to use (if going to AST)
- Output format (for serialization)

### Architecture

```
Source Text
    ↓
[Base Tokenization] ← Always happens (logos)
    ↓
[Named Transformation Pipeline] ← Configurable sequence of StreamMappers
    ↓ produces TokenStream
    ├─→ [Stop: Output TokenStream] ← Lexing-only path
    │
    └─→ [Named Parser] → Document AST ← Full parsing path
            ↓
        [Output Formatter]
```

### Proposed Structure

```rust
// src/lex/pipeline/config.rs - NEW FILE

/// A named configuration that specifies what processing to perform
pub struct ProcessingConfig {
    /// Unique identifier for this configuration
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// The transformation pipeline to run
    pub transformation_pipeline: TransformationPipelineSpec,

    /// What to do after transformations
    pub target: ProcessingTarget,
}

/// Specifies which transformations to run
pub enum TransformationPipelineSpec {
    /// Standard indentation-based pipeline
    /// [NormalizeWhitespace, SemanticIndentation, BlankLines]
    Indentation,

    /// Linebased pipeline (all transformations)
    /// [NormalizeWhitespace, SemanticIndentation, BlankLines, ToLineTokens, IndentationToTree]
    Linebased,

    /// Linebased up to LineTokens (without IndentationToTree)
    LinebasedFlat,

    /// Custom pipeline (for advanced use)
    Custom(Vec<Box<dyn StreamMapper>>),
}

/// What to produce from the pipeline
pub enum ProcessingTarget {
    /// Stop after transformations, output TokenStream
    Tokens {
        /// At which stage to stop and output
        stage: TokenStage,
    },

    /// Continue to AST parsing
    Ast {
        /// Which parser to use
        parser: ParserSpec,
    },
}

/// Stage at which to output tokens (for debugging/testing)
pub enum TokenStage {
    /// After base tokenization (raw tokens)
    Raw,

    /// After NormalizeWhitespace
    Normalized,

    /// After SemanticIndentation (Indent/Dedent tokens)
    Semantic,

    /// After all transformations in the pipeline
    Final,
}

/// Which parser to use for AST generation
pub enum ParserSpec {
    /// Reference combinator parser (requires flat tokens)
    Reference,

    /// Linebased declarative grammar parser (requires LineContainer tree)
    Linebased,
}

/// Registry of named processing configurations
pub struct ConfigRegistry {
    configs: HashMap<String, ProcessingConfig>,
}

impl ConfigRegistry {
    /// Create registry with standard configurations
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // Standard stable configuration
        registry.register(ProcessingConfig {
            name: "default".into(),
            description: "Stable: Indentation lexer + Reference parser".into(),
            transformation_pipeline: TransformationPipelineSpec::Indentation,
            target: ProcessingTarget::Ast {
                parser: ParserSpec::Reference,
            },
        });

        // Linebased experimental configuration
        registry.register(ProcessingConfig {
            name: "linebased".into(),
            description: "Experimental: Linebased lexer + Linebased parser".into(),
            transformation_pipeline: TransformationPipelineSpec::Linebased,
            target: ProcessingTarget::Ast {
                parser: ParserSpec::Linebased,
            },
        });

        // Token-only configurations (for testing/debugging)
        registry.register(ProcessingConfig {
            name: "tokens-indentation".into(),
            description: "Indentation lexer, stop at tokens".into(),
            transformation_pipeline: TransformationPipelineSpec::Indentation,
            target: ProcessingTarget::Tokens {
                stage: TokenStage::Final,
            },
        });

        registry.register(ProcessingConfig {
            name: "tokens-linebased-flat".into(),
            description: "Linebased transformations up to LineTokens".into(),
            transformation_pipeline: TransformationPipelineSpec::LinebasedFlat,
            target: ProcessingTarget::Tokens {
                stage: TokenStage::Final,
            },
        });

        registry.register(ProcessingConfig {
            name: "tokens-linebased-tree".into(),
            description: "Linebased transformations including IndentationToTree".into(),
            transformation_pipeline: TransformationPipelineSpec::Linebased,
            target: ProcessingTarget::Tokens {
                stage: TokenStage::Final,
            },
        });

        registry
    }
}
```

### Main Execution API

```rust
// src/lex/pipeline/executor.rs - NEW FILE

/// Executes a processing configuration on source text
pub struct PipelineExecutor {
    config_registry: ConfigRegistry,
}

impl PipelineExecutor {
    pub fn new() -> Self {
        Self {
            config_registry: ConfigRegistry::with_defaults(),
        }
    }

    /// Execute a named configuration
    pub fn execute(&self, config_name: &str, source: &str) -> Result<ProcessingOutput, PipelineError> {
        let config = self.config_registry.get(config_name)
            .ok_or(PipelineError::ConfigNotFound(config_name.to_string()))?;

        // Step 1: Base tokenization
        let tokens = base_tokenization::tokenize(source);
        let mut stream = TokenStream::Flat(tokens);

        // Step 2: Apply transformation pipeline
        stream = self.apply_transformations(stream, &config.transformation_pipeline)?;

        // Step 3: Process according to target
        match &config.target {
            ProcessingTarget::Tokens { stage } => {
                // Return tokens at the requested stage
                Ok(ProcessingOutput::Tokens(stream))
            }
            ProcessingTarget::Ast { parser } => {
                // Continue to parsing
                let doc = self.parse(stream, source, parser)?;
                Ok(ProcessingOutput::Document(doc))
            }
        }
    }

    fn apply_transformations(
        &self,
        mut stream: TokenStream,
        spec: &TransformationPipelineSpec
    ) -> Result<TokenStream, PipelineError> {
        match spec {
            TransformationPipelineSpec::Indentation => {
                stream = walk_stream(stream, &mut NormalizeWhitespaceMapper::new())?;
                stream = walk_stream(stream, &mut SemanticIndentationMapper::new())?;
                stream = walk_stream(stream, &mut BlankLinesMapper::new())?;
                Ok(stream)
            }
            TransformationPipelineSpec::Linebased => {
                // All transformations including ToLineTokens and IndentationToTree
                stream = walk_stream(stream, &mut NormalizeWhitespaceMapper::new())?;
                stream = walk_stream(stream, &mut SemanticIndentationMapper::new())?;
                stream = walk_stream(stream, &mut BlankLinesMapper::new())?;
                stream = walk_stream(stream, &mut ToLineTokensMapper::new())?;

                let mut indent_mapper = IndentationToTreeMapper::new();
                stream = indent_mapper.transform(stream)?;

                Ok(stream)
            }
            TransformationPipelineSpec::LinebasedFlat => {
                // Up to LineTokens, without IndentationToTree
                stream = walk_stream(stream, &mut NormalizeWhitespaceMapper::new())?;
                stream = walk_stream(stream, &mut SemanticIndentationMapper::new())?;
                stream = walk_stream(stream, &mut BlankLinesMapper::new())?;
                stream = walk_stream(stream, &mut ToLineTokensMapper::new())?;
                Ok(stream)
            }
            TransformationPipelineSpec::Custom(mappers) => {
                for mapper in mappers {
                    stream = walk_stream(stream, mapper.as_ref())?;
                }
                Ok(stream)
            }
        }
    }

    fn parse(
        &self,
        stream: TokenStream,
        source: &str,
        parser: &ParserSpec,
    ) -> Result<Document, PipelineError> {
        match parser {
            ParserSpec::Reference => {
                // Reference parser needs flat tokens
                let tokens = stream.unroll();
                parsers::reference::parse(tokens, source)
                    .map_err(|_| PipelineError::ParsingFailed("Reference parser failed".into()))
            }
            ParserSpec::Linebased => {
                // Linebased parser needs LineContainer tree
                let container = adapters_linebased::token_stream_to_line_container(stream)
                    .map_err(|e| PipelineError::InvalidStream(format!("{:?}", e)))?;
                parsers::linebased::parse_experimental_v2(container, source)
                    .map_err(|e| PipelineError::ParsingFailed(e))
            }
        }
    }

    /// List all available configurations
    pub fn list_configs(&self) -> Vec<&ProcessingConfig> {
        self.config_registry.list_all()
    }
}

/// Output from pipeline execution
pub enum ProcessingOutput {
    /// Token stream output (for lexing-only)
    Tokens(TokenStream),

    /// Document AST output (for full parsing)
    Document(Document),
}
```

### Output Formatting (Separate Concern)

```rust
// src/lex/output/formatter.rs - NEW FILE

/// Handles formatting of processing output for display/serialization
pub struct OutputFormatter;

impl OutputFormatter {
    /// Format output according to format spec
    pub fn format(output: &ProcessingOutput, format: OutputFormat) -> Result<String, FormattingError> {
        match (output, format) {
            (ProcessingOutput::Tokens(stream), OutputFormat::TokenSimple) => {
                Self::format_tokens_simple(stream)
            }
            (ProcessingOutput::Tokens(stream), OutputFormat::TokenJson) => {
                Self::format_tokens_json(stream)
            }
            (ProcessingOutput::Document(doc), OutputFormat::AstTag) => {
                Ok(serialize_ast_tag(doc))
            }
            (ProcessingOutput::Document(doc), OutputFormat::AstTreeviz) => {
                Ok(to_treeviz_str(doc))
            }
            // ... other combinations
            _ => Err(FormattingError::IncompatibleFormat),
        }
    }
}

pub enum OutputFormat {
    // Token formats
    TokenSimple,
    TokenJson,

    // AST formats
    AstTag,
    AstTreeviz,
    AstJson,
}
```

### Updated Binary Interface

```rust
// src/bin/lex.rs - SIMPLIFIED

fn main() {
    let matches = Command::new("lex")
        .subcommand(
            Command::new("process")
                .arg(Arg::new("config").long("config").default_value("default"))
                .arg(Arg::new("format").long("format").default_value("ast-tag"))
                .arg(Arg::new("path"))
        )
        .subcommand(
            Command::new("list-configs")
        )
        .get_matches();

    match matches.subcommand() {
        Some(("process", m)) => {
            let config = m.get_one::<String>("config").unwrap();
            let format = m.get_one::<String>("format").unwrap();
            let path = m.get_one::<String>("path").unwrap();

            let executor = PipelineExecutor::new();
            let source = fs::read_to_string(path)?;

            let output = executor.execute(config, &source)?;
            let formatted = OutputFormatter::format(&output, format.parse()?)?;

            println!("{}", formatted);
        }
        Some(("list-configs", _)) => {
            let executor = PipelineExecutor::new();
            for config in executor.list_configs() {
                println!("{}: {}", config.name, config.description);
            }
        }
        _ => {}
    }
}
```

### Usage Examples

```bash
# Default stable pipeline (indentation lexer + reference parser, AST output)
lex process document.lex

# Linebased experimental pipeline
lex process document.lex --config linebased

# Get tokens only (indentation pipeline)
lex process document.lex --config tokens-indentation --format token-json

# Get linebased LineTokens (for debugging)
lex process document.lex --config tokens-linebased-flat --format token-json

# Get linebased tree tokens
lex process document.lex --config tokens-linebased-tree --format token-json

# List all available configurations
lex list-configs
```

### In Test Code

```rust
#[test]
fn test_compare_parsers() {
    let source = "Hello:\n    World\n";
    let executor = PipelineExecutor::new();

    // Get AST from reference parser
    let ref_output = executor.execute("default", source).unwrap();
    let ref_doc = match ref_output {
        ProcessingOutput::Document(doc) => doc,
        _ => panic!("Expected document"),
    };

    // Get AST from linebased parser
    let lb_output = executor.execute("linebased", source).unwrap();
    let lb_doc = match lb_output {
        ProcessingOutput::Document(doc) => doc,
        _ => panic!("Expected document"),
    };

    // Compare ASTs
    assert_ast_equivalent(&ref_doc, &lb_doc);
}

#[test]
fn test_token_output() {
    let source = "Hello world";
    let executor = PipelineExecutor::new();

    // Get tokens from indentation pipeline
    let output = executor.execute("tokens-indentation", source).unwrap();
    let tokens = match output {
        ProcessingOutput::Tokens(stream) => stream.unroll(),
        _ => panic!("Expected tokens"),
    };

    assert_eq!(tokens.len(), 3); // Text("Hello"), Whitespace, Text("world")
}
```

## Migration Path

### Phase 1: Add New System (Non-Breaking)
1. Create `src/lex/pipeline/config.rs` with ProcessingConfig
2. Create `src/lex/pipeline/executor.rs` with PipelineExecutor
3. Create `src/lex/output/formatter.rs` for output formatting
4. Add new system alongside existing code
5. Update tests to use new API

### Phase 2: Deprecate Old Systems
1. Mark LexerRegistry, ParserRegistry, LexPipeline as `#[deprecated]`
2. Update processor.rs to use new executor internally
3. Update binary to use new API
4. Add deprecation warnings

### Phase 3: Remove Old Code
1. Remove `src/lex/lexers/common/interface.rs`
2. Remove `src/lex/parsers/common/interface.rs`
3. Remove `src/lex/pipeline/orchestration.rs`
4. Simplify `src/lex/processor.rs` to just format string parsing

## Benefits

### Clarity
- **Single Concept**: "Processing Configuration" instead of "Lexer + Parser + Pipeline + Format"
- **Clear Naming**: "default", "linebased", "tokens-indentation" instead of mixing concerns
- **Obvious Intent**: Config name tells you what you're getting

### Flexibility
- **Easy Debugging**: Named configs for stopping at specific transformation stages
- **Testing**: Simple to compare different pipeline/parser combinations
- **Extension**: Add new configs without modifying infrastructure

### Simplicity
- **Less Code**: Removes ~1000 lines of duplicated registry/orchestration logic
- **Fewer Abstractions**: One registry instead of three
- **Clearer Flow**: Source → Config → Execute → Format

### Maintainability
- **Single Source of Truth**: ConfigRegistry owns all valid combinations
- **Type Safety**: Enum-based specs instead of string parsing
- **Self-Documenting**: Config descriptions explain what each does

## Questions for Discussion

1. **Config Naming Convention**: Should we use "default", "stable", or "indentation-reference"?

2. **Custom Pipelines**: Should we support custom transformation sequences in configs, or keep that as advanced API only?

3. **Output Format Coupling**: Should output format be part of ProcessingConfig, or always separate (current proposal)?

4. **Backward Compatibility**: Timeline for deprecation? Support both systems for how long?

5. **Performance**: Should configs be lazily constructed or prebuilt? Cache pipeline instances?

## Conclusion

This proposal simplifies Lex processing control by:
- Eliminating three overlapping systems (registries + LexPipeline + processor)
- Introducing a single unified concept (ProcessingConfig)
- Separating concerns (transformation pipeline vs. output formatting)
- Providing clear, intuitive naming for common operations
- Maintaining flexibility for testing and debugging

The result is a clearer, more maintainable codebase that's easier to use and extend.
