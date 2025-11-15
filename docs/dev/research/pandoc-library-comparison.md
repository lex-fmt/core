# Pandoc Library Research for lex-babel

## Design Principles from lex-babel

Based on `lex-babel/src/lib.rs`:

1. **Pure lib, shell agnostic** - No shell dependencies, stdout/stderr, or env vars
2. **Offload to specialized crates** - Use existing libraries for format handling
3. **Only adapt ASTs** - We write adapters between Lex AST ↔ format AST, not parsers/serializers
4. **Prefer Rust crates over shelling out** - Avoid external binaries when possible

## Candidate Libraries

### Option 1: `pandoc_types` (elliottslaughter/rust-pandoc-types)

**Repository:** https://github.com/elliottslaughter/rust-pandoc-types
**Crates.io:** https://crates.io/crates/pandoc_types
**Version:** 0.6.0 (Feb 2023)

#### Pros
- ✅ **Official port** of Haskell `pandoc-types` library
- ✅ **Explicit Pandoc 3.0+ compatibility** - supports pandoc-types 1.23
- ✅ Full serde support (`serde ^1.0`, `serde_tuple ^0.5.0`)
- ✅ Complete AST type coverage from upstream Haskell library
- ✅ Zero open issues (stable)
- ✅ Apache 2.0 license
- ✅ Matches our principle: "offload to specialized crates"

#### Cons
- ⚠️ Last updated Feb 2023 (less active)
- ⚠️ Smaller community (14 stars)
- ⚠️ Lower documentation coverage (26.52%)
- ⚠️ Fewer contributors (4)

#### Key Dependencies
```toml
serde = "^1.0"
serde_tuple = "^0.5.0"
serde_json = "^1.0" # dev-dependency
```

---

### Option 2: `pandoc_ast` (oli-obk/pandoc-ast)

**Repository:** https://github.com/oli-obk/pandoc-ast
**Crates.io:** https://crates.io/crates/pandoc_ast
**Version:** 0.8.6 (more recent)

#### Pros
- ✅ **More active maintenance** - version 0.8.6 vs 0.6.0
- ✅ **Larger community** - 38 stars, 11 forks, 9 contributors
- ✅ **Built-in visitor pattern** - `MutVisitor` trait for AST traversal
- ✅ **Purpose-built for filters** - exactly our use case (AST transformation)
- ✅ Full serde support (`serde ^1.0`, `serde_derive ^1.0`, `serde_json ^1.0`)
- ✅ MIT license
- ✅ Includes `filter()` helper function for JSON deserialization/serialization

#### Cons
- ⚠️ Pandoc version compatibility not explicitly documented
- ⚠️ Less clear lineage from official Pandoc types

#### Key Dependencies
```toml
serde = "^1.0.2"
serde_derive = "^1.0.2"
serde_json = "^1.0.1"
```

#### Core Types
- `Pandoc` - root document object
- `Block` - structured text (tables, lists, etc.)
- `Inline` - formatting items (bold, italic, links)
- `MutVisitor` - trait for AST transformation

---

### Option 3: `pandoc` (CLI wrapper) - ❌ NOT SUITABLE

**Why excluded:** Violates lex-babel principle #1 (shell agnostic). This crate wraps the pandoc executable, requiring external binary dependencies.

---

## Analysis Matrix

| Criterion | `pandoc_types` | `pandoc_ast` | Weight |
|-----------|----------------|--------------|--------|
| **Matches lex-babel principles** | ✅ | ✅ | Critical |
| **Serde JSON support** | ✅ | ✅ | Critical |
| **Active maintenance** | ⚠️ (2023) | ✅ (2024) | High |
| **Pandoc 3.0+ support** | ✅ Explicit | ❓ Unclear | High |
| **Community size** | ⚠️ (14⭐) | ✅ (38⭐) | Medium |
| **AST visitor pattern** | ❌ | ✅ `MutVisitor` | Medium |
| **Documentation** | ⚠️ (26%) | ❓ | Medium |
| **Purpose-fit** | General types | Filter-oriented | Medium |

---

## Recommendation

### Primary: `pandoc_ast`

**Rationale:**

1. **Better aligned with our use case** - We're essentially building filters/adapters, which is exactly what `pandoc_ast` was designed for
2. **Active maintenance** - More recent updates suggest ongoing compatibility
3. **Built-in transformation tools** - `MutVisitor` trait will be useful for Lex ↔ Pandoc conversions
4. **Proven in production** - Higher star count and more contributors suggest real-world usage
5. **Simpler dependency chain** - Uses standard serde without `serde_tuple`

### Fallback: `pandoc_types`

Use if we encounter issues with:
- Pandoc 3.0+ compatibility (explicitly guaranteed in `pandoc_types`)
- Need for exact Haskell type parity
- Specific pandoc-types 1.23 features

---

## Implementation Plan

### Phase 1: Proof of Concept
1. Add `pandoc_ast` as dependency to `lex-babel/Cargo.toml`
2. Create basic serialization test: `Lex Document → Pandoc JSON`
3. Create basic parsing test: `Pandoc JSON → Lex Document`
4. Verify JSON roundtrip with actual `pandoc` CLI

### Phase 2: Core Adapters
1. Implement `lex_ast → pandoc_ast` adapter in `lex-babel/src/formats/pandoc/serializer.rs`
2. Implement `pandoc_ast → lex_ast` adapter in `lex-babel/src/formats/pandoc/parser.rs`
3. Handle structural mismatches (flat headers ↔ nested sessions)
4. Add comprehensive tests

### Phase 3: Integration
1. Register `PandocJsonFormat` in `FormatRegistry::with_defaults()`
2. Add CLI tests for `lex convert --from lex --to pandoc-json`
3. Document workflow: `lex → pandoc-json → (pandoc CLI) → docx/pdf/etc.`

---

## Dependency Addition

```toml
# lex-babel/Cargo.toml
[dependencies]
lex-parser = { path = "../lex-parser" }
pandoc_ast = "0.8"  # Add this
serde_json = "1.0"  # Already in workspace, but needed for JSON serialization

[dev-dependencies]
insta = { workspace = true }
```

---

## Risk Mitigation

**If `pandoc_ast` proves inadequate:**
- Switch to `pandoc_types` (minimal code changes, both use serde)
- Both libraries can coexist if needed (use different module paths)
- JSON format is standardized, so AST structure should be compatible

**Pandoc version compatibility:**
- Test with multiple Pandoc versions (2.x and 3.x)
- Document minimum required Pandoc version
- Add integration tests that shell out to `pandoc` for validation

---

## Questions to Validate

Before final commitment, verify:

1. ✅ Does `pandoc_ast` work with Pandoc 3.0+? (Test with actual JSON)
2. ✅ Can we use `MutVisitor` for our adapter pattern?
3. ✅ Are the JSON serialization formats compatible with `pandoc --to json`?
4. ⚠️ What's the latest Pandoc version the community has tested with?

---

## Conclusion

**Recommended:** Start with `pandoc_ast` (0.8.6)

- Aligns with lex-babel principles (pure lib, AST adapters only)
- More active development
- Purpose-built for our use case (filters/transformations)
- Easy fallback to `pandoc_types` if needed
- Both are thin wrappers over the same underlying JSON format

The visitor pattern in `pandoc_ast` will be particularly useful for traversing and transforming the AST, which is exactly what we need for Lex ↔ Pandoc conversions.
