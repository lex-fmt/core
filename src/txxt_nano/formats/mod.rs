//! Output format implementations for AST serialization
//!
//! This module contains different format implementations for serializing
//! the parsed AST to various output formats.

pub mod tag;
pub mod treeviz;

pub use tag::serialize_document as serialize_ast_tag;
pub use treeviz::to_treeviz_str;
