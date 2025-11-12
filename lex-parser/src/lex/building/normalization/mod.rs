//! Token normalization utilities
//!
//! This module provides functions to normalize various token formats into
//! standard `Vec<(Token, Range<usize>)>` representations for AST building.

pub(crate) mod normalize;
pub mod utilities;

// Re-export commonly used functions from normalize
pub(crate) use normalize::{flatten, normalize_line_token, normalize_line_tokens};

// Re-export utilities (individual items to avoid unused warning)
pub use utilities::{compute_bounding_box, extract_text, flatten_token_vecs};
