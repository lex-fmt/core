//! Parameter element
//!
//! A parameter is  a pair of key and value.
//! annotations and foreign blocks to convey structured metadata.
//!
//! They can be used in annotations and, together with labels allow for structured metadata.
//!
//! Syntax:
//! <key> "=" <value>
//!
//! Examples:
//! - `priority=high`
//!
//! Learn More:
//! - Parameters spec: docs/specs/v1/elements/parameters.lex

use super::super::range::{Position, Range};
use std::fmt;

/// A parameter represents a key-value pair
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub key: String,
    pub value: String,
    pub location: Range,
}

impl Parameter {
    fn default_location() -> Range {
        Range::new(0..0, Position::new(0, 0), Position::new(0, 0))
    }

    pub fn new(key: String, value: String) -> Self {
        Self {
            key,
            value,
            location: Self::default_location(),
        }
    }

    /// Preferred builder
    pub fn at(mut self, location: Range) -> Self {
        self.location = location;
        self
    }
}

impl fmt::Display for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.key, self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter() {
        let location = super::super::super::range::Range::new(
            0..0,
            super::super::super::range::Position::new(1, 0),
            super::super::super::range::Position::new(1, 10),
        );
        let param = Parameter::new("key".to_string(), "value".to_string()).at(location.clone());
        assert_eq!(param.location, location);
    }
}
