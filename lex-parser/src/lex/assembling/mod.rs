//! Assembling module
//!
//! The assembling stage processes parsed AST nodes to attach metadata and perform
//! post-parsing transformations. Unlike the parsing stage which converts tokens to AST,
//! assembling stages operate on the AST itself.
//!
//! Current stages:
//! - `attach_annotations`: Attaches annotations from content to AST nodes as metadata

pub mod stages;

pub use stages::AttachAnnotations;
