# Unify and Simplify Lex Pipeline Control

## Problem

Currently we have three overlapping systems for controlling Lex processing:
- `LexerRegistry` + `ParserRegistry` (in `interface.rs` files)
- `LexPipeline` (in `orchestration.rs`)
- `processor` module with format strings

This causes:
- Confusion about which system to use
- Duplicated code (~1500 lines across registries + orchestration)
- Can't easily express "run these transformations, then stop at tokens"
- Format strings mix multiple concerns (e.g., "ast-linebased-tag")

## Proposal

Replace three systems with **Named Processing Configurations** that specify:
1. Which transformation pipeline to run (e.g., Indentation, Linebased)
2. Whether to stop at tokens or continue to AST
3. Which parser to use (if continuing to AST)

### Core Types

```rust
// Configuration that specifies what processing to do
struct ProcessingConfig {
    name: String,
    pipeline_spec: PipelineSpec,    // Which transformations
    target: TargetSpec,              // Tokens or Ast+Parser
}

enum PipelineSpec {
    Indentation,      // Standard transformations
    Linebased,        // Full linebased with tree
    LinebasedFlat,    // Linebased up to LineTokens
    Raw,              // Minimal transformations
}

enum TargetSpec {
    Tokens,
    Ast { parser: ParserSpec },
}

enum ParserSpec {
    Reference,
    Linebased,
}

// Executor runs configs by name
struct PipelineExecutor {
    registry: ConfigRegistry,
}

impl PipelineExecutor {
    fn execute(&self, config_name: &str, source: &str) -> ExecutionOutput { ... }
}
```

### Standard Configs

- `default` - Indentation lexer + Reference parser
- `linebased` - Linebased lexer + Linebased parser
- `tokens-indentation` - Indentation transformations, output tokens
- `tokens-linebased-flat` - Linebased up to LineTokens
- `tokens-linebased-tree` - Full linebased with tree

### Usage

```rust
// Single entry point
let executor = PipelineExecutor::new();
let output = executor.execute("linebased", source)?;

// For debugging tokens
let output = executor.execute("tokens-indentation", source)?;
```

```bash
# CLI
lex execute --config default document.lex
lex execute --config linebased document.lex
lex list-configs
```

## Implementation Plan

### Phase 1: Extend Pipeline to Support Parsing (2-3h)
- Add parser support to existing `Pipeline` builder
- Pipeline can now output tokens OR document
- Non-breaking addition

### Phase 2: Introduce ProcessingConfig (5-6h)
- **2.1**: Create config data structures (`config.rs`)
- **2.2**: Create `PipelineExecutor` (`executor.rs`)
- Runs completely parallel to existing systems
- Zero impact on production code

### Phase 3: Integration Tests (2h)
- Validate new system works correctly
- Compare against existing behavior
- Tests only

### Phase 4: Migrate processor.rs (2-3h)
- Use `PipelineExecutor` internally
- External API unchanged
- Backwards compatible

### Phase 5: Update Binary (2h)
- Add `execute` and `list-configs` subcommands
- Old commands still work
- Additive change

### Phase 6: Add Deprecation Warnings (1h)
- Mark `LexerRegistry`, `ParserRegistry`, `LexPipeline` as `#[deprecated]`
- Add migration guide
- Code still works

### Phase 7: Remove Old Code (2h)
- Delete `interface.rs` files and `orchestration.rs`
- ~700 lines removed
- Breaking change (after deprecation period)

## Result

- Single clear entry point (`PipelineExecutor`)
- Named configs are self-documenting
- ~700 lines of duplicate code removed
- Easy to test different pipelines/parsers
- Separation: transformation logic vs output formatting

## Timeline

**Total effort**: 16-19 hours focused work
**Elapsed time**: 2-3 weeks (includes review, testing, deprecation period)
