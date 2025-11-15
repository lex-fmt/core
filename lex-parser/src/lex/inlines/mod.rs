//! Inline parsing primitives
//!
//! This module exposes the inline AST nodes plus the parser for flat inline
//! elements (formatting, code, math). Later stages layer references and
//! citations on top of the same building blocks.

pub mod nodes;
mod parser;

pub use nodes::{InlineContent, InlineNode, ReferenceInline, ReferenceType};
pub use parser::{
    parse_inlines, parse_inlines_with_parser, InlineKind, InlineParser, InlinePostProcessor,
    InlineSpec,
};
