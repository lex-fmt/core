# Required Rust APIs for Editor Features

    This document lists the Rust APIs required to support the features outlined in @[editors/README.lex].
    It tracks the status of each API, its signature, and the crate/module where it resides.

    Status Legend:
    - Live: Implemented and integrated into `lex-lsp`.
    - Rust Ready: Implemented in a crate but not yet exposed via LSP.
    - Planned: Not yet implemented.

2. Core & Navigation (LSP Standard)

    These features use standard LSP methods.

    | Feature | API / Command | Crate / Module | Signature (Approx) | Status |
    |---------|---------------|----------------|--------------------|--------|
    | Syntax Highlighting | N/A (Tree-sitter) | `lex-syntax` | N/A (Grammar) | Live |
    | Formatting | `textDocument/formatting` | `lex-lsp/src/features/formatting.rs` | `format_document(doc, source) -> Vec<TextEdit>` | Live |
    | Diagnostics | `textDocument/publishDiagnostics` | `lex-lsp/src/server.rs` | `parse_document(text) -> Result<Document, Vec<Error>>` | Live (Minimal) |
    | Document Symbols | `textDocument/documentSymbol` | `lex-lsp/src/features/document_symbols.rs` | `collect_document_symbols(doc) -> Vec<DocumentSymbol>` | Live |
    | Hover | `textDocument/hover` | `lex-lsp/src/features/hover.rs` | `hover(doc, pos) -> Option<HoverResult>` | Live |
    | Folding | `textDocument/foldingRange` | `lex-lsp/src/features/folding_ranges.rs` | `folding_ranges(doc) -> Vec<FoldingRange>` | Live |
    | Definition | `textDocument/definition` | `lex-lsp/src/features/go_to_definition.rs` | `goto_definition(doc, pos) -> Vec<Range>` | Live |
    | References | `textDocument/references` | `lex-lsp/src/features/references.rs` | `find_references(doc, pos) -> Vec<Range>` | Live |
    | Semantic Tokens | `textDocument/semanticTokens` | `lex-lsp/src/features/semantic_tokens.rs` | `collect_semantic_tokens(doc) -> Vec<SemanticToken>` | Live |
    | Completion | `textDocument/completion` | `lex-lsp/src/features/completion.rs` | `completion(doc, pos) -> Vec<CompletionItem>` | Planned |

3. Command-Based Features (`workspace/executeCommand`)

    These features rely on custom commands exposed by `lex-lsp`.

    | Feature | Command | Crate / Module | Signature (Arguments -> Result) | Status |
    |---------|---------|----------------|---------------------------------|--------|
    | Insert Asset | `lex.insert_asset` | `lex-lsp/src/features/commands.rs` | `(path: String) -> TextEdit` | Planned |
    | Insert Verbatim | `lex.insert_verbatim` | `lex-lsp/src/features/commands.rs` | `(path: String) -> TextEdit` | Planned |
    | Convert Doc | `lex.convert` | `lex-lsp/src/features/commands.rs` | `(format: "markdown" | "html") -> PathBuf` | Planned |
    | Export Doc | `lex.export` | `lex-lsp/src/features/commands.rs` | `(format: "pdf") -> PathBuf` | Planned |
    | Next Annotation | `lex.next_annotation` | `lex-lsp/src/features/commands.rs` | `(current_pos: Position) -> Position` | Planned |
    | Resolve Annotation | `lex.resolve_annotation` | `lex-lsp/src/features/commands.rs` | `(id: String) -> TextEdit` | Planned |
    | Toggle Annotations | `lex.toggle_annotations` | `lex-lsp/src/features/commands.rs` | `(enable: bool) -> void` | Planned |
    | Help | `lex.help` | `lex-lsp/src/features/commands.rs` | `(topic: Option<String>) -> String` | Planned |

4. Backend Support

    Underlying logic required by the above commands.

    | Logic | Crate | Description | Status |
    |-------|-------|-------------|--------|
    | Markdown Conversion | `lex-babel` | Convert Lex AST to Markdown | Live |
    | HTML Conversion | `lex-babel` | Convert Lex AST to HTML | Live |
    | PDF Export | `lex-babel` | Convert Lex AST to PDF (via HTML/Headless Chrome?) | Planned |
    | Asset Resolution | `lex-core` | Resolve relative paths for assets | Rust Ready |
    | Annotation Index | `lex-analysis` | Index and query annotations | Rust Ready |

