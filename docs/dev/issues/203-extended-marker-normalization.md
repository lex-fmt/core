# Issue #203: Normalize extended list markers from structured segments

## Current Behavior
Extended list markers (multi-part sequences such as `1.2.3` or `IV.b.2)`) are now preserved verbatim whenever the formatter's `normalize_seq_markers` flag is enabled. This avoids corrupting hierarchy, but it also means extended lists never normalize—whatever the author typed is re-emitted, even if numbering is inconsistent.

## Desired Behavior
When extended lists are normalized, the formatter should regenerate each marker deterministically using structured metadata, so the hierarchy becomes `1.1.1`, `1.1.2`, etc., according to the list's decoration style(s) and separators. That requires:

1. Parsing each extended marker into segments (style + value + separator) rather than treating it as an opaque string.
2. Tracking per-level counters so siblings bump the correct segment and lower levels reset when a higher segment advances.
3. Propagating ancestor metadata into child lists so nested numbering reflects the full path.
4. Defining how mixed styles/separators normalize (e.g., should `1.a.iii` normalize to numeric-only or preserve the mixed styles?).

## Definition Of Done
- Extend the formatter's list context to capture parsed segments for the entire ancestral path, not just the first item's overall style.
- When `normalize_seq_markers` is true and a list is `Form::Extended`, synthesize normalized markers from that structured state instead of copying the raw text.
- Document the normalization rules for mixed-style segments and add tests covering numeric, alphabetic, roman, and mixed extended markers.

## Verification
Add formatter tests that start from intentionally irregular extended lists and assert that the output normalizes to predictable sequences (e.g., `1.1.1`, `1.1.2`, etc.) under different style combinations. Also ensure round-trip tests still pass when normalization is disabled so authors can preserve custom numbering.
