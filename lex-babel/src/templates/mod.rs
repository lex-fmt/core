//! Ready-to-insert snippets for Lex documents.
//!
//! These helpers produce canonical text fragments (verbatim blocks, annotations, etc.) that
//! respect the active formatting rules so editors can insert complex structures without reimplementing
//! Babel's serialization logic.

pub mod asset;

pub use asset::{AssetKind, AssetSnippet, AssetSnippetRequest};
