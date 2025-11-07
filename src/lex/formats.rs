//! Output format implementations for AST serialization
//!
//! This module contains different format implementations for serializing
//! the parsed AST to various output formats.

pub mod registry;
pub mod tag;
pub mod treeviz;

pub use registry::{FormatError, FormatRegistry, Formatter};
pub use tag::{serialize_document as serialize_ast_tag, TagFormatter};
pub use treeviz::{to_treeviz_str, TreevizFormatter};
