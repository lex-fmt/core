# Required Rust APIs for Editor Features

    This document lists the Rust APIs required to support the features outlined in @editors/README.md.
    It maps the editor roadmap to the existing crates so we reuse the code already in place and keep
    the plugins as thin UI layers.

1. Workspace Architecture (see src/lib.rs)

    | Crate | Responsibilities | Shared By |
    |-------|-----------------|-----------|
    | `lex-parser` | Parse Lex text into ASTs, track ranges, resolve intra-document links | `lex-lsp`, `lex-cli`, tests |
    | `lex-analysis` | Document symbols, folding, hover previews, references, semantic tokens | `lex-lsp`, future commands |
    | `lex-babel` | Canonical Lex serialization + Markdown/HTML/PDF conversions | `lex-lsp` formatting, publishing commands, CLI |
    | `lex-lsp` | LSP server, TextEdit diffs, document links, executeCommand bridge | VSCode + Neovim plugins |

2. Core & Navigation (Standard LSP Methods)

    These features use standard LSP requests. All logic lives in reusable crates so new editor
    integrations only need to hook up `lex-lsp`.

    | Feature | LSP Method | Implementation | Status |
    |---------|-----------|----------------|--------|
    | Syntax Highlighting | `textDocument/semanticTokens` | `lex-analysis/src/semantic_tokens.rs` | Live |
    | Document Symbols | `textDocument/documentSymbol` | `lex-analysis/src/document_symbols.rs` | Live |
    | Folding | `textDocument/foldingRange` | `lex-analysis/src/folding_ranges.rs` | Live |
    | Hover | `textDocument/hover` | `lex-analysis/src/hover.rs` | Live |
    | Go to Definition | `textDocument/definition` | `lex-analysis/src/go_to_definition.rs` | Live |
    | References | `textDocument/references` | `lex-analysis/src/references.rs` | Live |
    | Document Links | `textDocument/documentLink` | `lex-lsp/src/features/document_links.rs` (+ `lex_parser::lex::ast::links`) | Live |
    | Formatting | `textDocument/formatting` + `/rangeFormatting` | `lex-lsp/src/features/formatting.rs` (uses `lex-babel`) | Live |
    | Semantic Tokens | `textDocument/semanticTokens/full` | `lex-analysis/src/semantic_tokens.rs` | Live |
    | Completion | `textDocument/completion` | (needs new module reusing `lex_parser::lex::ast::links` + `lex-analysis::reference_targets`) | Planned |
    | Diagnostics | `textDocument/publishDiagnostics` | **Postponed** – Lex never hard-fails parsing, need better design | Postponed |

3. Command-Based Features (`workspace/executeCommand`)

    Only the `lex.echo` placeholder is wired today. The table below lists the planned commands along
    with the internal APIs they should wrap.

    | Feature | Command | Implementation Notes | Status |
    |---------|---------|----------------------|--------|
    | Insert Asset | `lex.insert_asset` | Use `lex_parser::lex::ast::links` helpers to resolve relative paths and emit a TextEdit via formatter | Planned |
    | Insert Verbatim | `lex.insert_verbatim` | Similar to assets but template verbatim fences backed by `lex-babel` serializer | Planned |
    | Convert Doc | `lex.convert` | Invoke Babel’s format registry (`lex-babel/src/registry.rs`) and stream artifact paths back to the client | Planned |
    | Export Doc | `lex.export` | Same as convert but fixed templates (HTML → PDF) managed by Babel | Planned |
    | Next Annotation | `lex.next_annotation` | Use `lex-analysis::utils::find_annotation_at_position` to iterate and return a Position | Planned |
    | Resolve Annotation | `lex.resolve_annotation` | Use formatter + `lex-analysis` metadata to mark resolved/unresolved annotations | Planned |
    | Toggle Annotations | `lex.toggle_annotations` | Pure editor UX toggle; no new Rust logic needed (documented for completeness) | Planned |
    | Help | `lex.help` | Surface snippets from `docs/specs` and `docs/dev/guides` | Planned |

4. Backend Building Blocks

    Underlying logic required by the above commands already exists. When planning work, prefer
    plugging these modules in instead of re-implementing them in editor-specific code.

    | Capability | Module | Notes |
    |------------|--------|-------|
    | Markdown / HTML conversion | `lex-babel/src/formats/{markdown,html}` | Bidirectional, exercised via CLI |
    | PDF export (experimental) | `lex-babel/src/formats/pdf` | Uses HTML bridge; still evolving |
    | Canonical Lex formatting | `lex-babel/src/transforms.rs` + `lex-lsp/src/features/formatting.rs` | Shared between CLI and LSP |
    | Asset resolution | `lex_parser::lex::ast::links` | Generates file/document links for annotations and verbatim blocks |
    | Annotation index | `lex-analysis/src/utils.rs` | Find annotations/definitions/sessions by position or label |

5. Next Steps

    * Wire completion and command handlers inside `lex-lsp` so editors can call into the shared logic.
    * Keep adding tests in the Rust crates; editor repos should stay as thin as possible.
    * Revisit diagnostics once we define a meaningful signal for "structural issues" despite
      parser fallbacks.
