# Parser Bundle Refactor - Status

**Branch:** `feat/parser-bundle`
**Issue:** #26
**Status:** Phase 1 Complete ✅ | Phase 2 Needs Research 🔬

## Summary

Attempted to implement Parser Bundle pattern to enable ContentContainer support for Lists and Definitions. **Phase 1 succeeded**, creating clean `_impl` versions of all parsers. **Phase 2 blocked** on complex mutual recursion handling in chumsky.

## Phase 1: Success ✅

**Completed:**
- Created `list_item_impl(child_parser)`
- Created `definition_impl(child_parser)`
- Created `annotation_impl(child_parser)`
- Created `session_impl(child_parser)`
- All 55 parser tests passing
- Clean separation of parser logic from recursion handling

**Benefits:**
- More maintainable code structure
- Foundation for container improvements
- Easier to understand parser construction

**Commits:**
- `2b2f888` - Phase 1: Extract _impl versions
- `65f24e4` - Document Phase 2 blockers

## Phase 2: Needs Research 🔬

**Goal:** Use `Recursive::declare()` to enable mutual recursion between `annotation()` and `definition()` so Definitions can host Annotations and ForeignBlocks (full ContentContainer support).

**What We Tried:**

1. **Simple `Recursive::declare()` approach**
   - Declared parsers, built content dispatchers, defined parsers
   - Result: Parse failures at runtime

2. **Nested `recursive()` in content dispatchers**
   - Mixed `Recursive::declare()` with `recursive()` closures
   - Result: Type compatibility errors

**Blocker:** Don't fully understand `Recursive::declare()` API semantics in chumsky 0.9

## Current State

**Working:**
- ✅ Annotations can host: Paragraphs, Lists, Definitions
- ✅ Sessions can host: Everything
- ✅ All existing tests pass

**Deferred:**
- ⚠️ Definitions can only host: Paragraphs, Lists (missing Annotations, ForeignBlocks)
- ⚠️ Lists can only host: Paragraphs, nested Lists (missing Definitions, Annotations)

## Next Steps

1. **Research** `Recursive::declare()` in chumsky 0.9
   - Study examples and documentation
   - Create isolated 2-3 parser prototype
   - Validate pattern works

2. **Apply learnings** to full parser once pattern is understood

3. **Consider alternatives** if `Recursive::declare()` proves too complex
   - Chumsky 1.0 migration?
   - Grammar redesign to reduce mutual recursion?

## References

- See `local/parser-bundle-*.md` for detailed analysis
- Issue #26 for tracking
- `local/recursion-containers-challenge.txt` for original problem statement
