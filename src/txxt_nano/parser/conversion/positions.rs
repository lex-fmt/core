//! Position-preserving AST conversion functions (re-exported from ast_conversion.rs)
//!
//! Converts intermediate AST structures (with spans) to final AST structures
//! with both extracted text content AND source position information.

// Re-export conversion functions from ast_conversion.rs to consolidate duplication
#[allow(unused_imports)]
pub(crate) use crate::txxt_nano::parser::ast_conversion::{
    convert_annotation_with_positions, convert_content_item_with_positions,
    convert_definition_with_positions, convert_document_with_positions,
    convert_foreign_block_with_positions, convert_list_item_with_positions,
    convert_list_with_positions, convert_paragraph_with_positions, convert_session_with_positions,
};
