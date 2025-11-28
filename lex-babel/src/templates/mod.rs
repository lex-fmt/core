//! Ready-to-insert snippets for Lex documents.
//!
//! These helpers produce canonical text fragments (verbatim blocks, annotations, etc.) that
//! respect the active formatting rules so editors can insert complex structures without reimplementing
//! Babel's serialization logic.

mod util;

pub mod asset;
pub mod verbatim;

pub use asset::{AssetKind, AssetSnippet, AssetSnippetRequest};
pub(crate) use util::normalize_path;
pub use verbatim::{build_verbatim_snippet, VerbatimSnippet, VerbatimSnippetRequest};
