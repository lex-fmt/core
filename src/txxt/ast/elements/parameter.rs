//! Parameter element
//!
//! A parameter is a key with a value, attached to
//! annotations and foreign blocks to convey structured metadata.
//!
//! Business use:
//! - Lightweight configuration
//! - Enables filtering, routing, and automation in tools
//!
//! Examples:
//! - `priority=high`
//!
//! Learn More:
//! - Parameters spec: docs/specs/v1/elements/parameters.txxt

use super::super::location::{Location, Position};
use std::fmt;

/// A parameter represents a key-value pair
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub key: String,
    pub value: String,
    pub location: Location,
}

impl Parameter {
    fn default_location() -> Location {
        Location::new(Position::new(0, 0), Position::new(0, 0))
    }

    pub fn new(key: String, value: String) -> Self {
        Self {
            key,
            value,
            location: Self::default_location(),
        }
    }
    pub fn with_location(mut self, location: Location) -> Self {
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
    fn test_parameter_with_location() {
        let location = super::super::super::location::Location::new(
            super::super::super::location::Position::new(1, 0),
            super::super::super::location::Position::new(1, 10),
        );
        let param =
            Parameter::new("key".to_string(), "value".to_string()).with_location(location);
        assert_eq!(param.location, location);
    }
}
