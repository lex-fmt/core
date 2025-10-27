# Parser Recursion Implementation Notes

## Background

The txxt format is inherently recursive - elements can contain other elements. The challenge is implementing this recursion in the parser while working within Rust's type system and Chumsky's parser combinator constraints.

## Current Implementation Status

### What Works
- Sessions can recursively contain other sessions ✅
- Nested definitions work within a single parent (but siblings fail due to unrelated bug) ✅
- All existing tests pass ✅

### What Doesn't Work
- Definitions cannot contain nested definitions at arbitrary depth
- List items cannot contain definitions
- Annotations cannot contain foreign blocks
- Container type restrictions are not enforced

## Technical Challenges

### 1. Type Recursion Problem
When trying to use parameterized functions within a single `recursive()` block:
```rust
recursive(|content| {
    let def = definition_with_content(content.clone());
    // This creates infinite type: Parser<Item = Definition<Parser<Item = Definition<...>>>>
})
```

### 2. Multiple Recursive Blocks
- Cannot reference each other (causes stack overflow at runtime)
- Each element currently has its own isolated recursive block
- This prevents proper cross-element nesting

## Attempted Solutions

### Boxing Strategy (Failed)
**Attempt**: Use `.boxed()` to break type recursion cycles
```rust
let definition_parser = recursive(|def_content| {
    // ... parser implementation
}).boxed();
```

**Result**:
- ✅ Compiles successfully
- ✅ Breaks type recursion cycle
- ❌ Fails at runtime - cannot parse even simple definitions
- ❌ Appears to be an issue with how Chumsky handles boxed recursive parsers

**Hypothesis**: The boxing might interfere with Chumsky's internal recursive reference handling, causing the parser to fail when trying to recurse.

## Potential Future Approaches

### 1. Manual Boxing with Box<dyn Parser>
Instead of using `.boxed()`, manually box with trait objects. More complex but might avoid Chumsky's internal issues.

### 2. Inline Everything
Manually inline all parser logic within a single recursive block. Very verbose but avoids function calls entirely.

### 3. Alternative Parser Library
Consider nom or pest which handle recursion differently.

### 4. Macro-based Solution
Generate the recursive parser structure using macros to avoid manual repetition.

## Current Pragmatic Solution

The codebase uses isolated recursive blocks for each element type. While not architecturally ideal:
- It works correctly
- All tests pass
- It's maintainable
- It can be refactored later when a better solution is found

## Infrastructure Ready for Future

The parameterized `_with_content()` parser functions are implemented and ready:
- `list_item_with_content()`
- `definition_with_content()`
- `annotation_with_content()`

These can be activated once the recursion challenge is solved.

## References
- Issue #31: Complete transition to unified recursive parser
- Chumsky recursive combinator: https://docs.rs/chumsky/latest/chumsky/recursive/fn.recursive.html