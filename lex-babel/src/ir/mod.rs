//! Intermediate Representation (IR) for lex documents.
//!
//! This module defines a format-agnostic representation of a lex document,
//! designed to facilitate conversion to various output formats like HTML,
//! Markdown, etc.
//!
//! # Design Philosophy
//!
//! The IR serves as a semantic, simplified layer between Lex AST and output formats:
//!
//! - **Semantic Focus**: Represents document meaning, not Lex-specific syntax
//! - **Lossy by Design**: Discards formatting details (blank lines, exact spacing)
//! - **Bidirectional**: Supports both Lex → IR and IR → Lex conversions
//! - **Format Agnostic**: Suitable for conversion to HTML, Markdown, etc.
//!
//! # When to Use
//!
//! Use the IR when converting to/from formats that don't map 1:1 with Lex structure:
//! - HTML, Markdown (need flattened, semantic representation)
//! - Pandoc JSON (different hierarchy model)
//! - Any format requiring structural transformation
//!
//! For formats that preserve Lex structure exactly (treeviz, tag), use the Lex AST directly.
//!
//! # Information Loss
//!
//! Converting Lex → IR → Lex is **lossy**. Lost information includes:
//! - Blank line grouping and precise spacing
//! - Source positions and token information
//! - Comment annotations at document level
//! - Exact inline formatting representation (converted to string markers)
//!
//! # Modules
//!
//! - [`nodes`]: Core IR data structures
//! - [`from_lex`]: Lex AST → IR conversion
//! - [`to_lex`]: IR → Lex AST conversion

pub mod from_lex;
pub mod nodes;
pub mod to_lex;
