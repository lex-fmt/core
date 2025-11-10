# Repository Guidelines

## Project Structure & Module Organization
The Rust library entrypoint is `src/lib.rs`, with parsing, tokenizing, and UI modules under `src/lex/**`. The CLI lives in `src/bin/lex.rs` and should remain a thin wrapper over the library so features stay testable. Specifications live beside code in `docs/specs/<version>/{general,grammar}.lex`; update both when the language evolves. Fixtures and regression tests sit in `tests/`, exploratory parsers in `debug_parse/`, and reusable assets/themes in `assets/` and `themes/`.

## Build, Test, and Development Commands
- `cargo check` — fast gate used before every edit loop to validate the workspace.
- `cargo build --release` — produces the optimized `target/release/lex` binary for manual verification.
- `cargo run --bin lex -- examples/minimal.lex` — exercise the CLI parser against a sample document.
- `cargo test --all-targets` — runs unit, integration, proptest, and `insta` snapshot suites.
- `cargo fmt --all && cargo clippy --all-targets -- -D warnings` — enforces formatting and lints identical to CI.
- `./scripts/pre-commit --all` chains the same fmt/clippy/test steps the CI expects; use before every push. Coverage fans can run `./scripts/test-coverage` (requires `cargo-tarpaulin`).

## Coding Style & Naming Conventions
Follow `rustfmt` defaults (4-space indentation, trailing commas for multi-line literals). Prefer snake_case files and modules, CamelCase types, and SCREAMING_SNAKE_CASE constants to match existing modules. Keep parser stages pure and retry-friendly; pass shared state via structs instead of globals. Document grammar nuances in `///` doc comments and mirror the tone from `docs/specs` (terse, factual). New command-line flags should be added through Clap derives in `src/bin/lex.rs` and reflected in the specs.

## Testing Guidelines
Unit coverage lives close to the code, while integration suites such as `tests/tokenizer_elements.rs` and `tests/tokenizer_documents.rs` verify end-to-end token streams. Snapshot assertions use `insta`; after editing them, run `cargo insta review` to accept or reject updates before committing. Property-based checks rely on `proptest`, so seed-stabilize flaky cases with `ProptestConfig`. When touching CLI UX, add before/after output samples under `examples/` and point tests to them.

## Commit & Pull Request Guidelines
The history favors Conventional Commit prefixes (e.g., `fix(tests): adjust verbatim expectations`), so stick to `<type>(scope): imperative summary`. Reference impacted spec files (`docs/specs/vX.Y`) or grammar rules in the body and describe any snapshot updates. Every PR should explain motivation, list verification commands run, and include relevant screenshots or CLI snippets when UX changes. Link GitHub issues or spec tasks, request review from a domain owner, and keep diffs focused—open follow-ups for unrelated cleanups instead of mixing them here.
