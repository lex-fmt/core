//! Data Node
//!
//! Data nodes encapsulate the reusable :: label parameters? header shared by
//! annotations and future elements. They carry the label plus optional parameters
//! but no closing :: marker or content.

use super::super::range::{Position, Range};
use super::label::Label;
use super::parameter::Parameter;
use std::fmt;

/// Structured data payload extracted from `:: label params?` headers.
#[derive(Debug, Clone, PartialEq)]
pub struct Data {
    pub label: Label,
    pub parameters: Vec<Parameter>,
    pub location: Range,
}

impl Data {
    fn default_location() -> Range {
        Range::new(0..0, Position::new(0, 0), Position::new(0, 0))
    }

    pub fn new(label: Label, parameters: Vec<Parameter>) -> Self {
        Self {
            label,
            parameters,
            location: Self::default_location(),
        }
    }

    pub fn at(mut self, location: Range) -> Self {
        self.location = location;
        self
    }
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Data('{}', {} params)",
            self.label.value,
            self.parameters.len()
        )
    }
}
