# Unified Pipeline Architecture - Visual Reference

## Current State (Complex)

```
                    ┌─────────────────────────────────┐
                    │      User / Test / Binary       │
                    └────────────┬────────────────────┘
                                 │
                    ┌────────────┴────────────┐
                    │   Which system to use?  │
                    └────┬────────┬────────┬──┘
                         │        │        │
        ┌────────────────┘        │        └────────────────┐
        │                         │                         │
        ▼                         ▼                         ▼
┌───────────────┐         ┌──────────────┐        ┌────────────────┐
│  LexPipeline  │         │  Registries  │        │   processor    │
│ (orchestration│         │  - LexerReg  │        │  (format str)  │
│     .rs)      │         │  - ParserReg │        │                │
└───────┬───────┘         └──────┬───────┘        └────────┬───────┘
        │                        │                          │
        │ ┌──────────────────────┘                          │
        │ │                                                 │
        ▼ ▼                                                 ▼
┌─────────────────────────────────────────┐      ┌──────────────────┐
│       Pipeline Infrastructure           │      │  ProcessingSpec  │
│  - TokenStream                          │      │  (token-simple)  │
│  - StreamMapper                         │      │  (ast-tag)       │
│  - Transformations                      │      │  (token-line)    │
└─────────────────────────────────────────┘      └──────────────────┘

Problems:
✗ Three systems doing similar jobs
✗ Unclear which to use when
✗ Duplication between LexPipeline and Registries
✗ Format strings mix multiple concerns
✗ Can't express "run transformations X, Y, Z then stop"
```

## Proposed State (Unified)

```
                    ┌─────────────────────────────────┐
                    │      User / Test / Binary       │
                    └────────────┬────────────────────┘
                                 │
                                 │ "What config?"
                                 │ (e.g., "default",
                                 │  "linebased",
                                 │  "tokens-indentation")
                                 │
                                 ▼
                    ┌─────────────────────────────────┐
                    │      PipelineExecutor           │
                    │  - Single entry point           │
                    │  - Owns ConfigRegistry          │
                    └────────────┬────────────────────┘
                                 │
                                 │ Looks up config
                                 │
                    ┌────────────▼────────────────────┐
                    │     ProcessingConfig            │
                    │  ┌────────────────────────────┐ │
                    │  │ TransformationPipelineSpec │ │
                    │  │ (which transformations)    │ │
                    │  └────────────────────────────┘ │
                    │  ┌────────────────────────────┐ │
                    │  │ ProcessingTarget           │ │
                    │  │ (Tokens or Ast+Parser)     │ │
                    │  └────────────────────────────┘ │
                    └────────────┬────────────────────┘
                                 │
                    ┌────────────▼────────────────────┐
                    │  Pipeline Infrastructure        │
                    │  - TokenStream                  │
                    │  - StreamMapper                 │
                    │  - Transformations (unchanged)  │
                    └────────────┬────────────────────┘
                                 │
                    ┌────────────▼────────────────────┐
                    │    ProcessingOutput             │
                    │  - Tokens(TokenStream) OR       │
                    │  - Document(AST)                │
                    └────────────┬────────────────────┘
                                 │
                                 │ (Optional)
                                 ▼
                    ┌─────────────────────────────────┐
                    │     OutputFormatter             │
                    │  - Formats output for display   │
                    │  - JSON, XML, simple, etc.      │
                    └─────────────────────────────────┘

Benefits:
✓ Single clear entry point
✓ Named configurations are self-documenting
✓ Separation: pipeline logic vs. output formatting
✓ Easy to add new configs without changing code
```

## Processing Flow Detail

```
Input: Source String + Config Name
    │
    ▼
┌───────────────────────────────────────────────────────────────┐
│ PipelineExecutor.execute(config_name, source)                 │
└───────────────┬───────────────────────────────────────────────┘
                │
                ▼
        ┌───────────────┐
        │ Lookup Config │
        └───────┬───────┘
                │
                ▼
        ┌─────────────────────────────────────────────────┐
        │ ProcessingConfig {                              │
        │   name: "linebased",                            │
        │   transformation_pipeline: Linebased,           │
        │   target: Ast { parser: Linebased }            │
        │ }                                               │
        └───────┬─────────────────────────────────────────┘
                │
                ▼
        ┌───────────────────────┐
        │ Base Tokenization     │
        │ (logos)               │
        │ String → Vec<(Token,  │
        │            Range)>    │
        └───────┬───────────────┘
                │
                ▼
        ┌───────────────────────┐
        │ Wrap in TokenStream   │
        │ TokenStream::Flat     │
        └───────┬───────────────┘
                │
                ▼
        ┌───────────────────────────────────────────────┐
        │ Apply Transformation Pipeline                 │
        │ (based on TransformationPipelineSpec)         │
        │                                               │
        │ For "Linebased":                              │
        │   → NormalizeWhitespace                       │
        │   → SemanticIndentation                       │
        │   → BlankLines                                │
        │   → ToLineTokens                              │
        │   → IndentationToTree                         │
        └───────┬───────────────────────────────────────┘
                │
                ▼
        TokenStream (Flat or Tree)
                │
                │ Branch on ProcessingTarget
                │
    ┌───────────┴───────────┐
    │                       │
    ▼                       ▼
┌─────────────┐      ┌─────────────────┐
│ Stop: Return│      │ Continue:       │
│ Tokens      │      │ Parse to AST    │
│             │      │                 │
│ Output:     │      │ Based on        │
│ Tokens(     │      │ ParserSpec:     │
│   stream)   │      │  - Reference or │
│             │      │  - Linebased    │
└─────────────┘      │                 │
                     │ Output:         │
                     │ Document(ast)   │
                     └─────────────────┘
```

## Configuration Examples

### Standard Configurations

```
┌──────────────────────────────────────────────────────────────┐
│ Config Name: "default"                                       │
├──────────────────────────────────────────────────────────────┤
│ Description: Stable indentation lexer + reference parser    │
├──────────────────────────────────────────────────────────────┤
│ TransformationPipeline: Indentation                         │
│   - NormalizeWhitespace                                     │
│   - SemanticIndentation                                     │
│   - BlankLines                                              │
├──────────────────────────────────────────────────────────────┤
│ Target: Ast { parser: Reference }                           │
├──────────────────────────────────────────────────────────────┤
│ Output: Document (AST)                                       │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│ Config Name: "linebased"                                     │
├──────────────────────────────────────────────────────────────┤
│ Description: Experimental linebased lexer + parser          │
├──────────────────────────────────────────────────────────────┤
│ TransformationPipeline: Linebased                           │
│   - NormalizeWhitespace                                     │
│   - SemanticIndentation                                     │
│   - BlankLines                                              │
│   - ToLineTokens                                            │
│   - IndentationToTree                                       │
├──────────────────────────────────────────────────────────────┤
│ Target: Ast { parser: Linebased }                           │
├──────────────────────────────────────────────────────────────┤
│ Output: Document (AST)                                       │
└──────────────────────────────────────────────────────────────┘
```

### Debug/Testing Configurations

```
┌──────────────────────────────────────────────────────────────┐
│ Config Name: "tokens-indentation"                           │
├──────────────────────────────────────────────────────────────┤
│ Description: Indentation transformations, stop at tokens    │
├──────────────────────────────────────────────────────────────┤
│ TransformationPipeline: Indentation                         │
│   - NormalizeWhitespace                                     │
│   - SemanticIndentation                                     │
│   - BlankLines                                              │
├──────────────────────────────────────────────────────────────┤
│ Target: Tokens { stage: Final }                             │
├──────────────────────────────────────────────────────────────┤
│ Output: TokenStream (Flat with Indent/Dedent)               │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│ Config Name: "tokens-linebased-flat"                        │
├──────────────────────────────────────────────────────────────┤
│ Description: Linebased up to LineTokens (no tree)           │
├──────────────────────────────────────────────────────────────┤
│ TransformationPipeline: LinebasedFlat                       │
│   - NormalizeWhitespace                                     │
│   - SemanticIndentation                                     │
│   - BlankLines                                              │
│   - ToLineTokens                                            │
├──────────────────────────────────────────────────────────────┤
│ Target: Tokens { stage: Final }                             │
├──────────────────────────────────────────────────────────────┤
│ Output: TokenStream (Tree with LineToken nodes)             │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│ Config Name: "tokens-linebased-tree"                        │
├──────────────────────────────────────────────────────────────┤
│ Description: Full linebased transformations (with tree)     │
├──────────────────────────────────────────────────────────────┤
│ TransformationPipeline: Linebased                           │
│   - NormalizeWhitespace                                     │
│   - SemanticIndentation                                     │
│   - BlankLines                                              │
│   - ToLineTokens                                            │
│   - IndentationToTree                                       │
├──────────────────────────────────────────────────────────────┤
│ Target: Tokens { stage: Final }                             │
├──────────────────────────────────────────────────────────────┤
│ Output: TokenStream (Nested tree with LineContainer)        │
└──────────────────────────────────────────────────────────────┘
```

## Comparison Matrix

| Aspect | Current (Multiple Systems) | Proposed (Unified) |
|--------|---------------------------|-------------------|
| **Entry Points** | 3 (LexPipeline, Registries, processor) | 1 (PipelineExecutor) |
| **Concept Count** | Lexer name + Parser name + Format string + Pipeline stages | Config name + Output format |
| **Lines of Code** | ~1500 (registries + orchestration + processor) | ~800 (config + executor + formatter) |
| **To Add New Pipeline** | Create impl, register in 2 places, update processor | Add one ProcessingConfig |
| **To Test Both Parsers** | Create 2 LexPipelines or use processor format strings | Execute 2 configs by name |
| **To Debug Tokens** | Call lexer directly or use special format string | Execute token-only config |
| **Type Safety** | Strings everywhere | Enums for pipeline/parser specs |
| **Documentation** | Scattered across 3 modules | Self-contained in config descriptions |

## Migration Strategy Visual

```
Phase 1: Add New System (Parallel)
┌────────────────────────────────────────┐
│         Existing Code                  │
│  ┌──────────────────────────────────┐  │
│  │ LexerRegistry                    │  │
│  │ ParserRegistry                   │  │
│  │ LexPipeline                      │  │
│  │ processor                        │  │
│  └──────────────────────────────────┘  │
│                                        │
│         New Code (Added)               │
│  ┌──────────────────────────────────┐  │
│  │ ProcessingConfig                 │  │
│  │ PipelineExecutor                 │  │
│  │ OutputFormatter                  │  │
│  └──────────────────────────────────┘  │
│                                        │
│  Both systems work in parallel         │
│  Tests migrated incrementally          │
└────────────────────────────────────────┘

Phase 2: Deprecate Old System
┌────────────────────────────────────────┐
│         Existing Code (Deprecated)     │
│  ┌──────────────────────────────────┐  │
│  │ #[deprecated]                    │  │
│  │ LexerRegistry                    │  │
│  │ ParserRegistry                   │  │
│  │ LexPipeline                      │  │
│  └──────────────────────────────────┘  │
│                                        │
│         New Code (Primary)             │
│  ┌──────────────────────────────────┐  │
│  │ ProcessingConfig                 │  │
│  │ PipelineExecutor   ← Main API    │  │
│  │ OutputFormatter                  │  │
│  └──────────────────────────────────┘  │
│                                        │
│  processor uses new system internally  │
│  Binary uses new system                │
│  Old system emits warnings             │
└────────────────────────────────────────┘

Phase 3: Remove Old System
┌────────────────────────────────────────┐
│         New Code (Only)                │
│  ┌──────────────────────────────────┐  │
│  │ ProcessingConfig                 │  │
│  │ PipelineExecutor                 │  │
│  │ OutputFormatter                  │  │
│  └──────────────────────────────────┘  │
│                                        │
│  Clean, simplified codebase            │
│  Single clear path for all processing │
│  ~700 lines of duplicate code removed │
└────────────────────────────────────────┘
```

## Key Insights

### What Makes This Better?

1. **Conceptual Clarity**
   - Old: "Pick a lexer, pick a parser, hope they're compatible"
   - New: "Pick a named configuration that's guaranteed to work"

2. **Separation of Concerns**
   - Pipeline construction (config) ≠ Output formatting (formatter)
   - Processing logic ≠ Serialization logic
   - Transformation choice ≠ Target choice (tokens vs AST)

3. **Extensibility**
   - Add new config → Add one struct to registry
   - Add new transformation → Add variant to enum
   - Add new output format → Add variant to OutputFormat enum

4. **Discoverability**
   - `lex list-configs` shows all options with descriptions
   - Config names self-document ("tokens-linebased-flat" is obvious)
   - Type system guides usage (can't pass wrong args)

### What Stays The Same?

- ✓ Pipeline infrastructure (TokenStream, StreamMapper)
- ✓ All transformation implementations
- ✓ Parser implementations
- ✓ AST building utilities
- ✓ Base tokenization

The proposal is purely about **how we select and orchestrate** these components, not about changing the components themselves.
