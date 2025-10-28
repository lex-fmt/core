# Major Refactoring Roadmap for Parser

## Overview
Three critical refactoring opportunities exist that would reduce parser code by ~510 lines (12.7% reduction):

---

## CHALLENGE 1: Duplicate WithSpans Structures (64 lines, 100% duplication)

### Current State
- **parser.rs** (lines 35-103): Defines all 8 WithSpans structs + enum + document struct
- **intermediate_ast.rs** (lines 1-71): Re-defines same 8 structs identically
- **Root cause**: Incomplete refactoring - `intermediate_ast.rs` was created but `parser.rs` wasn't updated

### Problem
1. Single source of truth unclear - which file is authoritative?
2. Code review confusion - changes must be made in both places
3. Maintenance burden - bug fixes duplicated

### Proposed Solution
**Remove all struct definitions from parser.rs and import from intermediate_ast.rs**

#### Step 1: Update conversion/basic.rs imports
Change from:
```rust
use super::super::parser::{
    AnnotationWithSpans, ContentItemWithSpans, DefinitionWithSpans, DocumentWithSpans,
    ForeignBlockWithSpans, ListItemWithSpans, ListWithSpans, ParagraphWithSpans, SessionWithSpans,
};
```

To:
```rust
use super::super::intermediate_ast::{
    AnnotationWithSpans, ContentItemWithSpans, DefinitionWithSpans, DocumentWithSpans,
    ForeignBlockWithSpans, ListItemWithSpans, ListWithSpans, ParagraphWithSpans, SessionWithSpans,
};
```

#### Step 2: Add import to parser.rs (if not already present)
```rust
use super::intermediate_ast::{
    AnnotationWithSpans, ContentItemWithSpans, DefinitionWithSpans, DocumentWithSpans,
    ForeignBlockWithSpans, ListItemWithSpans, ListWithSpans, ParagraphWithSpans, SessionWithSpans,
};
```

#### Step 3: Remove lines 35-103 from parser.rs entirely
Delete the entire section with comments "Intermediate AST structures..."

### Impact
- **Lines removed**: 69 lines
- **Files modified**: 3 (parser.rs, conversion/basic.rs, intermediate_ast.rs)
- **Complexity**: LOW
- **Risk**: VERY LOW - only imports, no logic changes

---

## CHALLENGE 2: Unused/Abandoned Combinators Module (216 lines, 100% dead code)

### Current State
- **combinators.rs** exists with 8 parser combinator functions
- **parser.rs** defines its own versions of the same 8 functions (lines ~120-250)
- **Status**: combinators.rs is never imported or used anywhere
- **Root cause**: Incomplete refactoring - extracted combinators but never switched over

### Functions Duplicated
1. `token()` - match specific token type
2. `text_line()` - parse text sequence
3. `list_item_line()` - parse list item marker
4. `paragraph()` - parse paragraph block
5. `definition_subject()` - parse definition subject
6. `session_title()` - parse session title
7. `annotation_header()` - parse annotation metadata
8. `foreign_block()` - parse foreign code block

### Problem
1. Dead code maintenance burden
2. Divergence risk - fixes in one place don't apply to the other
3. Unclear intent - why are there two versions?
4. Developer confusion - which version to update?

### Proposed Solution
**DELETE combinators.rs entirely and use inline definitions from parser.rs**

OR (Better Alternative):

**ACTIVATE combinators.rs and switch parser.rs to import from it**

#### Implementation: Switch to using combinators module

Step 1: **Update parser.rs imports**
```rust
use super::combinators::{
    annotation_header, definition_subject, foreign_block, list_item_line,
    paragraph, session_title, text_line, token,
};
```

Step 2: **Remove duplicate function definitions from parser.rs**
Delete lines where these functions are defined (approximately 130+ lines)

Step 3: **Verify combinators.rs is correctly using helper functions**
Ensure all dependencies on `is_text_token()` and other helpers are properly imported

### Impact
- **Lines removed**: 216 lines (all of combinators.rs not used, OR inline definitions from parser.rs)
- **Files modified**: 2 (parser.rs, combinators.rs)
- **Complexity**: MEDIUM (need to ensure all imports work)
- **Risk**: MEDIUM - need comprehensive testing, but logic doesn't change

---

## CHALLENGE 3: Fragmented Conversion Function Exports (230+ lines, 80% duplication)

### Current State
```
conversion/basic.rs (154 lines)
  ├─ convert_document
  ├─ convert_paragraph
  ├─ convert_session
  ├─ ... (9 functions total)
  └─ All full implementations

ast_conversion.rs (466 lines)
  ├─ convert_document
  ├─ convert_paragraph
  ├─ convert_session
  ├─ ... (9 basic functions)
  └─ + 9 position-aware variants

conversion/positions.rs (stub - to be implemented)
```

### Problem
1. **Two versions of identical logic** exist side-by-side
2. **No consistency** - basic converters should not be duplicated
3. **Re-export modules** (basic.rs, positions.rs) are incomplete and confusing
4. **Maintenance nightmare** - changes to conversion logic must be made twice

### Current State of Solution
✅ **Already in progress**:
- Made 18 functions in ast_conversion.rs public (`pub(crate)`)
- Created conversion/basic.rs as re-export module
- Created conversion/positions.rs as re-export module

### What Remains
**Finish the consolidation**:

#### Step 1: Make conversion/basic.rs a pure re-export module
Already done! Current state:
```rust
//! Basic AST conversion functions (re-exported from ast_conversion.rs)
#[allow(unused_imports)]
pub(crate) use super::super::ast_conversion::{
    convert_annotation, convert_content_item, convert_definition, convert_document,
    convert_foreign_block, convert_list, convert_list_item, convert_paragraph, convert_session,
};
```

#### Step 2: Complete conversion/positions.rs as pure re-export module
Make sure it exports all position-preserving variants:
```rust
//! Position-preserving AST conversion functions (re-exported from ast_conversion.rs)
#[allow(unused_imports)]
pub(crate) use super::super::ast_conversion::{
    convert_annotation_with_positions, convert_content_item_with_positions,
    convert_definition_with_positions, convert_document_with_positions,
    convert_foreign_block_with_positions, convert_list_with_positions,
    convert_list_item_with_positions, convert_paragraph_with_positions,
    convert_session_with_positions,
};
```

#### Step 3: Remove duplicate conversions from parser.rs
Delete lines 109-296 (Position-Preserving Conversion Functions section)
Replace with import:
```rust
use super::conversion::positions::{
    convert_document_with_positions,
    // Only import the ones actually used
};
```

#### Step 4: Update imports in parser/parser.rs submodule
Ensure it imports from conversion modules, not internal functions

### Impact
- **Lines removed**: 192 lines (from parser.rs duplicates)
- **Files modified**: 4+ (ast_conversion.rs, conversion/basic.rs, conversion/positions.rs, parser.rs)
- **Complexity**: MEDIUM-HIGH (pre-commit hook complications observed)
- **Risk**: MEDIUM (need careful import management to avoid circular deps)

---

## BONUS CHALLENGE: Parser Combinator Complexity (238 inline patterns)

### Problem
Parser combinators use 238+ inline `.map()`, `.then()`, `.or()`, `.repeated()` chains
scattered throughout parser.rs with no abstraction

### Example
```rust
// Current: Inline chain (10+ lines in foreign_block)
token(Token::TxxtMarker)
    .ignore_then(annotation_header())
    .then_ignore(token(Token::TxxtMarker))
    .then(token(Token::Whitespace).ignore_then(text_line()).or_not())
    .map(|((label_span, parameters), content_span)| {
        // ... complex mapping logic ...
    })

// Could be: Named combinator (reusable)
fn closing_annotation_parser() -> impl Parser<TokenSpan, AnnotationWithSpans> {
    // ... same logic, but named and reusable
}
```

### Proposed Solution
Extract high-value chains into named combinators in combinators.rs

Focus on:
1. `closing_annotation_parser` - used in foreign block (20+ lines)
2. `with_content` - used in foreign block (15+ lines)
3. `rest_of_line` - duplicated in list_item_line and elsewhere (10+ lines)

### Impact
- **Lines reduced**: 50-75 lines (through abstraction, not deletion)
- **Readability**: Greatly improved
- **Testability**: Easier to unit test parser pieces
- **Complexity**: MEDIUM (requires understanding chumsky combinators)

---

## RECOMMENDED REFACTORING ORDER

### Phase 1 (Week 1) - Easy Wins
1. **Consolidate WithSpans structures** (Challenge 1)
   - Effort: 30 minutes
   - Impact: -64 lines, single source of truth
   - Risk: Very low

### Phase 2 (Week 1-2) - Medium Wins
2. **Activate combinators.rs** (Challenge 2)
   - Effort: 2-3 hours
   - Impact: -216 lines, eliminate dead code
   - Risk: Medium (needs full test suite)

3. **Complete conversion consolidation** (Challenge 3, finish)
   - Effort: 2-3 hours
   - Impact: -192 lines, single source of truth
   - Risk: Medium (pre-commit hooks, circular imports)

### Phase 3 (Week 2+) - Architectural Improvements
4. **Extract complex combinator chains** (Bonus Challenge)
   - Effort: 4-6 hours
   - Impact: -50+ lines, improved readability
   - Risk: Low (no logic changes, just refactoring)

---

## TOTAL REFACTORING IMPACT

### Code Reduction
- **Phase 1**: -64 lines (100% duplication removal)
- **Phase 2**: -216 lines (100% dead code removal)
- **Phase 3**: -192 lines (80% duplication removal)
- **Phase 4**: -50 lines (complexity reduction)
- **TOTAL**: -522 lines (13% of parser code)

### Files Modified
- Phase 1: 3 files
- Phase 2: 2 files
- Phase 3: 4 files
- Phase 4: 1 file

### Quality Improvements
✓ Single source of truth for all conversion logic
✓ No more dead code (combinators.rs becomes active or deleted)
✓ Unified WithSpans definitions
✓ Clearer module boundaries
✓ Better separation of concerns

---

## Testing Strategy

After each phase, run:
```bash
cargo test --lib              # Unit tests
cargo test --doc              # Doc tests
cargo clippy -- -D warnings   # Lint checks
cargo fmt --check             # Format checks
```

All 175 tests must pass after each change.
