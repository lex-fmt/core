# Test Audit Report

**Date:** 2025-11-02
**Total Tests:** 418
**Tests with Violations:** 40 (9.6%)
**Clean Tests:** 378 (90.4%)

## Summary by Violation Type

| Violation Type | Count | Percentage | Description |
|----------------|-------|------------|-------------|
| `hardcoded_source` | 22 | 5.3% | Tests using ad hoc hardcoded txxt source strings |
| `no_source` | 12 | 2.9% | AST nodes/tokens created without proper source/location data |
| `manual_construction` | 6 | 1.4% | Tokens/tags created manually instead of using factories |

## Tagged Files

The following files now have `@audit` tags marking violations:

1. `src/txxt/ast/location.rs` - 12 tests with `no_source` violations
2. `src/txxt/lexers/linebased/pipeline.rs` - 17 tests with `hardcoded_source` violations
3. `tests/lexer_proptest.rs` - 6 tests with `manual_construction` violations
4. `tests/parameter_proptest.rs` - 5 tests with `hardcoded_source` violations

## Finding Tagged Tests

```bash
# Find all violations
rg '@audit' --type rust

# Find specific violation types
rg '@audit:.*no_source' --type rust
rg '@audit:.*manual_construction' --type rust
rg '@audit:.*hardcoded_source' --type rust

# Count violations by type
rg '@audit:.*no_source' -c
rg '@audit:.*manual_construction' -c
rg '@audit:.*hardcoded_source' -c

# List files with violations
rg '@audit' --type rust --files-with-matches
```

## Remediation Strategy

### Phase 1: Low-Hanging Fruit (6 tests, ~1-2 hours)
- **Focus:** `manual_construction` violations in `tests/lexer_proptest.rs`
- **Action:** Replace direct `Token::*` construction with `mk_token()` and `mk_tokens()` factories
- **Files:** 1 file, 6 tests

### Phase 2: Location Data (12 tests, ~2-3 hours)
- **Focus:** `no_source` violations in `src/txxt/ast/location.rs`
- **Action:** Add proper location data to Position/Location test construction
- **Files:** 1 file, 12 tests
- **Note:** These are foundational location tests, may need careful consideration

### Phase 3: Hardcoded Sources (22 tests, ~4-6 hours)
- **Focus:** `hardcoded_source` violations across multiple files
- **Action:** Extract hardcoded strings to test fixtures or structured builders
- **Files:** 2 files (pipeline.rs, parameter_proptest.rs), 22 tests
- **Strategy:** Create reusable test fixtures in `src/txxt/testing/` module

### Batch Organization

**Batch 1:** Property tests (manual construction)
- `tests/lexer_proptest.rs` - 6 tests

**Batch 2:** Location infrastructure tests
- `src/txxt/ast/location.rs` - 12 tests

**Batch 3a:** Lexer pipeline tests
- `src/txxt/lexers/linebased/pipeline.rs` - 17 tests

**Batch 3b:** Parameter parsing tests
- `tests/parameter_proptest.rs` - 5 tests

## Benefits of Remediation

1. **Easier Maintenance:** Changing factories updates all tests automatically
2. **Better Error Messages:** Proper location data improves debugging
3. **Refactoring Safety:** Can change internal representations without updating 22+ hardcoded strings
4. **Documentation:** Test fixtures serve as canonical examples
5. **Consistency:** All tests follow the same patterns

## Next Steps

1. ✅ Audit complete - violations tagged
2. ⬜ Review and validate tags (spot-check a few tests)
3. ⬜ Choose a batch to start with
4. ⬜ Create test fixtures infrastructure (if needed for Phase 3)
5. ⬜ Fix tests batch by batch
6. ⬜ Remove `@audit` tags as tests are fixed
7. ⬜ Track progress: `rg '@audit' -c` should decrease over time
