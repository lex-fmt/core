# Issue #202: Extended list markers flattened by formatter

## Summary
The lex formatter now infers list style from `List.marker`, but `visit_list_item` ignores the `Form::Extended` flag. Extended markers such as `1.2.3` or `IV.2.1)` are serialized as simple incremental markers (`1.`, `2.`, …) whenever `normalize_seq_markers` is true (the default). Previously those markers were left untouched, so documents relying on hierarchical numbering are now mangled.

## Impact
- Multi-part list markers lose their structure, defeating outline numbering.
- Formatting `1.2.3 Item` results in `1. Item`, so re-formatting a file is destructive.

## Reproduction
1. Create a `.lex` document with an extended marker list:
   ```
   1.2.3 Item one
   1.2.4 Item two
   ```
2. Run the lex formatter (or `lex-babel` serializer) with default rules.
3. Output shows `1.`/`2.` markers instead of the original `1.2.3`/`1.2.4`.

## Recommended Fix
When `Form::Extended`, bypass normalization and emit `SequenceMarker.raw_text` exactly (respecting the author’s nested index). Alternatively, make normalization generate the proper extended numbering, but the simpler/safer fix is to preserve the original marker for extended forms.

Add regression tests in `lex-babel` to assert that formatting extended marker lists keeps the source markers intact when normalization is on.
