//! Lex Workspace
//!
//!     A complete toolchain for working with Lex documents - a structured document format
//!     designed for authoring technical content, academic papers, and documentation.
//!
//! Architecture Overview
//!
//!     The workspace is organized into focused crates with clear separation of concerns.
//!     The processing pipeline flows from text input through parsing, then branches to
//!     multiple consumers:
//!
//!     Text Input → lex-parser → [lex-analysis, lex-babel, lex-lsp, lex-cli]
//!
//! Crate Organization
//!
//! lex-parser: Foundation
//!
//!     Text → AST transformation
//!
//!     - Lexes and parses Lex source into structured AST
//!     - Provides AST types: Document, Session, Paragraph, List, Definition, Annotation
//!     - Handles position tracking, range calculation, source mapping
//!     - Includes inline parsing: references, formatting markers
//!     - Testing utilities via `lexplore` for loading sample fixtures
//!
//! lex-analysis: Semantic Analysis
//!
//!     AST → Insights extraction
//!
//!     - Document analysis and navigation utilities (symbols, folding, hovers)
//!     - Reference resolution: go-to-definition, find-references
//!     - Completion provider for references, sessions, paths, and verbatim labels
//!     - Annotation helpers: navigation and resolution edits
//!     - Token classification for syntax highlighting
//!     - Preview text extraction for hover tooltips
//!     - Protocol-agnostic: reusable across LSP, CLI, editor plugins
//!
//! lex-babel: Format Transformation
//!
//!     AST → Multiple output formats
//!
//!     - Multi-format conversion: Lex ↔ Markdown, HTML, LaTeX, PDF
//!     - Publish pipeline returning on-disk artifacts or in-memory strings
//!     - Document serialization with formatting rules
//!     - Shared snippet templates for assets and verbatim blocks
//!     - Format-specific transformations and normalization
//!     - Pretty-printing with configurable indentation, blank lines, list markers
//!
//! lex-lsp: Editor Integration
//!
//!     LSP Server implementation
//!
//!     - Language Server Protocol implementation using tower-lsp
//!     - Rich editor support for any LSP-compatible editor
//!     - Delegates to lex-analysis for feature implementation
//!     - LSP-specific features: document formatting with TextEdit diffs, document links
//!     - Supported capabilities: semantic tokens, document symbols, hover, folding ranges,
//!       go-to-definition, find-references, formatting, document links
//!
//! lex-cli: Command-Line Interface
//!
//!     Terminal-based document processing
//!
//!     - Format documents, convert between formats, validate documents, query AST
//!     - `lex help` style documentation lookup over docs/ and specs/
//!     - Designed for scripting, CI/CD pipelines, manual document processing
//!
//! lex-viewer: TUI Viewer
//!
//!     Terminal User Interface for document browsing
//!
//!     - Interactive document viewer with navigation
//!     - Real-time rendering and preview
//!     - Built with ratatui
//!
//! Design Principles
//!
//! Separation of Concerns
//!
//!     - Parser: Syntax only, no semantics
//!     - Analysis: Semantic understanding, protocol-agnostic
//!     - Babel: Format conversion and transformation
//!     - LSP/CLI: Protocol and interface adapters
//!
//! Reusability
//!
//!     Core analysis logic in lex-analysis is reusable across multiple consumers:
//!     LSP server (editor integration), CLI tools (validation, refactoring), direct editor
//!     plugins (Vim, Emacs, without LSP), documentation generators, static analysis tools.
//!
//! Testing Strategy
//!
//!     - Use official sample files from `specs/` directory
//!     - `lexplore` loader for consistent test fixtures
//!     - Comprehensive unit tests in each crate
//!     - Integration tests at workspace level
//!
//! Migration Notes
//!
//! Phase 3 Reorganization (Nov 2024)
//!
//!     Analysis features were extracted from `lex-lsp` into the new `lex-analysis` crate to
//!     enable reuse across CLI tools, direct editor integrations, and other consumers.
//!
//!     Moved to lex-analysis:
//!         Document traversal and lookup utilities, reference resolution (go-to-def,
//!         find-refs), symbol hierarchy extraction, token classification, inline span
//!         detection, hover preview extraction, folding range detection.
//!
//!     Remained in lex-lsp:
//!         LSP server implementation (tower-lsp), protocol-specific formatting (TextEdit
//!         diff computation), document links (LSP wrapper).
//!
//!     This enables `lex-cli` and direct editor plugins to use analysis features without
//!     depending on the LSP protocol.
