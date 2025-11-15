//! Multi-format interoperability for Lex documents
//!
//! This crate provides a uniform interface for converting between Lex AST and various
//! document formats (Markdown, HTML, Pandoc JSON, etc.).
//!
//! # Architecture
//!
//! - **Format trait**: Uniform interface for all formats (parsing and/or serialization)
//! - **FormatRegistry**: Centralized discovery and selection of formats
//! - **Format implementations**: Concrete implementations for each supported format
//!
//! # Examples
//!
//! ```ignore
//! use lex_babel::{FormatRegistry, Format};
//!
//! let registry = FormatRegistry::default();
//! let source = "# Hello\n\nWorld";
//! let doc = registry.parse(source, "markdown")?;
//! let output = registry.serialize(&doc, "html")?;
//! ```

pub mod error;
pub mod format;
pub mod formats;
pub mod registry;

pub use error::FormatError;
pub use format::Format;
pub use registry::FormatRegistry;
