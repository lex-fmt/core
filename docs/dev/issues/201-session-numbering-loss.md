# Issue #201: Session numbering removed from exporters

## Summary
Session titles parsed from `.lex` documents now carry `SequenceMarker` metadata, but the exporter pipeline strips the marker text entirely. When converting to Markdown or HTML, headings such as `1. Primary Session` become `Primary Session`, losing the numbering that conveys structure. This is a regression from main where numbering round-tripped correctly.

## Impact
- All Markdown and HTML exports drop session numbering.
- Tests already show the drift (e.g., `lib__html__export__kitchensink.snap` now lacks `1.`).
- Downstream tools cannot rely on numbering; TOCs and references break.

## Reproduction
1. Parse `tests/fixtures/kitchensink.lex` (or any numbered session doc).
2. Run `cargo test -p lex-babel html::export::kitchensink` (or use `lex convert input.lex html`).
3. Observe headings rendered without numeric prefixes.

## Recommended Fix
When building the IR (`lex-babel/src/ir/from_lex.rs`), keep the marker text. Either:
- Concatenate `session.marker.raw_text` with the stripped `title_text()` when constructing `Heading`, or
- Add explicit IR fields for marker metadata and re-emitting them in format serializers.

After fixing, add regression tests verifying that Markdown/HTML exports retain numbering for at least:
- Simple numeric markers (`1.`)
- Nested numbering (`1.2.`)
- Parenthetical markers (`1)` or `(1)`)
