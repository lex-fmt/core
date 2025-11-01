//! AST Builder Module
//!
//! This module contains shared AST element builders used by both the reference and
//! grammar engine parsers. It provides the building blocks for constructing parse trees.
//!
//! ## Structure
//!
//! - `annotations.rs`: Annotation element building
//! - `definitions.rs`: Definition element building
//! - `document.rs`: Document element building
//! - `foreign.rs`: Foreign block element building
//! - `labels.rs`: Label element building
//! - `lists.rs`: List element building
//! - `parameters.rs`: Parameter element building
//! - `sessions.rs`: Session element building

pub mod annotations;
pub mod definitions;
pub mod document;
pub mod foreign;
pub mod labels;
pub mod lists;
pub mod parameters;
pub mod sessions;

// Re-export main builders
pub use document::document;
