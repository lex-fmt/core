# Required Rust APIs for Editor Features

This document lists the Rust APIs required to support the features outlined in @editors/README.md.
It maps the editor roadmap to the existing crates so we reuse the code already in place and keep
the plugins as thin UI layers.

## Feature Matrix

| Feature Group | Feature | LSP Method / Command | Implementation | LSP Ready |
|--------------|---------|----------------------|----------------|-----------|
| **Core & Navigation** | Syntax Highlighting | `textDocument/semanticTokens` | `lex-analysis/src/semantic_tokens.rs` | Yes |
| | Document Symbols | `textDocument/documentSymbol` | `lex-analysis/src/document_symbols.rs` | Yes |
| | Folding | `textDocument/foldingRange` | `lex-analysis/src/folding_ranges.rs` | Yes |
| | Hover | `textDocument/hover` | `lex-analysis/src/hover.rs` | Yes |
| | Go to Definition | `textDocument/definition` | `lex-analysis/src/go_to_definition.rs` | Yes |
| | References | `textDocument/references` | `lex-analysis/src/references.rs` | Yes |
| | Document Links | `textDocument/documentLink` | `lex-lsp/src/features/document_links.rs` | Yes |
| | Completion | `textDocument/completion` | `lex-analysis/src/completion.rs` | Yes |
| | Diagnostics | `textDocument/publishDiagnostics` | (Postponed) | No |
| **Formatting & Editing** | Formatting | `textDocument/formatting` | `lex-lsp/src/features/formatting.rs` | Yes |
| | Range Formatting | `textDocument/rangeFormatting` | `lex-lsp/src/features/formatting.rs` | Yes |
| | Semantic Tokens (Full) | `textDocument/semanticTokens/full` | `lex-analysis/src/semantic_tokens.rs` | Yes |
| **Interop & Commands** | Echo | `lex.echo` | `lex-lsp/src/features/commands.rs` | Yes |
| | Import Doc | `lex.import` | `lex-lsp/src/features/commands.rs` | Yes |
| | Export Doc | `lex.export` | `lex-lsp/src/features/commands.rs` | Yes |
| | Insert Asset | `lex.insert_asset` | `lex-lsp/src/features/commands.rs` | Yes |
| | Insert Verbatim | `lex.insert_verbatim` | `lex-lsp/src/features/commands.rs` | Yes |
| | Next Annotation | `lex.next_annotation` | `lex-lsp/src/features/commands.rs` | Yes |
| | Resolve Annotation | `lex.resolve_annotation` | `lex-lsp/src/features/commands.rs` | Yes |
| | Toggle Annotations | `lex.toggle_annotations` | `lex-lsp/src/features/commands.rs` | Yes |


## Backend Building Blocks

Underlying logic required by the above commands already exists. When planning work, prefer
plugging these modules in instead of re-implementing them in editor-specific code.

| Capability | Module | Notes |
|------------|--------|-------|
| Markdown / HTML conversion | `lex-babel/src/formats/{markdown,html}` | Bidirectional, exercised via CLI |
| PDF export (experimental) | `lex-babel/src/formats/pdf` | Uses HTML bridge; still evolving |
| Canonical Lex formatting | `lex-babel/src/transforms.rs` + `lex-lsp/src/features/formatting.rs` | Shared between CLI and LSP |
| Asset resolution | `lex_parser::lex::ast::links` | Generates file/document links for annotations and verbatim blocks |
| Annotation index | `lex-analysis/src/utils.rs` | Find annotations/definitions/sessions by position or label |
