# Test Harness Guide

This guide explains the new test harness infrastructure for testing individual element variations.

## Overview

The test harness provides utilities for:

- Loading per-element test files by type and number
- Parsing with different parser implementations (Reference or Linebased)
- Comparing results from multiple parsers
- Extracting and asserting on AST elements
- **Fluent, chainable API** - no `.unwrap()` needed!

## Basic Usage (Fluent API - Recommended)

```rust
use lex::lex::testing::test_harness::*;

// Clean, chainable API - no unwrap() needed!
let parsed = ElementSources::paragraph(1).parse();
let paragraph = parsed.expect_paragraph();

// Make assertions
assert!(paragraph_text_starts_with(paragraph, "This is a simple"));
```

## Alternative: Verbose API (Still Available)

```rust
use lex::lex::testing::test_harness::*;

// Older verbose style (still works)
let source = ElementSources::get_source_for(ElementType::Paragraph, 1).unwrap();
let doc = parse_with_parser(&source, Parser::Reference).unwrap();
let paragraph = get_first_paragraph(&doc).unwrap();

assert!(paragraph_text_starts_with(paragraph, "This is a simple"));
```

## API Reference

### Fluent API (Recommended)

#### Element-Specific Shortcuts

```rust
// Type-specific shotcuts (cleaner than ElementType enum)
ElementSources::paragraph(1)    // Returns ElementLoader
ElementSources::list(2)
ElementSources::session(3)
ElementSources::definition(4)
ElementSources::annotation(5)
ElementSources::verbatim(6)

// Generic form
ElementSources::load(ElementType::Paragraph, 1)
```

#### ElementLoader Methods

```rust
// Get raw source string
.source() -> String

// Parse with default (Reference) parser
.parse() -> ParsedElement

// Parse with specific parser
.parse_with(Parser::Reference | Parser::Linebased) -> ParsedElement
```

#### ParsedElement Methods

**Panicking extraction (use in tests):**

```rust
.expect_paragraph() -> &Paragraph    // Panics if not found
.expect_session() -> &Session
.expect_list() -> &List
.expect_definition() -> &Definition
.expect_annotation() -> &Annotation
.expect_verbatim() -> &Verbatim
```

**Safe extraction (returns Option):**

```rust
.first_paragraph() -> Option<&Paragraph>
.first_session() -> Option<&Session>
.first_list() -> Option<&List>
.first_definition() -> Option<&Definition>
.first_annotation() -> Option<&Annotation>
.first_verbatim() -> Option<&Verbatim>
```

**Access underlying document:**

```rust
.document() -> &Document
```

### Must_ Methods (No unwrap needed)

```rust
// Get source, panicking with helpful message if not found
ElementSources::must_get_source_for(ElementType::Paragraph, 1) -> String

// Get AST, panicking if not found or parse fails
ElementSources::must_get_ast_for(ElementType::Paragraph, 1, Parser::Reference) -> Document
```

### Result-Based API (Original)

```rust
// Get source string for element type and number
ElementSources::get_source_for(ElementType::Paragraph, 1) -> Result<String>

// Get AST directly (parses with specified parser)
ElementSources::get_ast_for(ElementType::Paragraph, 1, Parser::Reference) -> Result<Document>

// List available numbers for an element type
ElementSources::list_numbers_for(ElementType::Paragraph) -> Result<Vec<usize>>
```

### Element Types

```rust
enum ElementType {
    Paragraph,
    List,
    Session,
    Definition,
    Annotation,
    Verbatim,
}
```

### Parser Selection

```rust
enum Parser {
    Reference,  // Combinator-based, stable
    Linebased,  // Grammar-based, experimental
}

// Parse with specific parser
parse_with_parser(source: &str, parser: Parser) -> Result<Document>

// Parse with multiple parsers
parse_with_multiple_parsers(source: &str, parsers: &[Parser])
    -> Result<Vec<(Parser, Document)>>

// Compare results from multiple parsers
compare_parser_results(results: &[(Parser, Document)]) -> Result<(), String>
```

### Extracting Elements

```rust
// Get first element of each type from a document
get_first_paragraph(doc: &Document) -> Option<&Paragraph>
get_first_session(doc: &Document) -> Option<&Session>
get_first_list(doc: &Document) -> Option<&List>
get_first_definition(doc: &Document) -> Option<&Definition>
get_first_annotation(doc: &Document) -> Option<&Annotation>
get_first_verbatim(doc: &Document) -> Option<&Verbatim>
```

### Assertion Helpers

```rust
// Paragraph text assertions
paragraph_text_starts_with(paragraph: &Paragraph, expected: &str) -> bool
paragraph_text_contains(paragraph: &Paragraph, expected: &str) -> bool
```

For more detailed assertions, use the existing `assert_ast` fluent API from `lex::testing`.

## File Naming Convention

Element test files follow this pattern:

```
docs/specs/v1/elements/{element}/{element}-{NN}-{flat|nested}-{hint}.lex
```

Examples:

- `paragraph-01-flat-oneline.lex`
- `list-07-nested-simple.lex`
- `annotation-10-nested-complex.lex`

## Multi-Parser Testing

Test that both parsers produce equivalent ASTs:

```rust
let source = ElementSources::get_source_for(ElementType::Paragraph, 1).unwrap();
let parsers = vec![Parser::Reference, Parser::Linebased];
let results = parse_with_multiple_parsers(&source, &parsers).unwrap();

// Compare AST structures
match compare_parser_results(&results) {
    Ok(()) => println!("Parsers produced matching ASTs"),
    Err(msg) => println!("AST mismatch: {}", msg),
}
```

## Example Tests

### Simple Test (Fluent API)

```rust
#[test]
fn test_paragraph_variation_1() {
    // Load and parse with fluent API
    let parsed = ElementSources::paragraph(1).parse();
    let paragraph = parsed.expect_paragraph();

    // Make assertions
    assert!(paragraph_text_starts_with(paragraph, "This is a simple"));
}
```

### With Parser Selection

```rust
#[test]
fn test_with_linebased_parser() {
    let parsed = ElementSources::paragraph(1)
        .parse_with(Parser::Linebased);

    // Use safe extraction since parser may have issues
    if let Some(paragraph) = parsed.first_paragraph() {
        assert!(paragraph_text_contains(paragraph, "simple"));
    }
}
```

### With Fluent Assertions

```rust
#[test]
fn test_with_assert_ast() {
    let parsed = ElementSources::paragraph(1).parse();

    // Use existing assert_ast API
    use lex::lex::testing::assert_ast;
    assert_ast(parsed.document())
        .item_count(1)
        .item(0, |item| {
            item.assert_paragraph()
                .text_contains("simple");
        });
}
```

### Just Get Source

```rust
#[test]
fn test_raw_source() {
    // Get source without parsing
    let source = ElementSources::paragraph(1).source();
    assert!(source.contains("simple"));
}
```

### Verbose Style (Still Supported)

```rust
#[test]
fn test_verbose_style() {
    // Original verbose API still works
    let source = ElementSources::get_source_for(ElementType::Paragraph, 1).unwrap();
    let doc = parse_with_parser(&source, Parser::Reference).unwrap();
    let paragraph = get_first_paragraph(&doc).unwrap();

    assert!(paragraph_text_starts_with(paragraph, "This is a simple"));
}
```

## Quick Reference

| What you want | Fluent API | Verbose API |
|--------------|------------|-------------|
| Load & parse paragraph | `ElementSources::paragraph(1).parse()` | `ElementSources::get_source_for(ElementType::Paragraph, 1).unwrap()` |
| Get element | `.expect_paragraph()` | `get_first_paragraph(&doc).unwrap()` |
| Choose parser | `.parse_with(Parser::Linebased)` | `parse_with_parser(&source, Parser::Linebased).unwrap()` |
| Just source | `.source()` | Same |
| Safe extraction | `.first_paragraph()` | `get_first_paragraph(&doc)` |

## Notes

- **Fluent API recommended** - cleaner, no `.unwrap()` needed
- Parser implementations may have bugs - the test harness doesn't fix them
- Use `expect_*()` in tests (panics with message), `first_*()` for safe extraction
- Use `compare_parser_results()` to identify discrepancies between parsers
- The infrastructure is designed to be extended as new elements are added
- All element test files have been verified to parse correctly

## See Also

- Per-element library structure: `docs/specs/v1/elements/`
- Existing test assertions: `src/lex/testing/testing_assertions.rs`
- Fluent API demo: `tests/fluent_api_demo.rs`
- Example tests: `tests/test_harness_examples.rs`
- Implementation: `src/lex/testing/test_harness.rs`
